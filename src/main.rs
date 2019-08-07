extern crate chrono;

use std::env;
use std::thread;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::sync::mpsc::channel;
use std::time::Duration;
use chrono::prelude::*;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 {
        if &args[1] == "start" {
            let (tx, rx) = channel();

            let reader = thread::spawn(move || {
                let file = File::open(&args[2]).expect("Unable to open file.");
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
