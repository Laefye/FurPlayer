use std::{path::Path, sync::Arc, time::Duration};

use tokio::{fs::File, io::AsyncWriteExt, sync::Mutex, time::sleep};

use crate::config::{self, LoadedData, Metadata, UrledData};

struct Manager {
    pub audio_dir: String,
    pub queue: Vec<u32>,
}

#[derive(Clone)]
pub struct Storage {
    manager: Arc<Mutex<Manager>>,
}

#[derive(Debug)]
pub enum StorageError {
    Internet(reqwest::Error)
}

impl Manager {
    pub fn new(audio_dir: String) -> Self {
        Self {
            audio_dir,
            queue: Vec::new(),
        }
    }

    pub fn has_audio(&self, id: u32) -> bool {
        let audio_dir = Path::new(&self.audio_dir).join(id.to_string());
        let thumbnail_path = audio_dir.join("thumbnail.jpeg");
        let audio_path = audio_dir.join("audio.webm");
        thumbnail_path.exists() && audio_path.exists()
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

impl Storage {
    pub fn new(audio_dir: String) -> Self {
        Self {
            manager: Arc::new(Mutex::new(Manager::new(audio_dir))),
        }
    }

    pub async fn has_audio(&self, id: u32) -> bool {
        let manager = self.manager.lock().await;
        manager.has_audio(id)
    }

    pub async fn load(&self, id: u32) -> LoadedData {
        let manager = self.manager.lock().await;
        manager.load(id).await
    }

    fn make_request(&self, url: String, starts: Option<usize>) -> reqwest::RequestBuilder {
        let client = reqwest::Client::new();
        let mut builder = client.request(reqwest::Method::GET, url);
        if let Some(starts) = starts {
            builder = builder.header("Range", format!("bytes={}-", starts));
        }
        builder
    }

    async fn download_file(&self, url: String, file_path: String, progress: impl Fn(f64) -> ()) -> Result<(), StorageError> {
        let mut file = File::create(file_path).await.unwrap();
        let mut count = 0; 
        'attempts: for i in 0..5 {
            let mut response = self.make_request(url.clone(), if count > 0 { Some(count) } else { None }).send().await.map_err(StorageError::Internet)?;
            let size = response.content_length().unwrap();
            loop {
                match response.chunk().await {
                    Ok(chunk) => {
                        if chunk.is_none() {
                            break 'attempts;
                        }
                        let chunk = chunk.unwrap();
                        count += chunk.len();
                        file.write_all(&chunk).await.unwrap();
                        println!("Downloaded {} bytes / {} bytes", count, size);
                        progress((count as f64) / (size as f64));
                    },
                    Err(err) => {
                        if err.is_connect() || err.is_timeout() {
                            println!("Trying again... {}", i);
                            sleep(Duration::from_secs(1)).await;
                            if i == 4 {
                                return Err(StorageError::Internet(err));
                            }
                            continue 'attempts;
                        }
                    }
                }
            }
        }
        println!("Downloaded!");
        Ok(())
    }

    async fn get_audio_dir(&self, id: u32) -> String {
        let manager = self.manager.lock().await;
        let audio_dir = Path::new(&manager.audio_dir).join(id.to_string());
        audio_dir.to_str().unwrap().to_string()
    }

    pub async fn is_in_queue(&self, id: u32) -> bool {
        let manager = self.manager.lock().await;
        manager.queue.contains(&id)
    }

    pub async fn download(&self, urled: UrledData, metadata: Metadata, progress: impl Fn(f64) -> ()) -> Result<(), StorageError> {
        let temp = std::env::temp_dir();
        let temp_thumbnail_path = temp.join(metadata.id.to_string() + "_thumbnail.jpeg").to_str().unwrap().to_string();
        let temp_audio_path = temp.join(metadata.id.to_string() + "_audio.webm").to_str().unwrap().to_string();
        {
            let mut manager = self.manager.lock().await;
            manager.queue.push(metadata.id);
        }
        let thumbnail = self.download_file(urled.thumbnail, temp_thumbnail_path.clone(), &progress).await;
        let audio = self.download_file(urled.audio, temp_audio_path.clone(), &progress).await;
        if thumbnail.is_ok() && audio.is_ok() {
            let audio_dir = self.get_audio_dir(metadata.id).await;
            let audio_path = Path::new(&audio_dir);
            tokio::fs::create_dir_all(audio_path).await.unwrap();
            tokio::fs::rename(temp_thumbnail_path, audio_path.join("thumbnail.jpeg")).await.unwrap();
            tokio::fs::rename(temp_audio_path, audio_path.join("audio.webm")).await.unwrap();
        }
        {
            let mut manager = self.manager.lock().await;
            manager.queue.retain(|&x| x != metadata.id);
        }
        if thumbnail.is_err() {
            Err(thumbnail.unwrap_err())
        } else if audio.is_err() {
            Err(audio.unwrap_err())
        } else {
            Ok(())
        }
    }
}
