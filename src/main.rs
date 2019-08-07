extern crate chrono;
extern crate tokio;

use std::env;
use std::thread;
use std::fs::File;
use std::io::{BufReader, Read};
use std::io::prelude::*;
use std::sync::mpsc::channel;
//use notify::{RecommendedWatcher, RecursiveMode, Result, watcher};
use std::time::Duration;
use chrono::prelude::*;

use tokio::prelude::Future;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 {
        if &args[1] == "start" {
            
            //let mut watcher = watcher(Duration::from_secs(1)).expect("Failed to create watcher.");
            //watcher.watch(&args[2]).expect("Failed to watch file.");

            /*let (tx, rx) = channel();


            let reader = thread::spawn(move || {
                let file = File::open(&args[2]).expect("Unable to open file.");
                let mut buf_reader = BufReader::new(file);
                let mut contents = String::new();
                loop {
                    let line_len = buf_reader.read_line(&mut contents).expect("Unable to read from buffer.");
                    buf_reader.consume(line_len);
                    println!("{}", contents);
                    tx.send(contents.to_owned()).expect("Unable to send on channel");
                }
            });

            let notifier = thread::spawn(move || {
                let value = rx.recv().expect("Unable to receive from channel");
                println!("{}", value);
            });

            reader.join().expect("The sender thread has panicked!");
            notifier.join().expect("the receiver thread has panicked!");
            */
            let task = tokio::fs::read(args[2].clone()).map(|data| {
                println!("{:?}", String::from_utf8_lossy(&data));
            }).map_err(|e| {
                eprintln!("IO error: {:?}", e);
            });
            tokio::run(task);

        } else if &args[1] == "test" {
            let mut buffer = File::create(&args[2]).expect("Unable to create file.");
            loop {
                let local: DateTime<Local> = Local::now();
                let local_time = local.to_string() + "\n";
                let bytes_written = buffer.write(&local_time.as_bytes()).expect("Unable to write data.");
                println!("{:?}", local);
                thread::sleep(Duration::from_millis(1000));
            }
        }
    } else {
        eprintln!("Invalid arguments");
    }
}
