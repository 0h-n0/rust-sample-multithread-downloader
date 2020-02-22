extern crate curl;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::{channel, sync_channel};
use std::sync::Arc;
use std::thread;

pub fn download(url: &str, filename: &PathBuf) -> std::io::Result<()> {
    println!("Now downloading data from {:?}", url);
    if filename.exists() {
        println!("{:?} already exsits.", filename);
        return Ok(());
    }
    let f = std::fs::File::create(&filename)?;
    let mut writer = std::io::BufWriter::new(f);
    let mut easy = curl::easy::Easy::new();
    easy.url(url)?;
    easy.write_function(move |data| Ok(writer.write(data).unwrap()))?;
    easy.perform()?;
    let response_code = easy.response_code()?;
    if response_code == 200 {
        println!("Download completed....");
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("download error response_code = {}", response_code),
        ))
    }
}

type URLS = Vec<&'static str>;
type FILES = Vec<PathBuf>;

pub fn simple_multithread_downloader(urls: URLS, filenames: FILES) {
    let urls = Arc::new(urls);
    let filenames = Arc::new(filenames);
    let n = urls.len();
    let mut childs = vec![];

    for i in 0..n {
        let us = Arc::clone(&urls);
        let fs = Arc::clone(&filenames);
        let c = thread::spawn(move || download(&us[i], &fs[i]));
        childs.push(c);
    }
    for c in childs {
        let _ = c.join().unwrap();
    }
}

pub fn channel_multithread_downloader(urls: URLS, filenames: FILES) {
    let (tx, rx) = channel();
    let n = urls.len();
    let join_handle = thread::spawn(move || loop {
        match rx.recv() {
            Ok(data) => {
                let (url, filename) = data;
                let _ = download(url, &filename);
                ()
            }
            Err(_) => thread::yield_now(),
        }
    });
    for i in 0..n {
        tx.send((urls[i], filenames[i].clone())).unwrap()
    }
    join_handle.join().unwrap();
}

pub fn sync_channel_multithread_downloader(urls: URLS, filenames: FILES, n_workers: usize) {
    let (tx, rx) = sync_channel(n_workers);
    let n = urls.len();
    let join_handle = thread::spawn(move || loop {
        match rx.recv() {
            Ok(data) => {
                let (url, filename) = data;
                let _ = download(url, &filename);
                ()
            }
            Err(_) => thread::yield_now(),
        }
    });
    for i in 0..n {
        tx.send((urls[i], filenames[i].clone())).unwrap()
    }
    join_handle.join().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_test() {
        let url = "https://upload.wikimedia.org/wikipedia/commons/d/d5/Rust_programming_language_black_logo.svg";
        let filename = PathBuf::from("/tmp/Rust_programming_language_black_logo_test1.svg");
        let _status = download(&url, &filename).unwrap();
        assert!(filename.exists());
        let _status = std::fs::remove_file(filename);
    }

    #[test]
    fn simple_multithread_downloader_test() {
        let urls = vec!["https://upload.wikimedia.org/wikipedia/commons/d/d5/Rust_programming_language_black_logo.svg",
                        "https://upload.wikimedia.org/wikipedia/commons/thumb/d/d5/Rust_programming_language_black_logo.svg/240px-Rust_programming_language_black_logo.svg.png"];
        let filenames = vec![
            PathBuf::from("/tmp/Rust_programming_language_black_logo.svg"),
            PathBuf::from("/tmp/240px-Rust_programming_language_black_logo.svg.png"),
        ];
        let filenames_cloned = filenames.clone();
        simple_multithread_downloader(urls, filenames);
        for f in filenames_cloned {
            assert!(f.exists());
            let _status = std::fs::remove_file(f);
        }
    }
    #[test]
    #[ignore = "I'dont know how to a thread. Could you tell me?"]
    fn channel_multithread_downloader_test() {
        let urls = vec!["https://upload.wikimedia.org/wikipedia/commons/d/d5/Rust_programming_language_black_logo.svg",
                        "https://upload.wikimedia.org/wikipedia/commons/thumb/d/d5/Rust_programming_language_black_logo.svg/240px-Rust_programming_language_black_logo.svg.png"];
        let filenames = vec![
            PathBuf::from("/tmp/Rust_programming_language_black_logo-2.svg"),
            PathBuf::from("/tmp/240px-Rust_programming_language_black_logo.svg-2.png"),
        ];
        let filenames_cloned = filenames.clone();
        channel_multithread_downloader(urls, filenames);
        for f in filenames_cloned {
            assert!(f.exists());
            let _status = std::fs::remove_file(f);
        }
    }
    #[test]
    #[ignore = "I'dont know how to a thread. Could you tell me?"]
    fn sync_channel_multithread_downloader_test() {
        let urls = vec!["https://upload.wikimedia.org/wikipedia/commons/d/d5/Rust_programming_language_black_logo.svg",
                        "https://upload.wikimedia.org/wikipedia/commons/thumb/d/d5/Rust_programming_language_black_logo.svg/240px-Rust_programming_language_black_logo.svg.png"];
        let filenames = vec![
            PathBuf::from("/tmp/Rust_programming_language_black_logo-3.svg"),
            PathBuf::from("/tmp/240px-Rust_programming_language_black_logo.svg-3.png"),
        ];
        let filenames_cloned = filenames.clone();
        sync_channel_multithread_downloader(urls, filenames, 1);
        for f in filenames_cloned {
            assert!(f.exists());
            let _status = std::fs::remove_file(f);
        }
    }
}
