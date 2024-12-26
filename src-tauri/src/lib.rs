use std::{fs::create_dir_all, path::Path, sync::{Arc, RwLock}};

use app_state::AppState;
use config::{Audio, LoadedData, Metadata, Platform, Playlist, UrledData};
use storage::Storage;
use tauri::{Manager, State};
use tokio::{spawn, sync::Mutex};
use ytdlp::{YouTubeLoadedMusic, YtDlp};

mod ytdlp;
mod config;
mod storage;
mod app_state;

#[tauri::command]
async fn add_playlist(state: State<'_, Mutex<AppState>>, url: String) -> Result<Audio, String> {
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(AppState::new()));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![add_playlist, get_playlist_metadata, load_audio])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
