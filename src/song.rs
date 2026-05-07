use regex::Regex;
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Debug, serde::Deserialize, Clone, Serialize)]
pub struct Song {
    #[serde(rename = "Card#")]
    pub number: u32,
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "Artist")]
    pub artist: String,
    #[serde(rename = "Year")]
    pub year: u16,
    #[serde(rename = "URL")]
    pub url: String,
    #[serde(rename = "Hashed Info")]
    pub hashed: String,
    #[serde(rename = "Youtube-Title")]
    pub youtube_title: String,
}

impl Song {
    pub fn check_hash(&self) -> bool {
        let hash = self.generate_hash();
        hash == self.hashed
    }

    pub fn generate_hash(&self) -> String {
        let combined_info = format!("{}{}{}", self.url, self.youtube_title, self.artist);
        let mut hasher = Sha256::new();
        hasher.update(combined_info.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    pub fn get_yt_id(&self) -> Option<String> {
        // 11 chars, allowed: A-Z a-z 0-9 _ -
        // Rexex from Microslop Copilot
        let re = Regex::new(
            r#"(?x)
        (?:^|[\s"'(])                                  # start or a common delimiter
        (?:https?://)?(?:www\.)?(?:m\.)?(?:music\.)?   # optional scheme + subdomain
        (?:youtu\.be/|youtube\.com/(?:watch\?v=|embed/|shorts/|v/))  # URL forms
        ([A-Za-z0-9_-]{11})                            # the id
        "#,
        )
        .ok()?;

        let id_only = Regex::new(r"^[A-Za-z0-9_-]{11}$").ok()?;
        if id_only.is_match(&self.url.trim()) {
            return Some(self.url.trim().to_string());
        }

        re.captures(&self.url)
            .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
    }

    pub(crate) fn refresh_hash(&mut self) {
        self.hashed = self.generate_hash();
    }
}
