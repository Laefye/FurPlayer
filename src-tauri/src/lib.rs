
use std::sync::Arc;

use app_state::{AppState, AudioDTO, EventForwarder, IndexedAudioDTO};
use config::{Audio, Metadata};
use serde::{Deserialize, Serialize};
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
async fn add_new_audio(state: State<'_, Mutex<AppState>>, url: String) -> Result<AudioDTO, String> {
    let mut state = state.lock().await;
    let audio = state.add_new_audio(url).await;
    match audio {
        Ok(audio) => Ok(audio),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
async fn load_audio(state: State<'_, Mutex<AppState>>, id: u32) -> Result<Option<AudioDTO>, String> {
    let state = state.lock().await;
    let audio = state.get_audio(id).await;
    Ok(audio)
}

#[tauri::command]
async fn get_playlist_metadata(state: State<'_, Mutex<AppState>>) -> Result<Vec<IndexedAudioDTO>, String> {
    let state = state.lock().await;
    Ok(state.get_all_audios().await)
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
mod tests;
