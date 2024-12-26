use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Platform {
    YouTube(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub id: u32,
    pub title: String,
    pub author: String,
    pub platform: Platform,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub bytes: Vec<u8>,
    pub mime: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadedData {
    pub thumbnail: File,
    pub audio: File,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UrledData {
    pub thumbnail: String,
    pub audio: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Data {
    Loaded(LoadedData),
    Urled(UrledData),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Audio {
    pub metadata: Metadata,
    pub data: Data,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Playlist {
    pub audios: Vec<Metadata>,
}

impl Playlist {
    pub fn new() -> Self {
        Self { audios: vec![] }
    }

    pub fn add_audio(&mut self, audio: Metadata) {
        self.audios.push(audio);
    }

    pub async fn save(&self, path: String) {
        let file = std::fs::File::create(path).unwrap();
        serde_json::to_writer(file, self).unwrap();
    }

    pub fn load(path: String) -> Self {
        // Где ошибки :3 ?
        let file = std::fs::File::open(path).unwrap();
        serde_json::from_reader(file).unwrap()
    }

    pub fn get_audio(&self, id: u32) -> Option<&Metadata> {
        self.audios.iter().find(|audio| audio.id == id)
    }
}
