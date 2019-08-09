extern crate toml;
extern crate chrono;
extern crate lettre;

use std::string::String;
use std::path::{Path, PathBuf};
use std::env;
use std::thread;
use std::fs;
use std::io::BufReader;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::sync::mpsc;

use std::time::Duration;
use chrono::prelude::*;

//use lettre::email::EmailBuilder;
//use lettre::transport::EmailTransport;

#[derive(serde::Deserialize)]
struct Container {
    name: String,
    install_dir: String,
    properties_file: String,
    email_notify: Vec<String>,
    enabled: bool
}

#[derive(serde::Deserialize)]
struct Config {
    containers: Vec<Container>
}

fn read_log_properties(prop_path: &Path) -> (PathBuf, usize) {
    let prop_file = fs::File::open(prop_path).expect("Unable to open properties file.");

    let prop_reader = BufReader::new(prop_file);
    let prop_contents = String::new();
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
            //println!("{:?}", max_log_size_string);
            let max_log_size = max_log_size_string.parse::<usize>().expect("Unable to parse maximum log size");
        }
    }
    return (log_path, max_log_size);
}


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 {
        if &args[1] == "start" {

            let config_file = fs::read_to_string(&args[2]).expect("Unable to open config file.");
            let config_info: Config = toml::from_str(&config_file).expect("Unable to parse config file.");

            for container in config_info.containers {
                if container.enabled {

                    let prop_path = Path::new(&container.install_dir).join(&container.properties_file);
                    let (log_path, max_log_size) = read_log_properties(&prop_path);

                    let (tx, rx) = mpsc::channel();

                    let reader = thread::spawn(move || {
                        //let log_path = Path::new(&container.install_dir).join(&container.log_file);
                        //println!("{:?}", log_path);
                        let mut file = fs::File::open(&log_path).expect("Unable to open log file.");
                        file.seek(SeekFrom::End(0)).expect("Unable to seek to end of log file.");
                        let mut f = BufReader::new(file);
                        let mut contents =  String::new();

                        println!("Ready and monitoring {}.", log_path.to_str().unwrap());

                        loop {
                            contents.clear();
                            let line_len = f.read_line(&mut contents).expect("Unable to read line.");
                            f.consume(line_len);
                            
                            if contents.len() != 0 {
                                let log_line: Vec<&str> = contents.split_whitespace().collect();
                                if log_line.len() > 2 && log_line[2] == "ERROR" {
                                    tx.send(contents.to_owned()).expect("Unable to send on channel.");
                                }
                            }
                        }
                    });

                    let notifier = thread::spawn(move || {
                        loop {
                            let value = rx.recv().expect("Unable to receive from channel.");
                            println!("{}", value);
                        }
                    });

                    reader.join().expect("The sender thread has panicked!");
                    notifier.join().expect("the receiver thread has panicked!");
                }
            }

        } else if &args[1] == "test" {
            let mut buffer = fs::File::create(&args[2]).expect("Unable to create file.");
            loop {
                let local: DateTime<Local> = Local::now();
                let local_time = local.to_string() + "\n";
                buffer.write(&local_time.as_bytes()).expect("Unable to write data.");
                println!("{:?}", local);
                thread::sleep(Duration::from_millis(1000));
            }
        }
    } else {
        eprintln!("Invalid arguments.");
    }
}
