use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::config::{File, LoadedData, Metadata, UrledData};

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
}

impl YtDlp {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub async fn get_metadata(&self, url: String) -> Result<YouTubeMetadata, Error> {
        let mut cmd = Command::new(&self.path);
        cmd.args(vec![
            "--dump-json".to_string(),
            url.to_string(),
        ]);
        let output = cmd.output().await.map_err(|_| Error::Unknown)?;
        let stderr = String::from_utf8(output.stderr).map_err(|_| Error::Unknown)?;
        if !output.status.success() && !stderr.is_empty() {
            return Err(Error::Unknown);
        }
        let stdout = String::from_utf8(output.stdout).unwrap();
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
            return Err(Error::Unknown);
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
