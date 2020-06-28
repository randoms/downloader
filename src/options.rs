pub struct DownloadOptions {
    pub source: String,
    pub output: String,
    pub thread: u32,
}

impl Default for DownloadOptions {
    fn default() -> DownloadOptions {
        return DownloadOptions {
            source: String::new(),
            output: String::new(),
            thread: 10,
        }
    }
}