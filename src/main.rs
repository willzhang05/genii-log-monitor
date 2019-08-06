use std::env;
use std::thread;
use std::fs::File;
use std::io::{BufReader, Read};
use std::io::prelude::*;
use std::sync::mpsc::channel;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 {
        if &args[1] == "start" {

            let (tx, rx) = channel();


            let reader = thread::spawn(move || {
                let file = File::open(&args[2]).expect("Unable to open file.");
                let mut buf_reader = BufReader::new(file);
                let mut contents = String::new();
                loop {
                    let line_len = buf_reader.read_line(&mut contents).expect("Unable to read from buffer.");
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
        }
    } else {
        eprintln!("Invalid arguments");
    }
}
