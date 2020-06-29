mod utils;
use utils::get_data_from_header;
use log::{info};
use std::io::Read;

// use options::DownloadOptions;

// struct HttpDownloader {
//     options: DownloadOptions,
// }

// impl HttpDownloader {
//     fn new(options:DownloadOptions) -> {
//         HttpDownloader {
//             options: options,
//         }
//     }

//     fn start() -> Result<String, String>{
        
//     }

//     fn cancel() -> Result<String, String>{

//     }
// }

// struct DownloadTask {
//     source: String,
//     chunk_size: u64,

// }
#[derive(Debug)]
pub struct WebFile {
    source: String, // 源文件网络地址
    filename: String, // 源文件文件名
    size: u64, // 源文件大小
    current_index: u64, // 当前的分割位置
    chunk_size: u64 // 大块的文件分割大小
}

impl WebFile {
    pub fn new(source:&str) -> Result<WebFile, Box<dyn std::error::Error>>{
        let response = reqwest::blocking::Client::new().get(source)
        .send()?;
        let filesize = response.content_length().unwrap_or_default();
        let mut webfile = WebFile {
            source: String::from(source),
            filename: String::new(),
            size: filesize,
            current_index: 0,
            chunk_size: 1 * 1024 * 1024 // 1MB
        };
        let header = response.headers();
        if header.get("content-disposition").is_some() {
            // 从 content disposition 中获取文件名
            info!("Get filename from header");
            let disposition_str = header.get("content-disposition").unwrap();
            let data = get_data_from_header(
                disposition_str.to_str().unwrap()
            );
            println!("{:?}", data);
            webfile.filename = data.get("filename").unwrap().to_owned();
        }else{
            info!("Get filename from url address");
            let mut path = "";
            let final_url = response.url().to_string();
            if final_url.starts_with("https://") {
                path = &final_url["https://".len() .. final_url.len()];
            }
            if final_url.starts_with("http://") {
                path = &final_url["http://".len() .. final_url.len()];
            }
            webfile.filename = std::path::Path::new(path).file_name().unwrap().to_str().unwrap().to_owned();
        }
        Ok(webfile)
    }

    pub fn set_chunk_size(&mut self, size: u64) -> &WebFile {
        self.chunk_size = size;
        return self;
    }

    pub fn get_file_size(&self) -> u64 {
        return self.size;
    }

    pub fn get_filename(&self) -> String {
        return self.filename.to_owned();
    }
}

impl Iterator for WebFile {
    type Item = FileChunk;

    fn next(&mut self) -> Option<Self::Item> {
        let mut file_chunk = FileChunk::new();
        file_chunk.source = self.source.clone();
        file_chunk.size = self.chunk_size;
        file_chunk.download_index = self.current_index;
        if self.current_index + self.chunk_size > self.size {
            // 最后一个chunk而且不能均分
            file_chunk.size = self.size % self.chunk_size;
        }
        if self.current_index >= self.size {
            return None;
        }
        self.current_index += self.chunk_size;
        return Some(file_chunk);
    }
}

#[derive(Debug, Clone)]
pub struct FileChunk {
    source: String, // 源文件地址
    size: u64, // 块大小
    current_index: u64, // 当前的分割位置
    download_index: u64, // 当前的下载位置
    chunk_size: u64, // 继续分割的小块大小
    cache: Vec<u8>
}

impl FileChunk {
    pub fn new() -> FileChunk{
        FileChunk {
            source: String::new(),
            size: 0,
            current_index: 0,
            download_index: 0,
            chunk_size: 100 * 1024, // 100KB
            cache: Vec::new(),
        }
    }

    pub fn set_chunk_size(&mut self, size: u64) -> &FileChunk {
        self.chunk_size = size;
        return self;
    }

    pub fn get_download_index(&self) -> u64 {
        return self.download_index;
    }

    pub fn get_download_cache(& self) -> &Vec<u8> {
        return &self.cache;
    }

    pub fn download(&mut self) -> Result<&Vec<u8>, Box<dyn std::error::Error>> {
        let mut res = reqwest::blocking::Client::new().get(self.source.as_str())
            .timeout(std::time::Duration::from_secs(5))
            .header("range", format!("bytes={}-{}", self.download_index, self.download_index + self.size))
            .send()?;
        let mut buf = [0; 1024];
        while let Ok(n) = res.read(&mut buf) {
            if n == 0 { break; }
            let data = (&buf[0..n]).to_owned();
            self.cache.extend(data.iter().copied());
            if self.cache.len() > (self.size as usize) {
                self.cache.truncate(self.size as usize);
                break;
            }
        }
        
        Ok(&self.cache)
    }
}

impl Iterator for FileChunk {
    type Item = FileChunk;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk = FileChunk::new();
        chunk.source = self.source.clone();
        chunk.size = self.chunk_size;
        chunk.chunk_size = self.chunk_size / 10;
        chunk.download_index = self.download_index + self.current_index;
        if self.current_index + self.chunk_size > self.size {
            // 最后一个，且不能均分
            chunk.size = self.size % self.chunk_size;
        }
        if self.current_index >= self.size {
            return None;
        }
        self.current_index += self.chunk_size;
        return Some(chunk);
    }
}