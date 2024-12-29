use std::time::{Duration, SystemTime};

use tokio::{process::Command, sync::Mutex};

pub struct YtDlp {
    path: String,
    cache: Mutex<Vec<OtherSavedDetails>>,
}


#[derive(Debug, Clone)]
pub enum FetchError {
    Unknown,
    NotFound,
    BadLink,
}

#[derive(Debug)]
struct OtherSavedDetails {
    date: SystemTime,
    url: String,
    details: Details,
}

#[derive(Debug, Clone)]
pub struct Details {
    pub url: String,
    pub title: String,
    pub author: String,
    pub thumbnail: String,
    pub media: String,
}

mod youtube;

impl YtDlp {
    pub fn new(path: String) -> Self {
        Self {
            path,
            cache: Mutex::new(Vec::new()),
        }
    }

    async fn save_cache(&self, url: String, metadata: Details) {
        self.cache.lock().await.push(OtherSavedDetails {
            date: SystemTime::now(),
            url,
            details: metadata,
        });
    }

    async fn get_cache(&self, url: String) -> Option<Details> {
        let cache = self.cache.lock().await;
        cache.iter()
            .filter(|c| c.url == url)
            .filter(|c| SystemTime::now().duration_since(c.date).unwrap() < Duration::from_secs(60 * 5))
            .map(|c| c.details.clone())
            .next()
    }
    
    fn get_command(&self, url: String) -> Command {
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
        cmd
    }

    pub async fn fetch(&self, url: String) -> Result<Details, FetchError> {
        if let Some(cached) = self.get_cache(url.clone()).await {
            return Ok(cached);
        } else if self.is_youtube(url.clone()) {
            let details = self.fetch_youtube(url.clone()).await?;
            self.save_cache(url.clone(), details.clone()).await;
            Ok(details)
        } else {
            Err(FetchError::BadLink)
        }
    }
}
