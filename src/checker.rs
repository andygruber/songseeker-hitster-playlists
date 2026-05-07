use crate::csv_parser::read_csv;
use crate::song::Song;
use log::debug;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
struct VideoResponse {
    items: Vec<VideoItem>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct VideoItem {
    id: String,
    snippet: Option<VideosSnippet>,
    status: Option<Status>,
    content_details: Option<ContentDetails>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VideosSnippet {
    title: String,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ContentDetails {
    region_restriction: Option<RegionRestriction>,
}

#[derive(Deserialize, Debug)]
struct RegionRestriction {
    blocked: Option<Vec<String>>,
    allowed: Option<Vec<String>>,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Status {
    embeddable: bool,
    upload_status: String,
}
#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub items: Vec<SearchItem>,
}

#[derive(Debug, Deserialize)]
pub struct SearchItem {
    pub id: SearchId,
    pub snippet: SearchSnippet,
}

#[derive(Debug, Deserialize)]
pub struct SearchSnippet {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchId {
    #[serde(rename = "videoId")]
    pub video_id: String,
}
pub fn check_set(filename: &str, folder: &str) {
    let path = format!("{}/{}", folder.trim_end_matches('/'), filename);

    let records: Vec<Song> = match read_csv(&path) {
        Ok(data) => data,
        Err(e) => {
            log::error!("skipping {} could not be read: ({})", filename, e);
            return;
        }
    };

    records.into_par_iter().for_each(|record| {
        let id = record.get_yt_id();
        if let Some(_) = id {
            if !record.check_hash() {
                log::warn!(
                    "File: {}, Song: {}, Artist: {} Hash mismatch",
                    filename,
                    record.title,
                    record.artist
                );
            }
        } else {
            log::error!(
                "File: {}, Song: {}, Artist: {} failed to parse URL: {}",
                filename,
                record.title,
                record.artist,
                record.url
            );
        }
    });
}
pub struct Checker {
    api_key: String,
    pub client: reqwest::Client,
}
impl Checker {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        let api_key = std::env::var("API_KEY").expect("API_KEY nicht gefunden");
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn check_availability(
        &self,
        video_ids: Vec<&str>,
        country_code: &str,
    ) -> Result<HashMap<String, bool>, Box<dyn std::error::Error>> {
        let ids_param = video_ids.join(",");
        let url = format!(
            "https://www.googleapis.com/youtube/v3/videos?part=contentDetails,status&id={}&key={}",
            ids_param, self.api_key
        );
        let resp = self.client.get(&url).send().await?;
        let status = resp.status();
        let body = resp.text().await?;

        debug!("HTTP status: {status}");
        debug!("Body: {body}");
        let response: VideoResponse = serde_json::from_str(&body)?;
        let mut videos: std::collections::HashMap<String, bool> = response
            .items
            .par_iter()
            .map(|item| {
                let mut is_available = true;

                if let Some(details) = &item.content_details {
                    if let Some(restriction) = &details.region_restriction {
                        if let Some(blocked) = &restriction.blocked {
                            if blocked.contains(&country_code.to_string()) {
                                is_available = false;
                            }
                        }
                        if let Some(allowed) = &restriction.allowed {
                            if !allowed.contains(&country_code.to_string()) {
                                is_available = false;
                            }
                        }

                        if let Some(status) = &item.status {
                            if !status.embeddable || status.upload_status != "processed" {
                                is_available = false;
                            }
                        }
                    }
                }

                (item.id.clone(), is_available)
            })
            .collect();
        for id in video_ids {
            videos.entry(id.to_string()).or_insert(false);
        }
        Ok(videos)
    }
    pub async fn update_song_url(&self, song: &mut Song) -> Result<bool, reqwest::Error> {
        let query = format!("{} - {}", song.artist, song.title);

        let url = match reqwest::Url::parse_with_params(
            "https://www.googleapis.com/youtube/v3/search",
            &[
                ("part", "snippet"),
                ("type", "video"),
                ("maxResults", "1"),
                ("q", query.as_str()),
                ("key", self.api_key.as_str()),
            ],
        ) {
            Ok(u) => u,
            Err(_) => return Ok(false),
        };

        let resp: SearchResponse = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let Some(first) = resp.items.first() else {
            return Ok(false);
        };

        song.url = format!("https://www.youtube.com/watch?v={}", first.id.video_id);
        song.youtube_title = first.snippet.title.clone();
        song.hashed = song.generate_hash();
        Ok(true)
    }
    pub async fn update_metadata(
        &self,
        songs: &mut [Song],
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let mut id_to_indices: HashMap<String, Vec<usize>> = HashMap::new();
        let mut ids: Vec<String> = Vec::new();
        for (idx, song) in songs.iter().enumerate() {
            if song.check_hash() {
                debug!("Skiping {} ", song.title);
                continue;
            }

            let Some(id) = song.get_yt_id() else {
                continue;
            };

            if let Some(v) = id_to_indices.get_mut(&id) {
                v.push(idx);
            } else {
                id_to_indices.insert(id.clone(), vec![idx]);
                ids.push(id);
            }
        }

        let mut updated = 0usize;

        for chunk in ids.chunks(50) {
            let ids_param = chunk.join(",");

            let url = reqwest::Url::parse_with_params(
                "https://www.googleapis.com/youtube/v3/videos",
                &[
                    ("part", "snippet"),
                    ("id", ids_param.as_str()),
                    ("key", self.api_key.as_str()),
                ],
            )?;

            let resp: VideoResponse = self
                .client
                .get(url)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            let returned_ids: std::collections::HashSet<String> =
                resp.items.iter().map(|item| item.id.clone()).collect();

            for id in chunk {
                if !returned_ids.contains(id) {
                    log::warn!("YouTube API did not return metadata for '{}'", id);
                }
            }
            for item in resp.items {
                let Some(indices) = id_to_indices.get(&item.id) else {
                    continue;
                };

                for &i in indices {
                    let song = &mut songs[i];

                    if let Some(snippet) = &item.snippet {
                        song.youtube_title = snippet.title.clone();
                    }

                    song.url = format!("https://www.youtube.com/watch?v={}", item.id);
                    song.refresh_hash();
                    updated += 1;
                }
            }
        }

        Ok(updated)
    }
}
