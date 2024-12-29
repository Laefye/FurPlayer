
use std::sync::Arc;

use app_state::{event::WebviewForwarder, AppState, ContentDTO, IndexedAudioDTO};
use tauri::{Manager, State};

mod app_state;
mod audio;
mod ytdlp_wrapper;
mod downloader;


#[tauri::command]
async fn add_new_audio(state: State<'_, Arc<AppState>>, url: String) -> Result<IndexedAudioDTO, String> {
    let audio = state.add_new_audio(url).await;
    match audio {
        Ok(audio) => Ok(audio),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
async fn get_playlist(state: State<'_, Arc<AppState>>) -> Result<Vec<IndexedAudioDTO>, String> {
    state.get_all_audios().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_audio(state: State<'_, Arc<AppState>>, id: u32) -> Result<(), ()> {
    let cloned = state.inner().clone();
    tokio::spawn(async move {
        cloned.remove_audio(id).await
    });
    Ok(())
}

#[tauri::command]
async fn get_thumbnail(state: State<'_, Arc<AppState>>, id: u32) -> Result<ContentDTO, String> {
    state.get_thumbnail(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_media(state: State<'_, Arc<AppState>>, id: u32) -> Result<ContentDTO, String> {
    state.get_media(id).await.map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let webview_forwarder = WebviewForwarder::new(app.get_webview_window("main").unwrap());
            app.manage(Arc::new(AppState::new(Arc::new(webview_forwarder))));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![add_new_audio, get_playlist, remove_audio, get_media, get_thumbnail])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests;
