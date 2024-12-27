use std::{env, sync::atomic::AtomicBool};

use crate::{downloader::{ContentRetriever, DefaultContentRetriever}, ytdlp_wrapper::{self, YtDlp}};


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
