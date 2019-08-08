extern crate toml;
extern crate chrono;

use std::string::String;
use std::path::Path;
use std::env;
use std::thread;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::sync::mpsc;
//use toml::{Value, de::Error};

use std::time::Duration;
use chrono::prelude::*;

#[derive(serde::Deserialize)]
struct Container {
    name: String,
    install_dir: String,
    log_file: String,
    properties_file: String,
    enabled: bool
}

#[derive(serde::Deserialize)]
struct Config {
    containers: Vec<Container>
}


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 {
        if &args[1] == "start" {

            let config_file = std::fs::read_to_string(&args[2]).expect("Unable to open config file.");
            let config_info: Config = toml::from_str(&config_file).expect("Unable to parse config file.");

            for container in config_info.containers {
                if container.enabled {
                    let (tx, rx) = mpsc::channel();

                    let reader = thread::spawn(move || {
                        let log_path = Path::new(&container.install_dir).join(&container.log_file);
                        println!("{:?}", log_path);
                        let file = File::open(&log_path).expect("Unable to open file.");
                        let mut f = BufReader::new(file);
                        let mut contents =  String::new();
                        loop {
                            contents.clear();
                            let line_len = f.read_line(&mut contents).expect("Unable to read line.");
                            f.consume(line_len);
                            if contents.len() != 0 {
                                tx.send(contents.to_owned()).expect("Unable to send on channel");
                            }
                        }
                    });

                    let notifier = thread::spawn(move || {
                        loop {
                            let value = rx.recv().expect("Unable to receive from channel");
                            println!("{}", value);
                        }
                    });

                    reader.join().expect("The sender thread has panicked!");
                    notifier.join().expect("the receiver thread has panicked!");
                }
            }

        } else if &args[1] == "test" {
            let mut buffer = File::create(&args[2]).expect("Unable to create file.");
            loop {
                let local: DateTime<Local> = Local::now();
                let local_time = local.to_string() + "\n";
                buffer.write(&local_time.as_bytes()).expect("Unable to write data.");
                println!("{:?}", local);
                thread::sleep(Duration::from_millis(1000));
            }
        }
    } else {
        eprintln!("Invalid arguments");
    }
}
