use std::{path::Path, sync::Arc};

use serde::{Deserialize, Serialize};
use tauri::{Emitter, Runtime, WebviewWindow};
use tokio::spawn;

use crate::{config::{Audio, Data, Metadata, Playlist, UrledData}, storage::{Storage, StorageError}, ytdlp::{self, YtDlp}, ytdlp_wrapper};

pub type ArcEventForwarder = Arc<dyn EventForwardTrait + Send + Sync>;

pub struct AppState {
    pub ytdlp: YtDlp,
    pub playlist: Playlist,
    pub storage: Storage,
    playlist_path: String,
    event_forwarder: ArcEventForwarder,
    ytdlp_wrapper: ytdlp_wrapper::YtDlp,
}

#[derive(Debug)]
pub enum Error {
    YouTube(ytdlp::Error),
    NotFound,
    StorageError(StorageError),
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
            Error::StorageError(error) => match error {
                StorageError::Internet(error) => {
                    if error.is_connect() || error.is_timeout() {
                        "Problems with connection".to_string()
                    } else {
                        "Unknown donwload error".to_string()
                    }
                },
            }
        }
    }
}

impl AppState {
    pub fn new(event_forwarder: ArcEventForwarder) -> Self {
        let is_portable = std::env::var("PORTABLE").is_ok();
        let config_dir = if is_portable {
            std::env::current_exe().unwrap().parent().unwrap().to_str().unwrap().to_string()
        } else {
            dirs::config_dir().unwrap().join("FurPlayer").to_str().unwrap().to_string()
        };
        let ytdlp_path = std::env::current_exe().unwrap().parent().unwrap().to_path_buf().join("utils").join("yt-dlp.exe").to_str().unwrap().to_string();
        let ytdlp = YtDlp::new(ytdlp_path.clone());
        let ytdlp_wrapper = ytdlp_wrapper::YtDlp::new(ytdlp_path);
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
            ytdlp,
            playlist,
            storage,
            playlist_path: playlist_path.to_str().unwrap().to_string(),
            event_forwarder,
            ytdlp_wrapper,
        }
    }

    pub async fn add_new_audio(&mut self, url: String) -> Result<Audio, Error> {
        let ytdlp_metadata = self.ytdlp.get_metadata(url.clone()).await.map_err(Error::YouTube)?;
        let ytdlp_urled = ytdlp_metadata.get_urled_data().map_err(Error::YouTube)?;
        let id = rand::random::<u32>();
        let metadata = ytdlp_metadata.create_metadata(id);
        self.download_audio(ytdlp_urled.clone(), metadata.clone());
        self.playlist.add_audio(metadata.clone());
        self.playlist.save(self.playlist_path.clone()).await;
        Ok(Audio {
            metadata,
            data: Data::Urled(ytdlp_urled),
        })
    }

    pub async fn remove_audio(&mut self, id: u32) {
        self.playlist.remove_audio(id);
        self.playlist.save(self.playlist_path.clone()).await;
    }

    pub fn download_audio(&self, urled: UrledData, metadata: Metadata) {
        let storage = self.storage.clone();
        let event_forwarder = self.event_forwarder.clone();
        spawn(async move {
            event_forwarder.on_status_download(metadata.id, DownloadStatus::Started);
            let result = storage.download(urled, metadata.clone(), |progress| {
                event_forwarder.on_status_download(metadata.id, DownloadStatus::Process(progress));
            }).await;
            match result {
                Ok(_) => event_forwarder.on_status_download(metadata.id, DownloadStatus::Finished),
                Err(err) => event_forwarder.on_status_download(metadata.id, DownloadStatus::Error(Error::StorageError(err))),
            }
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
}

pub struct EventForwarder<R: Runtime> {
    window: WebviewWindow<R>,
}

pub enum DownloadStatus {
    Started,
    Finished,
    Process(f64),
    Error(Error),
}

pub trait EventForwardTrait {
    fn on_status_download(&self, id: u32, status: DownloadStatus);
}

impl<R: Runtime> EventForwarder<R> {
    pub fn new(window: WebviewWindow<R>) -> Self {
        Self {
            window,
        }
    }
}

impl<R: Runtime> EventForwardTrait for EventForwarder<R> {
    fn on_status_download(&self, id: u32, status: DownloadStatus) {
        #[derive(Debug, Serialize, Deserialize, Clone)]
        enum Payload {
            Started(u32),
            Finished(u32),
            Process((u32, f64)),
            Error((u32, String)),
        }
        let _ = self.window.emit(
            "status_download",
            match status {
                DownloadStatus::Started => Payload::Started(id),
                DownloadStatus::Finished => Payload::Finished(id),
                DownloadStatus::Error(error) => Payload::Error((id, error.to_string())),
                DownloadStatus::Process(progress) => Payload::Process((id, progress)),
            }
        );
        
    }
}
