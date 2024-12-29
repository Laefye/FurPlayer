use std::time::{Duration, SystemTime};

use serde::Deserialize;
use tokio::{process::Command, sync::Mutex};

pub struct YtDlp {
    path: String,
    cache: Mutex<Vec<SavedMetadata>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct YouTubeMetadata {
    pub id: String,
    pub title: String,
    pub thumbnail: String,
    pub formats: Vec<YouTubeFormat>,
    pub channel: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct YouTubeFormat {
    pub url: String,
    pub ext: String,
    pub resolution: String,
}

#[derive(Debug, Clone)]
pub enum Error {
    Unknown,
    NotFound,
    PrivateVideo,
    BadLink,
    NotAudio,
}

#[derive(Debug)]
struct SavedMetadata {
    date: SystemTime,
    url: String,
    metadata: YouTubeMetadata,
}

impl YtDlp {
    pub fn new(path: String) -> Self {
        Self {
            path,
            cache: Mutex::new(Vec::new()),
        }
    }

    pub async fn fetch(&self, url: String) -> Result<YouTubeMetadata, Error> {
        if !url.starts_with("https://www.youtube.com/watch?v=") && !url.starts_with("https://youtu.be/") && !url.starts_with("https://youtube.com/watch?v=") {
            return Err(Error::BadLink);
        }
        {
            let cache = self.cache.lock().await;
            let cached = cache.iter()
                .filter(|c| c.url == url)
                .filter(|c| SystemTime::now().duration_since(c.date).unwrap() < Duration::from_secs(60 * 5))
                .next();
            if let Some(cached) = cached {
                return Ok(cached.metadata.clone());
            }
        }
        let mut cmd = Command::new(&self.path);
        // Костыльный костыль
        #[cfg(target_os = "windows")]
        {
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        cmd.args(vec![
            "--dump-json".to_string(),
            url.to_string(),
        ]);
        let output = cmd.output().await.map_err(|_| Error::Unknown)?;
        let stderr = String::from_utf8(output.stderr).map_err(|_| Error::Unknown)?;
        if !output.status.success() && !stderr.is_empty() {
            return if stderr.contains("Video unavailable") || stderr.contains("Incomplete YouTube ID") {
                Err(Error::NotFound)
            } else if stderr.contains("Private video") {
                Err(Error::PrivateVideo)
            } else {
                Err(Error::Unknown)
            };
        }
        let stdout = String::from_utf8(output.stdout).map_err(|_| Error::Unknown)?;
        let metadata = serde_json::from_str::<YouTubeMetadata>(&stdout).map_err(|_| Error::Unknown)?;
        self.cache.lock().await.push(SavedMetadata { date: SystemTime::now(), url, metadata: metadata.clone() });
        Ok(metadata)
    }
}

#[derive(Debug, Clone)]
pub struct YouTubeContentSource {
    pub thumbnail: String,
    pub media: String,
}

impl YouTubeMetadata {
    pub fn get_content(&self) -> Result<YouTubeContentSource, Error> {
        let format = self.formats.iter()
            .filter(|x| x.resolution == "audio only")
            .filter(|x| x.ext == "webm")
            .next();
        if format.is_none() {
            return Err(Error::NotAudio);
        }
        Ok(YouTubeContentSource {
            media: format.unwrap().url.clone(),
            thumbnail: self.thumbnail.clone(),
        })
    }
}
