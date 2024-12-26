use std::{path::Path, sync::Arc};

use tokio::{fs::File, io::AsyncWriteExt, sync::Mutex};

use crate::config::{self, LoadedData, Metadata, UrledData};

pub struct Storage {
    audio_dir: String,
}

impl Storage {
    pub fn new(audio_dir: String) -> Self {
        Self {
            audio_dir,
        }
    }

    async fn download_file(url: String, file_path: String) {
        let mut file = File::create(file_path).await.unwrap();
        let mut response = reqwest::get(&url).await.unwrap();
        let size = response.content_length().unwrap();
        let mut count = 0; 
        while let Some(chunk) = response.chunk().await.unwrap() {
            file.write_all(&chunk).await.unwrap();
            count += chunk.len();
            Self::log(format!("Downloaded {} bytes / {} bytes", count, size));
        }
    }

    fn log(message: String) {
        println!("{}", message);
    }

    pub fn has_audio(&self, id: u32) -> bool {
        let audio_dir = Path::new(&self.audio_dir).join(id.to_string());
        let thumbnail_path = audio_dir.join("thumbnail.jpeg");
        let audio_path = audio_dir.join("audio.webm");
        thumbnail_path.exists() && audio_path.exists()
    }

    pub async fn start_download(storage: Arc<Mutex<Self>>, urled: UrledData, metadata: Metadata) {
        let temp = std::env::temp_dir();
        let id = metadata.id;
        let temp_thumbnail_path = temp.join(id.to_string() + "_thumbnail.jpeg").to_str().unwrap().to_string();
        let temp_audio_path = temp.join(id.to_string() + "_audio.webm").to_str().unwrap().to_string();
        let downloading_audio = Self::download_file(urled.audio, temp_audio_path.clone());
        let downloading_thumbnail = Self::download_file(urled.thumbnail, temp_thumbnail_path.clone());
        let audio_dir;
        {
            let storage = storage.lock().await;
            audio_dir = Path::new(&storage.audio_dir).join(id.to_string()).to_str().unwrap().to_string();
        }
        Self::log(format!("Downloading audio and thumbnail for {}", id));
        tokio::join!(downloading_audio, downloading_thumbnail);
        let real_thumbnail_path = Path::new(&audio_dir).join(id.to_string()).join("thumbnail.jpeg").to_str().unwrap().to_string();
        let real_audio_path = Path::new(&audio_dir).join(id.to_string()).join("audio.webm").to_str().unwrap().to_string();
        tokio::fs::create_dir_all(Path::new(&audio_dir).join(id.to_string())).await.unwrap();
        tokio::fs::rename(temp_thumbnail_path, real_thumbnail_path).await.unwrap();
        tokio::fs::rename(temp_audio_path, real_audio_path).await.unwrap();
    }

    pub async fn load(&self, id: u32) -> LoadedData {
        let audio_dir = Path::new(&self.audio_dir).join(id.to_string());
        let thumbnail_bytes = tokio::fs::read(audio_dir.join("thumbnail.jpeg")).await.unwrap();
        let audio_bytes = tokio::fs::read(audio_dir.join("audio.webm")).await.unwrap();
        LoadedData {
            thumbnail: config::File {
                bytes: thumbnail_bytes,
                mime: "image/jpeg".to_string(),
            },
            audio: config::File {
                bytes: audio_bytes,
                mime: "audio/webm".to_string(),
            },
        }
    }
}
