
use app_state::{AppState, ContentDTO, IndexedAudioDTO};
use tauri::{Manager, State};
use tokio::sync::Mutex;

mod app_state;
mod audio;
mod ytdlp_wrapper;
mod downloader;


#[tauri::command]
async fn add_new_audio(state: State<'_, Mutex<AppState>>, url: String) -> Result<IndexedAudioDTO, String> {
    let mut state = state.lock().await;
    let audio = state.add_new_audio(url).await;
    match audio {
        Ok(audio) => Ok(audio),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
async fn get_playlist_metadata(state: State<'_, Mutex<AppState>>) -> Result<Vec<IndexedAudioDTO>, String> {
    let state = state.lock().await;
    state.get_all_audios().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_audio(state: State<'_, Mutex<AppState>>, id: u32) -> Result<(), ()> {
    let mut state = state.lock().await;
    state.remove_audio(id).await;
    Ok(())
}

#[tauri::command]
async fn get_thumbnail(state: State<'_, Mutex<AppState>>, id: u32) -> Result<ContentDTO, String> {
    let state = state.lock().await;
    state.get_thumbnail(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_media(state: State<'_, Mutex<AppState>>, id: u32) -> Result<ContentDTO, String> {
    let state = state.lock().await;
    state.get_media(id).await.map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(AppState::new()));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![add_new_audio, get_playlist_metadata, remove_audio, get_media, get_thumbnail])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests;
