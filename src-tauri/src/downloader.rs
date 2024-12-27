use std::{collections::HashMap, future::Future, io::Write, path::Path, sync::Arc};

use tokio::{io::AsyncWriteExt, sync::Mutex};

use crate::audio::Audio;

#[derive(Debug)]
pub struct Content {
    pub mime: String,
}

#[derive(Debug)]
pub enum Error {
    Unknown,
    Canceled,
    InQueue,
    NotFound,
}

pub trait ContentRetriever {
    async fn download<F, Fut>(&self, url: String, writer: &mut impl Write, callback: F) -> Result<Content, Error>
    where
        F: Fn(u64, u64) -> Fut,
        Fut: Future<Output = bool>;
}

#[derive(Debug)]
pub struct DefaultContentRetriever;

impl ContentRetriever for DefaultContentRetriever {
    async fn download<F, Fut>(&self, url: String, writer: &mut impl Write, callback: F) -> Result<Content, Error>
    where
        F: Fn(u64, u64) -> Fut,
        Fut: Future<Output = bool>,
    {
        let mut response = reqwest::get(url).await.map_err(|_| Error::Unknown)?;
        let size = response.content_length().unwrap_or(0);
        let mime = response.headers().get("Content-Type").map(|x| x.to_str().unwrap_or("")).unwrap_or("").to_string();
        let mut length = 0;
        while let Some(chunk) = response.chunk().await.map_err(|_| Error::Unknown)? {
            writer.write_all(&chunk).map_err(|_| Error::Unknown)?;
            length += chunk.len() as u64;
            println!("Downloaded {} bytes / {} bytes", length, size);
            if !callback(length, size).await {
                return Err(Error::Canceled);
            }
        }
        Ok(Content { mime })
    }
}

#[derive(Debug)]
pub struct RequestFiles {
    pub thumbnail: String,
    pub audio: String,
}

#[derive(Debug)]
pub struct ResponseFiles {
    pub thumbnail: Vec<u8>,
    pub audio: Vec<u8>,
}

impl RequestFiles {
    pub fn new(thumbnail: String, audio: String) -> Self {
        Self { thumbnail, audio }
    }
}

pub trait Storage {
    async fn save<C, Fut>(&self, audio: &Audio, callback: C, downloads: RequestFiles) -> Result<(), Error>
    where
        C: Fn(u64, u64) -> Fut,
        Fut: Future<Output = ()>;

    async fn has_file(&self, audio: &Audio) -> bool;

    async fn is_in_queue(&self, id: u32) -> bool;

    async fn get_files(&self, audio: &Audio) -> Result<ResponseFiles, Error>;
}

// #[derive(Debug)]
// pub struct MemoryStorage<D: ContentRetriever> {
//     map: Mutex<HashMap<u32, HashMap<String, Vec<u8>>>>,
//     queue: Mutex<Vec<u32>>,
//     content_retriever: D,
// }

// impl<D: ContentRetriever> MemoryStorage<D> {
//     pub fn new(content_retriever: D) -> Self {
//         Self {
//             map: Mutex::new(HashMap::new()),
//             queue: Mutex::new(Vec::new()),
//             content_retriever,
//         }
//     }
// }

// impl<D: ContentRetriever> Storage for MemoryStorage<D> {
//     async fn save<C, Fut>(&self, audio: &Audio, callback: C, downloads: HashMap<String, String>) -> Result<(), Error>
//     where
//         C: Fn(u64, u64) -> Fut,
//         Fut: Future<Output = bool>
//     {
//         {
//             let mut queue = self.queue.lock().await;
//             if queue.contains(&audio.id) {
//                 return Err(Error::InQueue);
//             }
//             queue.push(audio.id);
//         }
//         let mut downloaded = HashMap::new();
//         for (filename, url) in downloads {
//             let mut bytes = Vec::new();
//             self.content_retriever.download(url, &mut bytes, &callback).await?;
//             downloaded.insert(filename, bytes);
//         }
//         self.map.lock().await.insert(audio.id, downloaded);
//         self.queue.lock().await.retain(|x| *x != audio.id);
//         Ok(())
//     }
    
//     async fn has_file(&self, audio: &Audio, file: String) -> bool {
//         self.map.lock().await.get(&audio.id).map(|x| x.contains_key(&file)).unwrap_or(false)
//     }
    
//     async fn is_in_queue(&self, id: u32) -> bool {
//         self.queue.lock().await.contains(&id)
//     }
// }

pub struct FileDownloader {
    audio_dir: String,
    content_retriever: DefaultContentRetriever,
}

impl FileDownloader {
    pub fn new(audio_dir: String) -> Self {
        Self {
            audio_dir,
            content_retriever: DefaultContentRetriever,
        }
    }

    async fn donwload_file<C, Fut>(&self, audio: &Audio, callback: C, url: String, filename: String) -> Result<(), Error>
    where
        C: Fn(u64, u64) -> Fut,
        Fut: Future<Output = ()>,
    {
        let audio_dir = Path::new(&self.audio_dir).join(audio.id.to_string());
        let mut file = tokio::fs::File::create(audio_dir.join(filename)).await.map_err(|_| Error::Unknown)?;
        let mut bytes = Vec::new();
        self.content_retriever.download(url, &mut bytes, |downloaded, total| {
            let callback = &callback;
            async move {
                callback(downloaded, total).await;
                // self.is_in_queue(audio.id).await
                true
            }
        }).await?;
        file.write_all(&bytes).await.map_err(|_| Error::Unknown)?;
        Ok(())
    }
}

impl Storage for FileDownloader {
    async fn save<C, Fut>(&self, audio: &Audio, callback: C, downloads: RequestFiles) -> Result<(), Error>
    where
        C: Fn(u64, u64) -> Fut,
        Fut: Future<Output = ()>
    {
        // TODO: А очередь будет?
        let audio_dir = Path::new(&self.audio_dir).join(audio.id.to_string());
        tokio::fs::create_dir_all(&audio_dir).await.map_err(|_| Error::Unknown)?;
        self.donwload_file(audio, &callback, downloads.thumbnail, "thumbnail.jpeg".to_string()).await?;
        self.donwload_file(audio, &callback, downloads.audio, "audio.webm".to_string()).await?;
        Ok(())
    }
    
    async fn has_file(&self, audio: &Audio) -> bool {
        let audio_dir = Path::new(&self.audio_dir).join(audio.id.to_string());
        tokio::fs::metadata(audio_dir.join("thumbnail.jpeg")).await.is_ok() && tokio::fs::metadata(audio_dir.join("audio.webm")).await.is_ok()
    }
    
    async fn is_in_queue(&self, id: u32) -> bool {
        false
    }
    
    async fn get_files(&self, audio: &Audio) -> Result<ResponseFiles, Error> {
        let audio_dir = Path::new(&self.audio_dir).join(audio.id.to_string());
        let thumbnail = tokio::fs::read(audio_dir.join("thumbnail.jpeg")).await.map_err(|_| Error::NotFound)?;
        let audio = tokio::fs::read(audio_dir.join("audio.webm")).await.map_err(|_| Error::NotFound)?;
        Ok(ResponseFiles { thumbnail, audio })
    }
}
