use serde::Deserialize;

use super::{FetchError, Details, YtDlp};

#[derive(Debug, Deserialize, Clone)]
struct YouTubeVideo {
    pub id: String,
    pub title: String,
    pub thumbnail: String,
    pub formats: Vec<YouTubeFormat>,
    pub channel: String,
}

#[derive(Debug, Deserialize, Clone)]
struct YouTubeFormat {
    pub url: String,
    pub ext: String,
    pub resolution: String,
}

impl YtDlp {
    pub fn is_youtube(&self, url: String) -> bool {
        url.contains("youtube.com") || url.contains("youtu.be")
    }

    pub async fn fetch_youtube(&self, url: String) -> Result<Details, FetchError> {
        let mut cmd = self.get_command(url.clone());
        let output = cmd.output().await.map_err(|_| FetchError::Unknown)?;
        let stderr = String::from_utf8(output.stderr).map_err(|_| FetchError::Unknown)?;
        if !output.status.success() && !stderr.is_empty() {
            return if stderr.contains("Video unavailable") || stderr.contains("Incomplete YouTube ID") {
                Err(FetchError::NotFound)
            } else if stderr.contains("Private video") {
                Err(FetchError::NotFound)
            } else {
                Err(FetchError::Unknown)
            };
        }
        let stdout = String::from_utf8(output.stdout).map_err(|_| FetchError::Unknown)?;
        let metadata = serde_json::from_str::<YouTubeVideo>(&stdout).map_err(|_| FetchError::Unknown)?;
        Ok(Details {
            url: format!("https://www.youtube.com/watch?v={}", metadata.id),
            title: metadata.title,
            thumbnail: metadata.thumbnail,
            media: metadata.formats.iter()
                .filter(|x| x.resolution == "audio only")
                .filter(|x| x.ext == "webm")
                .next()
                .map(|x| x.url.clone())
                .ok_or(FetchError::NotFound)?,
            author: metadata.channel,
        })
    }
}