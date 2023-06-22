use clap::{value_parser, Parser};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::mpsc::channel;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_parser = value_parser!(PathBuf))]
    path: PathBuf,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();

    // Add a path to be watched.
    watcher
        .watch(&args.path, RecursiveMode::NonRecursive)
        .unwrap();

    let mut file = File::open(&args.path)?;
    file.seek(SeekFrom::End(0))?; // start reading from the end of file

    let mut reader = BufReader::new(file);

    loop {
        match rx.recv() {
            Ok(_) => {
                print_sql(&mut reader)?;
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

fn print_sql(reader: &mut BufReader<File>) -> io::Result<()> {
    let mut line = String::new();

    loop {
        match reader.read_line(&mut line)? {
            0 => return Ok(()),
            _ => {
                // TODO probably better still to use a treesitter grammar + highlights
                let formatted = sqlformat::format(
                    &line[52..], // TODO could actually regex out the irrelevant stuff
                    &sqlformat::QueryParams::None,
                    sqlformat::FormatOptions::default(),
                );
                println!("{}", formatted);
                println!("{}", "\n");
                line.clear();
            }
        }
    }
}
