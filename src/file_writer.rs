use std::sync::mpsc;
use super::http_downloader::FileChunk;
use std::io::SeekFrom;
use std::io::Seek;
use std::io::Write;
use std::time::{SystemTime};

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
        let mut now = SystemTime::now();
        let start = SystemTime::now();
        for data in &self.rx {
            file.seek(SeekFrom::Start(data.get_download_index())).expect("Seek download file failed");
            let downloaded_data = data.get_download_cache();
            file.write(&downloaded_data[0..downloaded_data.len()]).expect("Write download file failed");
            self.total_write_bytes += downloaded_data.len() as u64;
            for i in 0..downloaded_data.len() {
                self.file_flags[(data.get_download_index() + (i as u64)) as usize] = true;
            }
            // 计算下载速度， 估算剩余时间
            let downloadspeed = (downloaded_data.len() as f64) * 1000 as f64 / now.elapsed().unwrap().as_millis() as f64 / 1024f64;
            let remaining = (self.file_size - self.total_write_bytes) as f64 / (downloadspeed) / 1024f64;
            println!("Download Speed: {:.2} KB/s, Time elaspsed: {}, Time Remaining: {:.1}", 
                downloadspeed, start.elapsed().unwrap().as_secs(), remaining);
            // 显示下载进度, 一个文件分成128份进行显示
            let mut progress_str = String::new();
            let step = self.file_size / 128;
            for i in 0..128 {
                let part = &(self.file_flags[i * step as usize.. ( i + 1 ) * step as usize]);
                if !part.iter().any(|x|!x) {
                    progress_str.push('>');
                }else{
                    progress_str.push('=');
                }
            }
            println!("{}", progress_str);
            now = SystemTime::now();

            // 检查文件是否全部写入
            if self.total_write_bytes == self.file_size && !(self.file_flags.iter().any(|x| !x)) {
                // 文件全部写入完成
                break;
            }
        }
        return Ok(());
    }
}