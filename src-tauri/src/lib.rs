use std::{fs::create_dir_all, path::Path};

use config::{Audio, LoadedData, Metadata, Platform, Playlist, UrledData};
use tauri::{async_runtime::spawn, Manager, State};
use tokio::sync::Mutex;
use ytdlp::{YouTubeLoadedMusic, YtDlp};

mod ytdlp;
mod config;

struct AppState {
    pub config_dir: String,
    pub ytdlp: YtDlp,
    pub playlist: Playlist,
}

async fn save_audio(metadata: Metadata, loaded: LoadedData, config_dir: String) {
    let thumbnail_path = Path::new(&config_dir).join("thumbnails").join(&metadata.id.to_string());
    let audio_path = Path::new(&config_dir).join("audios").join(&metadata.id.to_string());
    create_dir_all(thumbnail_path.clone()).unwrap();
    create_dir_all(audio_path.clone()).unwrap();

    let thumbnail_file = thumbnail_path.join("thumbnail.jpg");
    let audio_file = audio_path.join("audio.webm");
    
    tokio::fs::write(thumbnail_file, &loaded.thumbnail.bytes).await.unwrap();
    tokio::fs::write(audio_file, &loaded.audio.bytes).await.unwrap();
}

#[tauri::command]
async fn add_playlist(state: State<'_, Mutex<AppState>>, url: String) -> Result<Audio, String> {
    let mut state = state.lock().await;
    let id = rand::random::<u32>();
    let yt_metadata = state.ytdlp.get_metadata(url).await.map_err(|_| "Failed to get metadata".to_string())?;
    let urled = yt_metadata.get_urled_data().map_err(|_| "Failed to get urled data".to_string())?;
    let audio = Audio {
        metadata: yt_metadata.create_metadata(id),
        data: config::Data::Urled(urled),
    };
    state.playlist.add_audio(audio.metadata.clone());
    let playlist_path = Path::new(&state.config_dir).join("playlist.json");
    state.playlist.save(playlist_path.to_str().unwrap().to_string()).await;
    Ok(audio)
}

#[tauri::command]
async fn load_audio(state: State<'_, Mutex<AppState>>, id: u32) -> Result<Audio, String> {
    let state = state.lock().await;
    let metadata = state.playlist.get_audio(id).ok_or("Audio not found")?;
    match &metadata.platform {
        Platform::YouTube(url) => {
            let ytdlp_metadata = state.ytdlp.get_metadata(url.clone()).await.map_err(|_| "Failed to fetch loaded data".to_string())?;
            let urled = ytdlp_metadata.get_urled_data().map_err(|_| "Failed to get urled data".to_string())?;
            Ok(Audio {
                metadata: metadata.clone(),
                data: config::Data::Urled(urled),
            })
        },
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
            let config_dir = dirs::config_dir().unwrap().join("FurPlayer").to_str().unwrap().to_string();
            create_dir_all(config_dir.clone()).unwrap();
            let playlist_path = Path::new(&config_dir).join("playlist.json");
            let playlist;
            if playlist_path.exists() {
                playlist = Playlist::load(playlist_path.to_str().unwrap().to_string());
            } else {
                playlist = Playlist::new();
            }
            let ytdlp_path = std::env::current_exe().unwrap().parent().unwrap().to_path_buf().join("utils").join("yt-dlp.exe");
            app.manage(Mutex::new(AppState {
                config_dir,
                ytdlp: YtDlp::new(ytdlp_path.to_str().unwrap().to_string()),
                playlist: playlist,
            }));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![add_playlist, get_playlist_metadata, load_audio])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
