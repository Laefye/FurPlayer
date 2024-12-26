use std::{path::Path, sync::Arc};

use tokio::{spawn, sync::Mutex};

use crate::{config::{Audio, Data, Metadata, Playlist, UrledData}, storage::Storage, ytdlp::{self, YtDlp}};

pub struct AppState {
    pub config_dir: String,
    pub ytdlp: YtDlp,
    pub playlist: Playlist,
    pub storage: Storage,
    playlist_path: String,
}

#[derive(Debug)]
pub enum Error {
    YouTube(ytdlp::Error),
    NotFound,
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Error::YouTube(err) => match err {
                ytdlp::Error::Unknown => "Unknown error".to_string(),
                ytdlp::Error::NotFound => "Video not found".to_string(),
                ytdlp::Error::PrivateVideo => "Private video".to_string(),
                ytdlp::Error::BadLink => "Bad link".to_string(),
                ytdlp::Error::NotAudio => "Not audio".to_string(),
            },
            Error::NotFound => "Audio not found in playlist".to_string(),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir().unwrap().join("FurPlayer").to_str().unwrap().to_string();
        let ytdlp_path = std::env::current_exe().unwrap().parent().unwrap().to_path_buf().join("utils").join("yt-dlp.exe").to_str().unwrap().to_string();
        let ytdlp = YtDlp::new(ytdlp_path);
        let playlist_path = Path::new(&config_dir).join("playlist.json");
        let playlist;
        if playlist_path.exists() {
            playlist = Playlist::load(playlist_path.to_str().unwrap().to_string());
        } else {
            playlist = Playlist::new();
        }
        let audio_dir = Path::new(&config_dir).join("audios").to_str().unwrap().to_string();
        let storage = Storage::new(audio_dir);
        Self {
            config_dir,
            ytdlp,
            playlist,
            storage,
            playlist_path: playlist_path.to_str().unwrap().to_string(),
        }
    }

    pub async fn add_new_audio(&mut self, url: String) -> Result<Audio, Error> {
        let ytdlp_metadata = self.ytdlp.get_metadata(url.clone()).await.map_err(Error::YouTube)?;
        let ytdlp_urled = ytdlp_metadata.get_urled_data().map_err(Error::YouTube)?;
        let id = rand::random::<u32>();
        let metadata = ytdlp_metadata.create_metadata(id);
        println!("{:?}", metadata);
        self.download_audio(ytdlp_urled.clone(), metadata.clone());
        println!("{:?}", metadata);
        self.playlist.add_audio(metadata.clone());
        self.playlist.save(self.playlist_path.clone()).await;
        Ok(Audio {
            metadata,
            data: Data::Urled(ytdlp_urled),
        })
    }

    pub fn download_audio(&self, urled: UrledData, metadata: Metadata) {
        let storage = self.storage.clone();
        spawn(async move {
            storage.download(urled, metadata.clone()).await;
        });
    }

    pub async fn get_audio(&self, id: u32) -> Result<Audio, Error> {
        if self.storage.has_audio(id).await {
            let loaded_data = self.storage.load(id).await;
            let metadata = self.playlist.get_audio(id).ok_or(Error::NotFound)?;
            Ok(Audio {
                metadata: metadata.clone(),
                data: Data::Loaded(loaded_data),
            })
        } else {
            let metadata = self.playlist.get_audio(id).ok_or(Error::NotFound)?;
            match &metadata.platform {
                crate::config::Platform::YouTube(url) => {
                    let ytdlp_metadata = self.ytdlp.get_metadata(url.clone()).await.map_err(Error::YouTube)?;
                    let ytdlp_urled = ytdlp_metadata.get_urled_data().map_err(Error::YouTube)?;
                    if !self.storage.is_in_queue(id).await {
                        println!("Downloading audio because it's not in queue");
                        self.download_audio(ytdlp_urled.clone(), metadata.clone());
                    }
                    Ok(Audio {
                        metadata: metadata.clone(),
                        data: Data::Urled(ytdlp_urled),
                    })
                },
            }
        }
    }

    pub async fn get_audio_metadata(&self, id: u32) -> Option<Metadata> {
        self.playlist.get_audio(id).cloned()
    }
}
