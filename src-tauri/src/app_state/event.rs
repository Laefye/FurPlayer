use std::sync::Arc;

use serde::Serialize;
use tauri::{Emitter, Runtime, WebviewWindow};

use super::{AppError, IndexedAudioDTO};

#[derive(Debug, Clone)]
pub enum Event {
    StartDownload {
        audio: IndexedAudioDTO,
    },
    FinishedDownload {
        audio: IndexedAudioDTO,
    },
    ErrorDownload {
        audio: IndexedAudioDTO,
        error: AppError,
    },
    Download {
        audio: IndexedAudioDTO,
        downloaded: u64,
        total: u64,
    }
}

pub trait ForwardEvents {
    fn forward_event(&self, event: Event);
}

pub type Forwarder = Arc<dyn ForwardEvents + Send + Sync>;

pub struct WebviewForwarder<R: Runtime> {
    webview: WebviewWindow<R>
}

#[derive(Debug, Clone, Serialize)]
enum WebviewEvent {
    StartDownload {
        audio: IndexedAudioDTO,
    },
    FinishedDownload {
        audio: IndexedAudioDTO,
    },
    ErrorDownload {
        audio: IndexedAudioDTO,
        error: String,
    },
    Download {
        audio: IndexedAudioDTO,
        downloaded: u64,
        total: u64,
    }
}

impl From<Event> for WebviewEvent {
    fn from(value: Event) -> Self {
        match value {
            Event::StartDownload { audio } => WebviewEvent::StartDownload { audio },
            Event::FinishedDownload { audio } => Self::FinishedDownload { audio },
            Event::ErrorDownload { audio, error } => Self::ErrorDownload { audio, error: error.to_string() },
            Event::Download { audio, downloaded, total } => Self::Download { audio, downloaded, total },
        }
    }
}

impl<R: Runtime> ForwardEvents for WebviewForwarder<R> {
    fn forward_event(&self, event: Event) {
        match &event {
            Event::StartDownload {audio: _} => {
                self.webview.emit("download", WebviewEvent::from(event)).unwrap();
            },
            Event::FinishedDownload { audio: _ } => {
                self.webview.emit("download", WebviewEvent::from(event)).unwrap();
            },
            Event::ErrorDownload { audio: _, error: _} => {
                self.webview.emit("download", WebviewEvent::from(event)).unwrap();
            },
            Event::Download { audio: _, downloaded: _, total: _ } => {
                self.webview.emit("download", WebviewEvent::from(event)).unwrap();
            },
        }
    }
}

impl<R: Runtime> WebviewForwarder<R> {
    pub fn new(webview: WebviewWindow<R>) -> Self {
        Self {
            webview,
        }
    }
}
