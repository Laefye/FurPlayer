use std::{future::Future, io::Write, path::Path};

use mime2ext::mime2ext;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use tokio::{io::AsyncWriteExt, sync::{broadcast, Mutex}};

use crate::audio::Audio;

#[derive(Debug)]
pub struct Content {
    pub mime: String,
}

#[derive(Debug, Clone)]
pub enum Error {
    Unknown,
    Connection,
    Canceled,
    InQueue,
    NotFound,
}

pub trait ContentRetriever {
    async fn download<F, Fut>(&self, url: String, writer: &mut impl Write, callback: F, skip: usize) -> Result<Content, Error>
    where
        F: Fn(u64, u64) -> Fut,
        Fut: Future<Output = bool>;
}

#[derive(Debug)]
pub struct DefaultContentRetriever;

impl ContentRetriever for DefaultContentRetriever {
    async fn download<F, Fut>(&self, url: String, writer: &mut impl Write, callback: F, skip: usize) -> Result<Content, Error>
    where
        F: Fn(u64, u64) -> Fut,
        Fut: Future<Output = bool>,
    {
        let mut request = reqwest::Client::new().request(Method::GET, url);
        if skip > 0 {
            request = request.header("Range", format!("bytes={}-", skip));
        }
        let mut response = request.send()
            .await
            .map_err(|_| Error::Connection)?;
        let size = response.content_length().unwrap_or(0);
        let mime = response.headers().get("Content-Type").map(|x| x.to_str().unwrap_or("")).unwrap_or("").to_string();
        let mut length = 0;
        while let Some(chunk) = response.chunk().await.map_err(|_| Error::Connection)? {
            writer.write_all(&chunk).map_err(|_| Error::Unknown)?;
            length += chunk.len() as u64;
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
    pub media: String,
}

#[derive(Debug)]
pub struct ResponseFiles {
    pub thumbnail: Vec<u8>,
    pub thumbnail_mime: String,
    pub media: Vec<u8>,
    pub media_mime: String,
}

impl RequestFiles {
    pub fn new(thumbnail: String, media: String) -> Self {
        Self { thumbnail, media }
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

    async fn remove(&self, id: u32) -> Result<(), Error>;

}

pub struct FileDownloader {
    audio_dir: String,
    downloading_dir: String,
    content_retriever: DefaultContentRetriever,
    queue: Mutex<Vec<u32>>,

    cancel_broadcast: broadcast::Sender<u32>,
}

impl FileDownloader {
    pub fn new(audio_dir: String, downloading_dir: String) -> Self {
        Self {
            audio_dir,
            downloading_dir,
            content_retriever: DefaultContentRetriever,
            queue: Mutex::new(Vec::new()),

            cancel_broadcast: broadcast::channel(10).0,
        }
    }

    async fn download_files<C, Fut>(&self, audio: &Audio, callback: C, downloads: RequestFiles) -> Result<(), Error>
    where
        C: Fn(u64, u64) -> Fut,
        Fut: Future<Output = ()>,
    {
        let audio_dir = Path::new(&self.audio_dir).join(audio.id.to_string());
        let downloading_dir = Path::new(&self.downloading_dir).join(audio.id.to_string());
        tokio::fs::create_dir_all(&audio_dir).await.map_err(|_| Error::Unknown)?;
        tokio::fs::create_dir_all(&downloading_dir).await.map_err(|_| Error::Unknown)?;
        let thumbnail_content = self.donwload_file(audio, &callback, downloads.thumbnail, "thumbnail.bin".to_string()).await?;
        let audio_content = self.donwload_file(audio, &callback, downloads.media, "media.bin".to_string()).await?;
        tokio::fs::rename(downloading_dir.join("thumbnail.bin"), audio_dir.join(format!("thumbnail.{}", mime2ext(thumbnail_content.mime.clone()).unwrap_or("bin")))).await.map_err(|_| Error::Unknown)?;
        tokio::fs::rename(downloading_dir.join("media.bin"), audio_dir.join(format!("media.{}", mime2ext(audio_content.mime.clone()).unwrap_or("bin")))).await.map_err(|_| Error::Unknown)?;
        let index = Index {
            media_mime: audio_content.mime,
            thumbnail_mime: thumbnail_content.mime,
        };
        let index = serde_json::to_string(&index).map_err(|_| Error::Unknown)?;
        tokio::fs::write(audio_dir.join("index.json"), index).await.map_err(|_| Error::Unknown)?;
        Ok(())
    }

    async fn donwload_file<C, Fut>(&self, audio: &Audio, callback: C, url: String, filename: String) -> Result<Content, Error>
    where
        C: Fn(u64, u64) -> Fut,
        Fut: Future<Output = ()>,
    {
        let downloading_dir = Path::new(&self.downloading_dir).join(audio.id.to_string());
        let mut file = tokio::fs::File::create(downloading_dir.join(filename)).await.map_err(|_| Error::Unknown)?;
        let mut bytes = Vec::new();
        let mut result = Err(Error::Unknown);
        for _attempt in 0..5 {
            let len = bytes.len();
            result = self.content_retriever.download(url.clone(), &mut bytes, |downloaded, total| {
                let callback = &callback;
                async move {
                    callback(downloaded, total).await;
                    self.is_in_queue(audio.id).await
                }
            }, len).await;
            match &result {
                Err(Error::Canceled) => { break; },
                Ok(_) => { break; }
                _ => {},
            }
        }
        match result {
            Ok(content) => {
                file.write_all(&bytes).await.map_err(|_| Error::Unknown)?;
                Ok(content)
            }
            Err(Error::Canceled) => {
                self.cancel_broadcast.send(audio.id).map_err(|_| Error::Unknown)?;
                return Err(Error::Canceled);
            }
            Err(err) => {Err(err)},
        }
    }

    async fn push_queue(&self, id: u32) -> Result<(), Error> {
        let mut queue = self.queue.lock().await;
        if queue.contains(&id) {
            return Err(Error::InQueue);
        }
        queue.push(id);
        Ok(())
    }

    async fn anyway_pop_queue(&self, id: u32) {
        self.queue.lock().await.retain(|x| *x != id);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Index {
    media_mime: String,
    thumbnail_mime: String,
}

impl Storage for FileDownloader {
    async fn save<C, Fut>(&self, audio: &Audio, callback: C, downloads: RequestFiles) -> Result<(), Error>
    where
        C: Fn(u64, u64) -> Fut,
        Fut: Future<Output = ()>
    {        
        self.push_queue(audio.id).await?;
        let result = self.download_files(audio, callback, downloads).await;
        self.anyway_pop_queue(audio.id).await;
        result
    }
    
    async fn has_file(&self, audio: &Audio) -> bool {
        let audio_dir = Path::new(&self.audio_dir).join(audio.id.to_string());
        audio_dir.join("index.json").exists()
    }
    
    async fn is_in_queue(&self, id: u32) -> bool {
        self.queue.lock().await.contains(&id)
    }
    
    async fn get_files(&self, audio: &Audio) -> Result<ResponseFiles, Error> {
        let audio_dir = Path::new(&self.audio_dir).join(audio.id.to_string());
        let index = tokio::fs::read(audio_dir.join("index.json")).await.map_err(|_| Error::NotFound)?;
        let index = serde_json::from_slice::<Index>(&index).map_err(|_| Error::Unknown)?;
        let thumbnail = tokio::fs::read(audio_dir.join(format!("thumbnail.{}", mime2ext(index.thumbnail_mime.clone()).unwrap_or("bin")))).await.map_err(|_| Error::NotFound)?;
        let audio = tokio::fs::read(audio_dir.join(format!("media.{}", mime2ext(index.media_mime.clone()).unwrap_or("bin")))).await.map_err(|_| Error::NotFound)?;
        Ok(ResponseFiles {
            thumbnail,
            thumbnail_mime: index.thumbnail_mime,
            media: audio,
            media_mime: index.media_mime,
        })
    }
    
    async fn remove(&self, id: u32) -> Result<(), Error> {
        if self.queue.lock().await.contains(&id) {
            let mut rx = self.cancel_broadcast.subscribe();
            self.anyway_pop_queue(id).await;
            loop {
                let id = rx.recv().await.map_err(|_| Error::Unknown)?;
                if id == id {
                    break;
                }
            }
        }
        let audio_dir = Path::new(&self.audio_dir).join(id.to_string());
        let downloading_dir = Path::new(&self.downloading_dir).join(id.to_string());
        tokio::fs::remove_dir_all(audio_dir).await.map_err(|_| Error::Unknown)?;
        tokio::fs::remove_dir_all(downloading_dir).await.map_err(|_| Error::Unknown)?;
        Ok(())
    }
}
