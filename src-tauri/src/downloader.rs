use std::{collections::HashMap, future::Future, io::Write};

#[derive(Debug)]
pub struct Content {
    pub mime: String,
}

#[derive(Debug)]
pub enum Error {
    Unknown,
    Canceled,
}

pub trait ContentRetriever {
    async fn download<F, Fut>(&self, url: String, writer: &mut impl Write, callback: F) -> Result<Content, Error>
    where
        F: Fn(u64, u64) -> Fut,
        Fut: Future<Output = bool>;
}

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
    async fn save(&self, map: HashMap<String, String>) -> Result<(), Error>;
}
