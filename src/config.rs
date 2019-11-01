use std::fs;
use std::io::{BufReader, prelude::*};
use std::path::{Path, PathBuf};

#[derive(serde::Deserialize)]
pub struct Container {
    pub alias: String,
    pub name: String,
    pub install_dir: String,
    pub properties_file: String,
    pub src_email: String,
    pub email_list: Vec<String>,
    pub notify_interval: i64,
    pub flap_interval: i64,
    pub cache_size: usize,
    pub enabled: bool
}

#[derive(Copy, Clone, Debug)]
pub struct ErrorInfo {
    pub last_update: chrono::NaiveDateTime,
    pub update_period: chrono::Duration,
    pub email_sent: bool
}

#[derive(serde::Deserialize)]
pub struct Config {
    pub containers: Vec<Container>
}

pub fn read_log_properties(prop_path: &Path) -> (PathBuf, usize) {
    let prop_file = fs::File::open(prop_path).expect("Unable to open properties file.");
    let prop_reader = BufReader::new(prop_file);
    let mut log_path = PathBuf::new();
    let mut max_log_size: usize = 0;

    for line in prop_reader.lines() {
        let line_str = line.unwrap();
        if line_str.starts_with('#') {
            continue;
        }
        if line_str.contains("log4j.appender.LOGFILE.File") {
            let split_line: Vec<&str> = line_str.split("=").to_owned().collect();
            log_path = Path::new(split_line[1].clone()).to_path_buf();
        }
        if line_str.contains("log4j.appender.LOGFILE.MaxFileSize") {
            let split_line: Vec<&str> = line_str.split("=").to_owned().collect();
            let mut max_log_size_string = split_line[1].to_string().clone();
            max_log_size_string.truncate(max_log_size_string.len() - 2);
            max_log_size = max_log_size_string.parse::<usize>().expect("Unable to parse max log size");
        }
    }
    return (log_path, max_log_size);
}

