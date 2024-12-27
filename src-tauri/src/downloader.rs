use std::{collections::HashMap, future::Future, io::Write, sync::Arc};

use tokio::sync::Mutex;

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
            if !callback(length, size).await {
                return Err(Error::Canceled);
            }
        }
        Ok(Content { mime })
    }
}

pub trait Storage {
    async fn save<C, Fut>(&self, audio: &Audio, callback: C, downloads: HashMap<String, String>) -> Result<(), Error>
    where
        C: Fn(u64, u64) -> Fut,
        Fut: Future<Output = bool>;

    async fn has_file(&self, audio: &Audio, file: String) -> bool;

    async fn is_in_queue(&self, id: u32) -> bool;
}

#[derive(Debug)]
pub struct MemoryStorage<D: ContentRetriever> {
    map: Mutex<HashMap<u32, HashMap<String, Vec<u8>>>>,
    queue: Mutex<Vec<u32>>,
    content_retriever: D,
}

impl<D: ContentRetriever> MemoryStorage<D> {
    pub fn new(content_retriever: D) -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
            queue: Mutex::new(Vec::new()),
            content_retriever,
        }
    }
}

impl<D: ContentRetriever> Storage for MemoryStorage<D> {
    async fn save<C, Fut>(&self, audio: &Audio, callback: C, downloads: HashMap<String, String>) -> Result<(), Error>
    where
        C: Fn(u64, u64) -> Fut,
        Fut: Future<Output = bool>
    {
        {
            let mut queue = self.queue.lock().await;
            if queue.contains(&audio.id) {
                return Err(Error::InQueue);
            }
            queue.push(audio.id);
        }
        let mut downloaded = HashMap::new();
        for (filename, url) in downloads {
            let mut bytes = Vec::new();
            self.content_retriever.download(url, &mut bytes, &callback).await?;
            downloaded.insert(filename, bytes);
        }
        self.map.lock().await.insert(audio.id, downloaded);
        self.queue.lock().await.retain(|x| *x != audio.id);
        Ok(())
    }
    
    async fn has_file(&self, audio: &Audio, file: String) -> bool {
        self.map.lock().await.get(&audio.id).map(|x| x.contains_key(&file)).unwrap_or(false)
    }
    
    async fn is_in_queue(&self, id: u32) -> bool {
        self.queue.lock().await.contains(&id)
    }
}
