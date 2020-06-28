use std::sync::mpsc;
use super::http_downloader::FileChunk;
use std::io::SeekFrom;
use std::io::Seek;
use std::io::Write;

pub struct FileWriter {
    rx: mpsc::Receiver<FileChunk>,
    tx: mpsc::Sender<FileChunk>,
    file_path: String,
    file_size: u64,
    file_flags: Vec<bool>,
    total_write_bytes: u64,
}

impl FileWriter {
    pub fn new(filepath:&str, file_size:u64) -> FileWriter {
        let (tx, rx) = mpsc::channel();
        FileWriter {
            rx: rx,
            tx: tx,
            file_path: String::from(filepath),
            file_size: file_size,
            file_flags: vec![false; file_size as usize],
            total_write_bytes: 0
        }
    }

    pub fn get_tx(&self) -> mpsc::Sender<FileChunk> {
        let tx1 = mpsc::Sender::clone(&self.tx);
        return tx1;
    }

    pub fn write(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = std::fs::File::create(self.file_path.as_str()).expect("create failed");
        file.set_len(self.file_size).expect("resize download file failed");
        for data in &self.rx {
            file.seek(SeekFrom::Start(data.get_download_index())).expect("Seek download file failed");
            let downloaded_data = data.get_download_cache();
            file.write(&downloaded_data[0..downloaded_data.len()]).expect("Write download file failed");
            self.total_write_bytes += downloaded_data.len() as u64;
            for i in 0..downloaded_data.len() {
                self.file_flags[(data.get_download_index() + (i as u64)) as usize] = true;
            }
            println!("{}={}", self.total_write_bytes, self.file_size);
            if self.total_write_bytes == self.file_size && !(self.file_flags.iter().any(|x| !x)) {
                // 文件全部写入完成
                break;
            }
        }
        return Ok(());
    }
}