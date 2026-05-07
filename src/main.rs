mod checker;
use crate::checker::Checker;
use crate::checker::check_set;
use crate::record::ParsedRecord;
use crate::record::Record;
use log::warn;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
mod csv_parser;
mod record;
mod song;
use clap::Parser;
use log::info;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the CSV file containing the playlists
    #[arg(short, long, default_value = "playlists.csv")]
    playlists: String,
    /// Output folder for the updated playlists
    #[arg(short, long, default_value = "playlists/")]
    output_folder: String,
    /// Input folder containing the original playlists
    #[arg(short, long, default_value = "playlists/")]
    input_folder: String,
    /// List of files to check
    #[arg(short, long)]
    file: Vec<String>,
    /// Update metadata of songs
    #[arg(short, long)]
    metadata: bool,
    /// Check availability of songs
    #[arg(short, long)]
    availability: bool,
    /// Update Video urls of songs to given location
    #[arg(short, long)]
    update_urls: bool,
    /// Countrycode for checking the urls
    #[arg(short, long, default_value = "DE")]
    country: String,
    /// Validate the hash and videoid of the songs in the playlists
    #[arg(short, long)]
    validate: bool,
    /// Set the logging level (error, warn, info, debug, trace)
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&args.log_level))
        .init();
    let all_sets = Record::get_sets(&args.playlists)?;

    let sets: Vec<Record> = if args.file.is_empty() {
        all_sets
    } else {
        all_sets
            .into_iter()
            .filter(|set| args.file.contains(&set.file))
            .collect()
    };

    if args.validate {
        sets.clone()
            .into_par_iter()
            .for_each(|record| check_set(&record.file, &args.input_folder));
    }
    if args.metadata || args.availability || args.update_urls {
        let checker = Checker::new();
        for set in sets {
            info!("Checking: {}", set.game);
            let mut parsed = ParsedRecord::from_record(set.clone(), &args.input_folder);
            if parsed.songs.is_empty() {
                warn!("No songs found for {} - skipping", set.game);
                continue;
            }

            parsed.validate_urls();

            if args.metadata {
                let n = checker.update_metadata(&mut parsed.songs).await?;
                info!("Updated metadata for {n} songs");
                parsed.write_songs_csv(&args.output_folder)?;
            }

            if args.availability || args.update_urls {
                let blocked = parsed.check_availability(&args.country).await.unwrap();

                for i in blocked {
                    let song = &mut parsed.songs[i];

                    info!(
                        "BLOCKED in {}: {} - {} ({})",
                        args.country, song.artist, song.title, song.url
                    );

                    if args.update_urls {
                        checker.update_song_url(song).await?;
                        info!("New Url: {}", song.url);
                    }
                }

                parsed.write_songs_csv(&args.output_folder)?;
            }
        }
    }

    Ok(())
}
