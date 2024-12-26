use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::config::{File, Metadata, UrledData};

pub struct YtDlp {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YouTubeMetadata {
    pub id: String,
    pub title: String,
    pub description: String,
    pub thumbnail: String,
    pub formats: Vec<YouTubeFormat>,
    pub channel: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YouTubeFormat {
    pub format: String,
    pub url: String,
    pub ext: String,
    pub resolution: String,
    pub format_id: String,
    pub protocol: String,
    pub abr: Option<f64>,
}

#[derive(Debug)]
pub enum Error {
    Unknown,
    NotFound,
    PrivateVideo,
    BadLink,
    NotAudio,
}

impl YtDlp {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub async fn get_metadata(&self, url: String) -> Result<YouTubeMetadata, Error> {
        if !url.starts_with("https://www.youtube.com/watch?v=") && !url.starts_with("https://youtu.be/") && !url.starts_with("https://youtube.com/watch?v=") {
            return Err(Error::BadLink);
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
        serde_json::from_str(&stdout).map_err(|_| Error::Unknown)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YouTubeLoadedMusic {
    pub id: String,
    pub title: String,
    pub description: String,
    pub thumbnail: File,
    pub audio: File,
}

impl YouTubeMetadata {
    pub fn get_urled_data(&self) -> Result<UrledData, Error> {
        let format = self.formats.iter()
            .filter(|x| x.resolution == "audio only")
            .filter(|x| x.ext == "webm")
            .next();
        if format.is_none() {
            return Err(Error::NotAudio);
        }
        Ok(UrledData {
            audio: format.unwrap().url.clone(),
            thumbnail: self.thumbnail.clone(),
        })
    }

    pub fn create_metadata(&self, id: u32) -> Metadata {
        Metadata {
            id,
            title: self.title.clone(),
            author: self.channel.clone(),
            platform: crate::config::Platform::YouTube(format!("https://www.youtube.com/watch?v={}", self.id)),
        }
    } 
}
