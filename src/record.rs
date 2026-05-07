use log::{debug, error};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::{HashMap, HashSet};

use crate::{checker::Checker, csv_parser::read_csv, song::Song};

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Record {
    #[serde(rename = "File")]
    pub file: String,
    #[serde(rename = "Game")]
    pub game: String,
}

impl Record {
    pub fn get_sets(filename: &str) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        let records: Vec<Record> = match read_csv(&filename) {
            Ok(data) => data,
            Err(e) => {
                error!("Error while reading File '{}': {}", filename, e);
                return Err(e.into());
            }
        };
        Ok(records)
    }
}

#[derive(Debug, Clone)]
pub struct ParsedRecord {
    pub file: String,
    //pub game: String,
    pub songs: Vec<Song>,
}
impl ParsedRecord {
    pub fn from_record(record: Record, folder: &str) -> Self {
        let path = format!("{}/{}", folder.trim_end_matches('/'), record.file);
        debug!("reading {}", path);
        let songs: Vec<Song> = read_csv(&path).unwrap_or_else(|e| {
            error!("Error while parsing songs from {}: {}", record.file, e);
            Vec::new()
        });
        Self {
            file: record.file,
            //game: record.game,
            songs,
        }
    }
    pub fn write_songs_csv(&self, folder: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = format!("{}/{}", folder.trim_end_matches('/'), self.file);
        let mut wtr = csv::Writer::from_path(path)?;

        for song in &self.songs {
            wtr.serialize(song)?;
        }

        wtr.flush()?;
        Ok(())
    }
    pub fn validate_urls(&self) {
        <Vec<Song> as Clone>::clone(&self.songs)
            .into_par_iter()
            .for_each(|record| {
                let id = record.get_yt_id();
                if let None = id {
                    error!(
                        "File: {}, Song: {}, Artist: {} failed to parse URL: {}",
                        self.file, record.title, record.artist, record.url
                    );
                }
            });
    }
    pub async fn check_availability(
        &self,
        country: &str,
    ) -> Result<HashSet<usize>, Box<dyn std::error::Error>> {
        let id_to_index = self.build_id_index();
        let checker = Checker::new();

        let all_ids: Vec<String> = id_to_index.keys().cloned().collect();
        let mut blocked_indices: HashSet<usize> = HashSet::new();

        for chunk in all_ids.chunks(50) {
            let id_refs: Vec<&str> = chunk.iter().map(|s| s.as_str()).collect();

            match checker.check_availability(id_refs, country).await {
                Ok(results) => {
                    for (id, is_available) in results {
                        if !is_available {
                            if let Some(&idx) = id_to_index.get(&id) {
                                blocked_indices.insert(idx);
                            }
                        }
                    }
                }
                Err(e) => error!("Fehler beim API-Check: {}", e),
            }
        }

        Ok(blocked_indices)
    }
    fn build_id_index(&self) -> HashMap<String, usize> {
        self.songs
            .iter()
            .enumerate()
            .filter_map(|(i, song)| song.get_yt_id().map(|id| (id, i)))
            .collect()
    }
}
