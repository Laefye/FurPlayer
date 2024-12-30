use std::{path::Path, sync::Arc};

use event::{Event, Forwarder};
use serde::{Deserialize, Serialize};

use crate::{audio::{self, Audio, Playlist, PlaylistIOImpl, Source}, binaries, downloader::{self, FileDownloader, RequestFiles, Storage}, ytdlp::{self}};

pub struct AppState {
    ytdlp: ytdlp::YtDlp,
    playlist: audio::Playlist<PlaylistIOImpl>,
    downloader: Arc<FileDownloader>,
    forwarder: Forwarder,
}

#[derive(Debug, Clone)]
pub enum AppError {
    Downloader(downloader::Error),
    YtDlp(ytdlp::FetchError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentDTO {
    Url(String),
    Local{bytes: Vec<u8>, mime: String},
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioSourceDTO {
    YouTube(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedAudioDTO {
    id: u32,
    title: String,
    author: String,
    source: AudioSourceDTO,
}

impl From<Audio> for IndexedAudioDTO {
    fn from(value: Audio) -> Self {
        Self {
            id: value.id,
            title: value.title,
            author: value.author,
            source: match value.source {
                Source::YouTube(url) => AudioSourceDTO::YouTube(url),
            }
        }
    }
}

pub mod event;

impl ToString for AppError {
    fn to_string(&self) -> String {
        match self {
            AppError::Downloader(err) => match err {
                downloader::Error::Unknown => "Unknown error".to_string(),
                downloader::Error::Canceled => "Download canceled".to_string(),
                downloader::Error::InQueue => "Audio is already in queue".to_string(),
                downloader::Error::NotFound => "Audio not found".to_string(),
                downloader::Error::Connection => "Connection error".to_string(),
            },
            AppError::YtDlp(err) => match err {
                ytdlp::FetchError::Unknown => "Unknown error".to_string(),
                ytdlp::FetchError::NotFound => "Video not found".to_string(),
                ytdlp::FetchError::BadLink => "Bad link".to_string(),
            },
        }
    }
}

impl AppState {
    pub fn new(forwarder: Forwarder) -> Self {
        let is_portable = std::env::var("PORTABLE").is_ok();
        let app_dir = if is_portable {
            std::env::current_exe().unwrap().parent().unwrap().to_str().unwrap().to_string()
        } else {
            dirs::config_dir().unwrap().join("FurPlayer").to_str().unwrap().to_string()
        };
        let ytdlp_path;
        #[cfg(target_arch = "x86_64")]
        {
            #[cfg(target_os = "windows")]
            {
                ytdlp_path = Self::install_binary(app_dir.clone(), "yt-dlp.exe".to_string(), binaries::YTDLP);
            }
            #[cfg(target_os = "linux")]
            {
                ytdlp_path = Self::install_binary(app_dir.clone(), "yt-dlp_linux".to_string(), binaries::YTDLP);
                let mut metadata = fs::metadata(ytdlp_path.clone()).unwrap().permissions();
                metadata.set_mode(0o775);
                fs::set_permissions(ytdlp_path.clone(), metadata).unwrap();
            }
        }
        let ytdlp = ytdlp::YtDlp::new(ytdlp_path);
        let playlist_path = Path::new(&app_dir).join("playlist.json");
        let playlist = tokio::runtime::Runtime::new().unwrap().block_on(async {
            let playlist = Playlist::new(audio::PlaylistIOImpl(playlist_path.to_str().unwrap().to_string()));
            playlist.load().await.unwrap();
            playlist
        });
        let audio_dir = Path::new(&app_dir).join("audios").to_str().unwrap().to_string();
        let downloading_dir = Path::new(&app_dir).join("downloading").to_str().unwrap().to_string();
        Self {
            downloader: Arc::new(FileDownloader::new(audio_dir, downloading_dir)),
            ytdlp,
            playlist,
            forwarder,
        }
    }

    fn install_binary(app_dir: String, filename: String, binary: &[u8]) -> String {
        let path = Path::new(&app_dir).join("bin");
        let executable_path = path.join(filename);
        if !executable_path.exists() {
            std::fs::create_dir_all(&path).unwrap();
            std::fs::write(&executable_path, binary).unwrap();
        }
        executable_path.to_str().unwrap().to_string()
    }

    pub async fn add_new_audio(&self, url: String) -> Result<IndexedAudioDTO, AppError> {
        let details = self.ytdlp.fetch(url.clone()).await.map_err(AppError::YtDlp)?;
        let audio = audio::Audio::create(details.title, details.author, audio::Source::YouTube(details.url.clone()));
        self.playlist.add_audio(audio.clone()).await;
        self.save_playlist().await;
        self.download_audio(audio.clone(), details.thumbnail, details.media);
        Ok(audio.into())
    }

    pub async fn save_playlist(&self) {
        let _ = self.playlist.save().await;
    }

    pub async fn remove_audio(&self, id: u32) {
        self.playlist.remove_audio(id).await;
        self.save_playlist().await;
        let _ = self.downloader.remove(id).await;
    }

    pub fn download_audio(&self, audio: audio::Audio, thumbnail: String, media: String) {
        let downloader = self.downloader.clone();
        let forwarder = self.forwarder.clone();
        tokio::spawn(async move {
            if downloader.is_in_queue(audio.id).await {
                return;
            }
            forwarder.forward_event(Event::StartDownload { audio: audio.clone().into() });
            let result = downloader.save(&audio, |downloaded, total| {
                let forwarder = forwarder.clone();
                let audio = audio.clone();
                async move {
                    forwarder.forward_event(Event::Download { audio: audio.into(), downloaded, total, });
                }
            }, RequestFiles::new(thumbnail, media)).await;
            if result.is_ok() {
                forwarder.forward_event(Event::FinishedDownload { audio: audio.into() });
            } else if let Err(err) = result {
                forwarder.forward_event(Event::ErrorDownload { audio: audio.into(), error: AppError::Downloader(err) });
            }
        });
    }

    pub async fn get_all_audios(&self) -> Result<Vec<IndexedAudioDTO>, AppError> {
        let audios = self.playlist.get_audios().await;
        let mut indexed_audios = Vec::new();
        for audio in audios.iter()  {
            indexed_audios.push(IndexedAudioDTO {
                id: audio.id,
                author: audio.author.clone(),
                source: match &audio.source {
                    Source::YouTube(youtube) => AudioSourceDTO::YouTube(youtube.clone()),
                },
                title: audio.title.clone(),
            });
        }
        Ok(indexed_audios)
    }

    pub async fn get_thumbnail(&self, id: u32) -> Result<ContentDTO, AppError> {
        let audio = self.playlist.get_audio(id).await.ok_or(AppError::Downloader(downloader::Error::NotFound))?;
        if self.downloader.has_file(&audio).await {
            let content = self.downloader.get_files(&audio).await.map_err(AppError::Downloader)?;
            Ok(ContentDTO::Local { bytes: content.thumbnail, mime: content.thumbnail_mime })
        } else {
            match &audio.source {
                Source::YouTube(url) => {
                    let details = self.ytdlp.fetch(url.clone()).await.map_err(AppError::YtDlp)?;
                    Ok(ContentDTO::Url(details.thumbnail))
                },
            }
        }
    }

    pub async fn get_media(&self, id: u32) -> Result<ContentDTO, AppError> {
        let audio = self.playlist.get_audio(id).await.ok_or(AppError::Downloader(downloader::Error::NotFound))?;
        if self.downloader.has_file(&audio).await {
            let content = self.downloader.get_files(&audio).await.map_err(AppError::Downloader)?;
            Ok(ContentDTO::Local { bytes: content.media, mime: content.media_mime })
        } else {
            match &audio.source {
                Source::YouTube(url) => {
                    let details = self.ytdlp.fetch(url.clone()).await.map_err(AppError::YtDlp)?;
                    self.download_audio(audio, details.thumbnail, details.media.clone());
                    Ok(ContentDTO::Url(details.media))
                },
            }
        }
    }
}
