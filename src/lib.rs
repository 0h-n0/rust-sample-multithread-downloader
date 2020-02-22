extern crate curl;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

fn download(url: &PathBuf, filename: &PathBuf) -> std::io::Result<()> {
    println!("Now downloading data from {:?}", url);
    if filename.exists() {
        println!("{:?} already exsits.", filename);
        return Ok(());
    }
    let f = std::fs::File::create(&filename)?;
    let mut writer = std::io::BufWriter::new(f);
    let mut easy = curl::easy::Easy::new();
    easy.url(url.to_str().unwrap())?;
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

fn simple_multithread_download(urls: Vec<PathBuf>, filenames: Vec<PathBuf>) {
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
        c.join().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_test() {
        let mut url = PathBuf::from("https://upload.wikimedia.org/wikipedia/commons/d/d5/Rust_programming_language_black_logo.svg");
        let mut filename = PathBuf::from("/tmp/Rust_programming_language_black_logo_test1.svg");
        let _status = download(&url, &filename).unwrap();
        assert!(filename.exists());
        let _status = std::fs::remove_file(filename);
    }

    #[test]
    fn simple_multithread_download_test() {
        let urls = vec![PathBuf::from("https://upload.wikimedia.org/wikipedia/commons/d/d5/Rust_programming_language_black_logo.svg"),
                        PathBuf::from("https://upload.wikimedia.org/wikipedia/commons/thumb/d/d5/Rust_programming_language_black_logo.svg/240px-Rust_programming_language_black_logo.svg.png")];
        let filenames = vec![
            PathBuf::from("/tmp/Rust_programming_language_black_logo.svg"),
            PathBuf::from("/tmp/240px-Rust_programming_language_black_logo.svg.png"),
        ];
        let filenames_cloned = filenames.clone();
        simple_multithread_download(urls, filenames);
        for f in filenames_cloned {
            assert!(f.exists());
            let _status = std::fs::remove_file(f);
        }
    }
}
