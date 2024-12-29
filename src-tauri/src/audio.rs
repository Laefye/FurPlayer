#[derive(Debug, Clone)]
pub enum Source {
    YouTube(String),
}

impl ToString for Source {
    fn to_string(&self) -> String {
        match self {
            Source::YouTube(_) => "YouTube".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Audio {
    pub id: u32,
    pub title: String,
    pub author: String,
    pub source: Source,
}

mod playlist;

pub use playlist::Playlist;
pub use playlist::PlaylistIOImpl;

impl Audio {
    pub fn create(title: String, author: String, source: Source) -> Self {
        Self {
            id: rand::random(),
            title,
            author,
            source,
        }
    }
}
