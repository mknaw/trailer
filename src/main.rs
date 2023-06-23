use clap::{value_parser, Parser};
use notify::{Config, KqueueWatcher, RecursiveMode, Watcher};
use std::fs::File;
use std::io::{self, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::mpsc::channel;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_parser = value_parser!(PathBuf))]
    path: PathBuf,
}

mod processor;

fn main() -> io::Result<()> {
    let args = Args::parse();

    let (tx, rx) = channel();
    let mut watcher = KqueueWatcher::new(tx, Config::default()).unwrap();

    // Add a path to be watched.
    watcher
        .watch(&args.path, RecursiveMode::NonRecursive)
        .unwrap();

    let mut file = File::open(&args.path)?;
    file.seek(SeekFrom::End(0))?; // start reading from the end of file

    let mut reader = BufReader::new(file);
    let mut parser = processor::init_parser();

    loop {
        match rx.recv() {
            Ok(_) => {
                processor::process(&mut reader, &mut parser)?;
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
