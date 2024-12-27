#[derive(Debug, Clone)]
pub enum Source {
    YouTube(String),
}

#[derive(Debug, Clone)]
pub struct Metadata {
    title: String,
    author: String,
    source: Source,
}

#[derive(Debug, Clone)]
pub struct Audio {
    id: u32,
    metadata: Metadata,
}

impl Audio {
    pub fn new(id: u32, metadata: Metadata) -> Self {
        Self {
            id,
            metadata,
        }
    }

    pub fn create(title: String, author: String, source: Source) -> Self {
        Self {
            id: rand::random(),
            metadata: Metadata {
                title,
                author,
                source,
            },
        }
    }
}
