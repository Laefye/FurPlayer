use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use super::{Audio, Source};

#[derive(Debug)]
pub struct Playlist<T: PlaylistIO<T>> {
    audios: Mutex<Vec<Audio>>,
    io: T,
}

#[derive(Debug, Clone)]
pub enum LoadError {
    NotFound,
    Unknown,
}

pub trait PlaylistIO<T: PlaylistIO<T>> {
    async fn load(&self, playlist: &Playlist<T>) -> Result<(), LoadError>;
    async fn save(&self, playlist: &Playlist<T>) -> Result<(), LoadError>;
}

impl<T: PlaylistIO<T>> Playlist<T> {
    pub fn new(io: T) -> Self {
        Self {
            audios: Mutex::new(Vec::new()),
            io,
        }
    }

    pub async fn load(&self) -> Result<(), LoadError> {
        println!("Playlist loaded");
        self.io.load(self).await?;
        Ok(())
    }

    pub async fn save(&self) -> Result<(), LoadError> {
        println!("Playlist saved");
        self.io.save(self).await
    }

    pub async fn add_audio(&self, audio: Audio) {
        self.audios.lock().await.push(audio);
    }

    pub async fn get_audio(&self, id: u32) -> Option<Audio> {
        self.audios.lock().await.iter().find(|audio| audio.id == id).cloned()
    }

    pub async fn remove_audio(&self, id: u32) {
        let mut audios = self.audios.lock().await;
        audios.retain(|audio| audio.id != id);
    }

    pub async fn get_audios(&self) -> Vec<Audio> {
        self.audios.lock().await.clone()
    }

    pub async fn set_audios(&self, audios: Vec<Audio>) {
        *self.audios.lock().await = audios;
    }
}

#[derive(Debug)]
pub struct PlaylistIOImpl(pub String);


#[derive(Debug, Serialize, Deserialize)]
enum LocalSource {
    YouTube(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct AudioDTO {
    id: u32,
    title: String,
    author: String,
    source: LocalSource,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlaylistDTO {
    audios: Vec<AudioDTO>,
}

impl PlaylistIO<PlaylistIOImpl> for PlaylistIOImpl {
    async fn load(&self, playlist: &Playlist<PlaylistIOImpl>) -> Result<(), LoadError> {
        let serialized = std::fs::read_to_string(self.0.clone()).map_err(|_| LoadError::NotFound)?;
        let playlist_dto: PlaylistDTO = serde_json::from_str(&serialized).map_err(|_| LoadError::Unknown)?;
        let audios = playlist_dto.audios.iter().map(|audio| Audio {
            id: audio.id,
            metadata: super::Metadata {
                title: audio.title.clone(),
                source: match &audio.source {
                    LocalSource::YouTube(url) => Source::YouTube(url.clone()),
                },
                author: audio.author.clone(),
            },
        }).collect();
        playlist.set_audios(audios).await;
        Ok(())
    }

    async fn save(&self, playlist: &Playlist<PlaylistIOImpl>) -> Result<(), LoadError> {
        let audios = playlist.get_audios().await;
        let playlist_dto = PlaylistDTO {
            audios: audios.iter().map(|audio| AudioDTO {
                id: audio.id,
                title: audio.metadata.title.clone(),
                author: audio.metadata.author.clone(),
                source: match &audio.metadata.source {
                    Source::YouTube(url) => LocalSource::YouTube(url.clone()),
                },
            }).collect(),
        };
        let serialized = serde_json::to_string(&playlist_dto).map_err(|_| LoadError::Unknown)?;
        std::fs::write(self.0.clone(), serialized).map_err(|_| LoadError::Unknown)?;
        Ok(())
    }
}
