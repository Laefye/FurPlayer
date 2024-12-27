use serde::{Deserialize, Serialize};
use tokio::{join, runtime::{Handle, Runtime}, sync::Mutex, task::spawn_blocking};

use super::{Audio, Source};

#[derive(Debug)]
pub struct Playlist {
    audios: Mutex<Vec<Audio>>,
}

pub enum LoadError {
    NotFound,
    Unknown,
}

pub trait PlaylistIO {
    fn load(&self) -> Result<Playlist, LoadError>;
    async fn save(&self, playlist: &Playlist) -> Result<(), LoadError>;
}

impl Playlist {
    pub fn new() -> Self {
        Self {
            audios: Mutex::new(Vec::new()),
        }
    }

    pub fn load<T: PlaylistIO>(io: T) -> Result<Playlist, LoadError> {
        io.load()
    }

    pub async fn save<T: PlaylistIO>(&self, io: T) -> Result<(), LoadError> {
        println!("Playlist saved");
        io.save(self).await
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

impl PlaylistIO for PlaylistIOImpl {
    fn load(&self) -> Result<Playlist, LoadError> {
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
        Ok(Playlist {
            audios: Mutex::new(audios),
        })
    }

    async fn save(&self, playlist: &Playlist) -> Result<(), LoadError> {
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
