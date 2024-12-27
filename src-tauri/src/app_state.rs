use std::{path::Path, sync::Arc};

use serde::{Deserialize, Serialize};
use tauri::{Emitter, Runtime, WebviewWindow};
use tokio::spawn;

use crate::{audio::{self, Audio}, config::Playlist, downloader::{self, FileDownloader, RequestFiles, Storage}, storage::StorageError, ytdlp::{self, YtDlp}, ytdlp_wrapper::{self, YouTubeContentSource}};

pub type ArcEventForwarder = Arc<dyn EventForwardTrait + Send + Sync>;

pub struct AppState {
    pub ytdlp: YtDlp,
    storage: Arc<FileDownloader>,
    playlist_path: String,
    event_forwarder: ArcEventForwarder,
    
    ytdlp_wrapper: ytdlp_wrapper::YtDlp,
    playlist: audio::Playlist,
}

#[derive(Debug)]
pub enum OldError {
    YouTube(ytdlp::Error),
    NotFound,
    StorageError(StorageError),
}

#[derive(Debug)]
pub enum AppError {
    Downloader(downloader::Error),
    Old(OldError),
    YtDlp(ytdlp_wrapper::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDTO {
    pub bytes: Vec<u8>,
    pub mime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentDTO {
    Url{ thumbnail: String, media: String },
    Local{ thumbnail: FileDTO, media: FileDTO },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDTO {
    id: u32,
    content: ContentDTO,
    title: String,
    author: String,
    source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedAudioDTO {
    id: u32,
    title: String,
    author: String,
    source: String,
}

impl From<(audio::Audio, String, String)> for AudioDTO {
    fn from(value: (audio::Audio, String, String)) -> Self {
        Self {
            id: value.0.id,
            content: ContentDTO::Url{ thumbnail: value.1, media: value.2 },
            title: value.0.metadata.title,
            author: value.0.metadata.author,
            source: value.0.metadata.source.to_string(),
        }
    }
}

impl ToString for OldError {
    fn to_string(&self) -> String {
        match self {
            OldError::YouTube(err) => match err {
                ytdlp::Error::Unknown => "Unknown error".to_string(),
                ytdlp::Error::NotFound => "Video not found".to_string(),
                ytdlp::Error::PrivateVideo => "Private video".to_string(),
                ytdlp::Error::BadLink => "Bad link".to_string(),
                ytdlp::Error::NotAudio => "Not audio".to_string(),
            },
            OldError::NotFound => "Audio not found in playlist".to_string(),
            OldError::StorageError(error) => match error {
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

impl ToString for AppError {
    fn to_string(&self) -> String {
        match self {
            AppError::Downloader(err) => match err {
                downloader::Error::Unknown => "Unknown error".to_string(),
                downloader::Error::Canceled => "Download canceled".to_string(),
                downloader::Error::InQueue => "Audio is already in queue".to_string(),
                downloader::Error::NotFound => "Audio not found".to_string(),
            },
            AppError::Old(err) => err.to_string(),
            AppError::YtDlp(err) => match err {
                ytdlp_wrapper::Error::Unknown => "Unknown error".to_string(),
                ytdlp_wrapper::Error::NotFound => "Video not found".to_string(),
                ytdlp_wrapper::Error::PrivateVideo => "Private video".to_string(),
                ytdlp_wrapper::Error::BadLink => "Bad link".to_string(),
                ytdlp_wrapper::Error::NotAudio => "Not audio".to_string(),
            },
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
        match audio::Playlist::load(audio::PlaylistIOImpl(playlist_path.to_str().unwrap().to_string())) {
            Ok(loaded) => {
                playlist = loaded;
            },
            Err(_) => {
                playlist = audio::Playlist::new();
            },
        }
        let audio_dir = Path::new(&config_dir).join("audios").to_str().unwrap().to_string();
        Self {
            ytdlp,
            storage: Arc::new(FileDownloader::new(audio_dir)),
            playlist_path: playlist_path.to_str().unwrap().to_string(),
            event_forwarder,
            ytdlp_wrapper,
            playlist,
        }
    }

    pub async fn add_new_audio(&mut self, url: String) -> Result<AudioDTO, AppError> {
        let metadata = self.ytdlp_wrapper.fetch(url.clone()).await.map_err(AppError::YtDlp)?;
        let content = metadata.get_content().map_err(AppError::YtDlp)?;
        let audio = audio::Audio::create(metadata.title, metadata.channel, audio::Source::YouTube(url));
        self.playlist.add_audio(audio.clone()).await;
        self.save_playlist().await;
        self.download_audio(audio.clone(), content.clone());
        Ok((audio, content.thumbnail, content.audio).into())
    }

    pub async fn save_playlist(&self) {
        let _ = self.playlist.save(audio::PlaylistIOImpl(self.playlist_path.clone())).await;
    }

    pub async fn remove_audio(&mut self, id: u32) {
        self.playlist.remove_audio(id).await;
    }

    pub fn download_audio(&self, audio: audio::Audio, content: YouTubeContentSource) {
        let downloader = self.storage.clone();
        tokio::spawn(async move {
            downloader.save(&audio, |_, _| {async {}}, RequestFiles::new(content.thumbnail, content.audio)).await.unwrap();
        });
    }

    pub async fn get_audio(&self, id: u32) -> Option<AudioDTO> {
        let audio = self.playlist.get_audio(id).await;
        match audio {
            Some(audio) => {
                if self.storage.has_file(&audio).await {
                    let files = self.storage.get_files(&audio).await.unwrap();
                    Some(
                        AudioDTO { id: audio.id,
                            content: ContentDTO::Local {
                                thumbnail: FileDTO { bytes: files.thumbnail, mime: "image/jpeg".to_string() },
                                media: FileDTO { bytes: files.audio, mime: "audio/webm".to_string() },
                            },
                            title: audio.metadata.title,
                            author: audio.metadata.author,
                            source: audio.metadata.source.to_string(),
                        }
                    )
                } else {
                    let cloned = audio.clone();
                    match audio.metadata.source {
                        audio::Source::YouTube(url) => {
                            let metadata = self.ytdlp_wrapper.fetch(url.clone()).await.ok()?;
                            let content = metadata.get_content().ok()?;
                            Some((cloned, content.thumbnail, content.audio).into())
                        },
                    }
                }
            },
            None => None,
        }
    }

    pub async fn get_all_audios(&self) -> Vec<IndexedAudioDTO> {
        self.playlist.get_audios()
            .await
            .into_iter()
            .map(|audio| IndexedAudioDTO { id: audio.id, title: audio.metadata.title.clone(), author: audio.metadata.author.clone(), source: audio.metadata.source.to_string() })
            .collect()
    }
}

pub struct EventForwarder<R: Runtime> {
    window: WebviewWindow<R>,
}

pub enum DownloadStatus {
    Started,
    Finished,
    Process(f64),
    Error(OldError),
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
