
use std::sync::Arc;

use app_state::{AppState, EventForwarder};
use config::{Audio, Metadata};
use tauri::{Manager, State};
use tokio::sync::Mutex;

mod ytdlp;
mod config;
mod storage;
mod app_state;
mod audio;
mod ytdlp_wrapper;
mod downloader;

#[tauri::command]
async fn add_new_audio(state: State<'_, Mutex<AppState>>, url: String) -> Result<Audio, String> {
    let mut state = state.lock().await;
    let audio = state.add_new_audio(url).await;
    match audio {
        Ok(audio) => Ok(audio),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
async fn load_audio(state: State<'_, Mutex<AppState>>, id: u32) -> Result<Audio, String> {
    let state = state.lock().await;
    let audio = state.get_audio(id).await;
    match audio {
        Ok(audio) => Ok(audio),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
async fn get_playlist_metadata(state: State<'_, Mutex<AppState>>) -> Result<Vec<Metadata>, String> {
    let state = state.lock().await;
    Ok(state.playlist.audios.clone())
}

#[tauri::command]
async fn remove_audio(state: State<'_, Mutex<AppState>>, id: u32) -> Result<(), ()> {
    let mut state = state.lock().await;
    state.remove_audio(id).await;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            app.manage(Mutex::new(AppState::new(Arc::new(EventForwarder::new(window)))));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![add_new_audio, get_playlist_metadata, load_audio, remove_audio])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use std::{env, sync::atomic::{AtomicBool, AtomicI64}};

    use crate::{downloader::{DefaultContentRetriever, ContentRetriever}, ytdlp_wrapper::{self, YtDlp}};


    #[tokio::test]
    async fn ytdlp_fetch() {
        let ytdlp = YtDlp::new(env::current_exe().unwrap().parent().unwrap().parent().unwrap().join("utils").join("yt-dlp.exe").to_str().unwrap().to_string());
        {
            let metadata = ytdlp.fetch("https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string()).await;
            assert!(metadata.is_ok());
            let metadata = metadata.unwrap();
            assert_eq!(metadata.id, "dQw4w9WgXcQ");
            let source = metadata.get_content();
            assert!(source.is_ok());
        }
        {
            let metadata = ytdlp.fetch("https://www.youtube.com/watch?v=123".to_string()).await;
            assert!(metadata.is_err());
            assert!(matches!(metadata.unwrap_err(), ytdlp_wrapper::Error::NotFound));
        }
        {
            let metadata = ytdlp.fetch("https://www.some.com/watch?v=123".to_string()).await;
            assert!(metadata.is_err());
            assert!(matches!(metadata.unwrap_err(), ytdlp_wrapper::Error::BadLink));
        }
    }

    #[tokio::test]
    async fn downloader_test() {
        let downloader = DefaultContentRetriever;
        let downloaded = AtomicBool::new(false);
        let d = |current, size| {
            let downloaded = &downloaded;
            async move {
                if current == size {
                    downloaded.store(true, std::sync::atomic::Ordering::SeqCst);
                }
                return true;
            }
        };
        let mut bytes = Vec::new();
        let source = downloader.download("https://raw.githubusercontent.com/Laefye/FurPlayer/refs/heads/main/LICENSE".to_string(), &mut bytes, d).await;
        assert!(source.is_ok());
        let source = source.unwrap();
        assert!(source.mime.contains("text/plain"));
        assert_eq!(downloaded.load(std::sync::atomic::Ordering::SeqCst), true);
    }
}
