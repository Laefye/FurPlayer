use std::{path::Path, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{audio::{self, Source}, downloader::{self, FileDownloader, RequestFiles, Storage}, ytdlp_wrapper::{self, YouTubeContentSource}};

pub struct AppState {
    playlist_path: String,
    ytdlp: ytdlp_wrapper::YtDlp,
    playlist: audio::Playlist,
    downloader: Arc<FileDownloader>,
}

#[derive(Debug)]
pub enum AppError {
    Downloader(downloader::Error),
    YtDlp(ytdlp_wrapper::Error),
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
    pub fn new() -> Self {
        let is_portable = std::env::var("PORTABLE").is_ok();
        let config_dir = if is_portable {
            std::env::current_exe().unwrap().parent().unwrap().to_str().unwrap().to_string()
        } else {
            dirs::config_dir().unwrap().join("FurPlayer").to_str().unwrap().to_string()
        };
        let ytdlp_path = std::env::current_exe().unwrap().parent().unwrap().to_path_buf().join("utils").join("yt-dlp.exe").to_str().unwrap().to_string();
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
        let downloading_dir = Path::new(&config_dir).join("downloading").to_str().unwrap().to_string();
        Self {
            downloader: Arc::new(FileDownloader::new(audio_dir, downloading_dir)),
            playlist_path: playlist_path.to_str().unwrap().to_string(),
            ytdlp: ytdlp_wrapper,
            playlist,
        }
    }

    pub async fn add_new_audio(&mut self, url: String) -> Result<IndexedAudioDTO, AppError> {
        let metadata = self.ytdlp.fetch(url.clone()).await.map_err(AppError::YtDlp)?;
        let content = metadata.get_content().map_err(AppError::YtDlp)?;
        let audio = audio::Audio::create(metadata.title, metadata.channel, audio::Source::YouTube(format!("https://www.youtube.com/watch?v={}", metadata.id)));
        self.playlist.add_audio(audio.clone()).await;
        self.save_playlist().await;
        self.download_audio(audio.clone(), content.clone());
        Ok(IndexedAudioDTO {
            id: audio.id,
            title: audio.metadata.title.clone(),
            author: audio.metadata.author,
            source: match &audio.metadata.source {
                Source::YouTube(url) => AudioSourceDTO::YouTube(url.clone()),
            },
        })
    }

    pub async fn save_playlist(&self) {
        let _ = self.playlist.save(audio::PlaylistIOImpl(self.playlist_path.clone())).await;
    }

    pub async fn remove_audio(&mut self, id: u32) {
        self.playlist.remove_audio(id).await;
        self.save_playlist().await;
        let _ = self.downloader.remove(id).await;
    }

    pub fn download_audio(&self, audio: audio::Audio, content: YouTubeContentSource) {
        let downloader = self.downloader.clone();
        tokio::spawn(async move {
            let _ = downloader.save(&audio, |_, _| {async {}}, RequestFiles::new(content.thumbnail, content.media)).await;
        });
    }

    pub async fn get_all_audios(&self) -> Result<Vec<IndexedAudioDTO>, AppError> {
        let audios = self.playlist.get_audios().await;
        let mut indexed_audios = Vec::new();
        for audio in audios.iter()  {
            indexed_audios.push(IndexedAudioDTO {
                id: audio.id,
                author: audio.metadata.author.clone(),
                source: match &audio.metadata.source {
                    Source::YouTube(youtube) => AudioSourceDTO::YouTube(youtube.clone()),
                },
                title: audio.metadata.title.clone(),
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
            match &audio.metadata.source {
                Source::YouTube(url) => {
                    let metadata = self.ytdlp.fetch(url.clone()).await.map_err(AppError::YtDlp)?;
                    let content = metadata.get_content().map_err(AppError::YtDlp)?;
                    Ok(ContentDTO::Url(content.thumbnail))
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
            match &audio.metadata.source {
                Source::YouTube(url) => {
                    let metadata = self.ytdlp.fetch(url.clone()).await.map_err(AppError::YtDlp)?;
                    let content = metadata.get_content().map_err(AppError::YtDlp)?;
                    Ok(ContentDTO::Url(content.media))
                },
            }
        }
    }
}
