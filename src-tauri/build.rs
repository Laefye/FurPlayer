use std::fs;

use reqwest::Method;

fn download_ytdlp() {
    let url;
    let filename;
    #[cfg(target_arch = "x86_64")]
    {
        #[cfg(target_os = "windows")]
        {
            url = "https://github.com/yt-dlp/yt-dlp/releases/download/2024.12.23/yt-dlp.exe";
            filename = "yt-dlp.exe";
        }
        #[cfg(target_os = "linux")]
        {
            url = "https://github.com/yt-dlp/yt-dlp/releases/download/2024.12.23/yt-dlp_linux";
            filename = "yt-dlp_linux";
        }
    }
    let real_path = format!(".ytdlp/{}", filename);
    if !fs::exists(&real_path).unwrap() {
        let result = reqwest::blocking::Client::new()
            .request(Method::GET, url)
            .header("User-Agent", "reqwest")
            .send()
            .unwrap()
            .bytes()
            .unwrap();
        std::fs::create_dir_all(".ytdlp").unwrap();
        std::fs::write(&real_path, &result).unwrap();
    }
    println!("cargo:rustc-env=YTDLP_BIN={}", std::fs::canonicalize(real_path).unwrap().display());
}

fn main() {
    download_ytdlp();
    tauri_build::build()
}
