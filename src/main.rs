// William Zhang

extern crate toml;
extern crate chrono;
extern crate lru;
extern crate lettre;

use std::string::String;
use std::path::{Path, PathBuf};
use std::{env, fs, thread};
use std::io::{BufReader, SeekFrom, prelude::*};
use std::sync::{mpsc, Arc, Mutex};

use chrono::{Duration, prelude::*};

use lru::LruCache;

//use lettre::email::EmailBuilder;
//use lettre::transport::EmailTransport;

#[derive(serde::Deserialize)]
struct Container {
    name: String,
    install_dir: String,
    properties_file: String,
    email_notify: Vec<String>,
    notify_interval: i64,
    cache_size: usize,
    enabled: bool
}

#[derive(serde::Deserialize)]
struct Config {
    containers: Vec<Container>
}

#[derive(Copy, Clone, Debug)]
struct ErrorInfo {
    last_update: chrono::NaiveDateTime,
    update_period: i64,
    email_sent: bool
}

fn read_log_properties(prop_path: &Path) -> (PathBuf, usize) {
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
            max_log_size = max_log_size_string.parse::<usize>().expect("Unable to parse maximum log size");
        }
    }
    return (log_path, max_log_size);
}

fn monitor_log(error_cache: &Arc<Mutex<LruCache<String, ErrorInfo>>>, tx: &mpsc::Sender<String>, log_path: &Path, max_log_size: usize) {
    let mut log_file = fs::File::open(log_path).expect("Unable to open log file.");
    log_file.seek(SeekFrom::End(0)).expect("Unable to seek to end of log file.");
    let mut log_reader = BufReader::new(log_file);
    let mut contents =  String::new();

    println!("Ready and monitoring {}.", log_path.to_str().unwrap());

    loop {
        log_reader.get_mut().sync_all().expect("Unable to sync log file.");
        let metadata = log_reader.get_ref().metadata().expect("Unable to fetch log file metadata.");
        let log_size: f32 = metadata.len() as f32 / 1e6 as f32;
        // Untested, meant to handle log rollover when maximum log size is reached.
        if log_size >= max_log_size as f32 {
            log_reader.get_mut().seek(SeekFrom::Start(0)).expect("Unable to seek to start of log file.");
        }
        contents.clear();
        log_reader.read_line(&mut contents).expect("Unable to read line.");
        
        if contents.len() != 0 {
            let log_split: Vec<&str> = contents.split_whitespace().collect();
            if log_split.len() > 3 && (log_split[2] == "DEBUG" || log_split[2] == "ERROR" || log_split[2] == "WARN") {
                let log_date = log_split[..2].join(" ");

                let error_date = NaiveDateTime::parse_from_str(log_date.as_str(), "%Y-%m-%d %H:%M:%S%.3f").expect("Invalid date format in log.");
                let error_msg = log_split[3..].join(" ");

                let mut error_cache_lock = error_cache.lock().unwrap();
                if error_cache_lock.contains(&error_msg) {
                    let mut error_info = (&mut error_cache_lock).get_mut(&error_msg).unwrap();
                    
                    let current_time = Utc::now().naive_local();

                    let error_period = current_time.signed_duration_since(error_info.last_update).num_seconds();
                    error_info.last_update = current_time;
                    error_info.update_period = error_period;

                    //(&mut error_cache_lock).put(error_msg, error_info);
                    println!("[READER] {:?} {:?} Updated!", error_msg, (&mut error_cache_lock).get(&error_msg).unwrap());
                } else {
                    let error_info: ErrorInfo = ErrorInfo { last_update: error_date, update_period: 0, email_sent: false };
                    println!("[READER] {:?} {:?} Inserted!", error_msg, error_info);
                    (&mut error_cache_lock).put(error_msg, error_info);
                }

                println!("[READER]  {:?}", (&error_cache_lock).len());
                tx.send(contents.to_owned()).expect("Unable to send on channel.");
            }
        }
    }
}

fn notify_error(error_cache: &Arc<Mutex<LruCache<String, ErrorInfo>>>, rx: &mpsc::Receiver<String>, email_list: &Vec<String>, notify_interval: chrono::Duration) {
    let start_time = Utc::now().naive_local();
    loop {
        let value = rx.recv().expect("Unable to receive from channel.");
        println!("{}", value);
        let current_time = Utc::now().naive_local();
        if current_time.signed_duration_since(start_time) > notify_interval {
            let mut error_cache_lock = error_cache.lock().unwrap();
            println!("[NOTIFIER]  {:?}", (&error_cache_lock).len());
            if error_cache_lock.contains(&value) {
                println!("[NOTIFIER] {:?} {:?}", value, (&mut error_cache_lock).get(&value).unwrap());
            }
        }
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 {
        if &args[1] == "start" {

            let config_file = fs::read_to_string(&args[2]).expect("Unable to open config file.");
            let config_info: Config = toml::from_str(&config_file).expect("Unable to parse config file.");

            for container in config_info.containers {
                if container.enabled {
                    println!("{:?} container monitoring enabled.", &container.name);

                    let prop_path = Path::new(&container.install_dir).join(&container.properties_file);
                    let (log_path, max_log_size) = read_log_properties(&prop_path);

                    let (tx, rx) = mpsc::channel();
                    let error_cache: Arc<Mutex<LruCache<String, ErrorInfo>>> = Arc::new(Mutex::new(LruCache::new(container.cache_size)));

                    let reader = thread::spawn({ let error_cache = error_cache.clone(); move || {
                        monitor_log(&error_cache, &tx, &log_path, max_log_size);
                    }});

                    let notify_interval = Duration::minutes(container.notify_interval);

                    let notifier = thread::spawn({ let error_cache = error_cache.clone(); move || {
                        notify_error(&error_cache, &rx, &container.email_notify, notify_interval);
                    }});

                    reader.join().expect("The sender thread has panicked!");
                    notifier.join().expect("the receiver thread has panicked!");
                }
            }
        }
    } else {
        eprintln!("Invalid arguments.");
    }
}
