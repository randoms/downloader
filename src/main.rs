extern crate argparse;
use argparse::{ArgumentParser, Store};
mod options;
mod http_downloader;
mod file_writer;
mod threadpool;

use options::DownloadOptions;
use http_downloader::{WebFile};
use file_writer::FileWriter;
use threadpool::ThreadPool;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    // 初始化命令行参数
    let mut download_options = DownloadOptions::default();
    { 
        let mut ap = ArgumentParser::new();
        ap.set_description("Http multithread downloader.");
        ap.refer(&mut download_options.source)
            .add_option(&["-s", "--source"], Store,
            "Download source url address or seed file location");
        ap.refer(&mut download_options.output)
            .add_option(&["-o", "--output"], Store,
            "Output file path");
        ap.refer(&mut download_options.thread)
            .add_option(&["-t", "--thread"], Store, 
            "Download thread num");
        ap.parse_args_or_exit();
    }
    // 下载测试数据
    let mut file = WebFile::new("http://www.bwbot.org/static/video/obj3.mp4")?;
    // 文件写入
    let mut file_writer = FileWriter::new("test.mp4", file.get_file_size());
    // 线程池
    let mut threadpool = ThreadPool::new(2);
    file.set_chunk_size(1024 * 1024);
    for f in file.into_iter() {
        let mut f = f.clone();
        f.set_chunk_size(512 * 1024);
        for file_chunk in f.into_iter() {
            // 细分的块
            let tx = file_writer.get_tx();
            let mut file_chunk = file_chunk.clone();
            threadpool.add_task(move || {
                file_chunk.download().expect("Download block failed");
                tx.send(file_chunk).expect("Send data to filewriter failed");
            });
        }
    };
    file_writer.write().expect("Write file failed");
    Ok(())
}
