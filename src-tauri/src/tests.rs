use std::{collections::HashMap, env, sync::atomic::AtomicBool};

use crate::{audio::{Audio, Source}, downloader::{ContentRetriever, DefaultContentRetriever, MemoryStorage, Storage}, ytdlp_wrapper::{self, YtDlp}};


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

#[tokio::test]
async fn storage_test() {
    let mut storage = MemoryStorage::new(DefaultContentRetriever);
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
    let audio = Audio::create("Test".to_string(), "Artist".to_string(), Source::YouTube("https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string()));
    let mut map = HashMap::new();
    map.insert("license".to_string(), "https://raw.githubusercontent.com/Laefye/FurPlayer/refs/heads/main/LICENSE".to_string());
    storage.save(&audio, d, map).await.unwrap();
    assert!(downloaded.load(std::sync::atomic::Ordering::SeqCst));
    assert!(storage.has_file(&audio, "license".to_string()));
}

#[tokio::test]
async fn downloader_cancel_test() {
    let downloader = DefaultContentRetriever;
    let d = |_, _| {
        async move {
            return false;
        }
    };
    let mut bytes = Vec::new();
    let source = downloader.download("https://raw.githubusercontent.com/Laefye/FurPlayer/refs/heads/main/LICENSE".to_string(), &mut bytes, d).await;
    assert!(source.is_err());
    assert!(matches!(source.unwrap_err(), crate::downloader::Error::Canceled));
}

#[tokio::test]
async fn audio_download_test() {
    let ytdlp = YtDlp::new(env::current_exe().unwrap().parent().unwrap().parent().unwrap().join("utils").join("yt-dlp.exe").to_str().unwrap().to_string());
    let mut storage = MemoryStorage::new(DefaultContentRetriever);
    let d = |downloaded, total| {
        async move {
            println!("Downloaded {}/{}!", downloaded, total);
            return true;
        }
    };
    let youtube_url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string();
    let metadata = ytdlp.fetch(youtube_url.clone()).await.unwrap();
    let content = metadata.get_content().unwrap();
    let map = vec![
        ("thumbnail.jpeg".to_string(), content.thumbnail),
        // ("audio.webm".to_string(), content.audio),
    ].into_iter().collect();
    let audio = Audio::create("Test".to_string(), "Artist".to_string(), Source::YouTube(youtube_url.clone()));
    storage.save(&audio, d, map).await.unwrap();

    println!("{:#?}", storage);
}
