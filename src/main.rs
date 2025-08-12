mod ccerror;
mod lexer;
mod source;

use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;

use clap::Parser;

use source::Source;

#[derive(clap::Parser)]
struct Args {
    #[arg(short = 'I')]
    includes: Vec<PathBuf>,
    #[arg(short = 'D')]
    defines: Vec<String>,

    source_file: PathBuf,
}

struct SourceFile {
    name: PathBuf,
    source: Source,
}

fn main() {
    let args = Args::parse();

    let mut source = Source::new();

    match source.push_file(&args.source_file) {
        Ok(()) => {},
        Err(e) => {
            eprintln!("{}: {}", args.source_file.to_string_lossy(), e);
            exit(1);
        }
    };

    loop {
        if let Some(ch) = source.next() {
            println!("{}@{}:{}: {} ", 
                source.get_filename(ch.pt.file).unwrap(), 
                ch.pt.line, 
                ch.pt.col, 
                ch.ch); 

            if ch.pt.line == 14 && ch.pt.col == 15 {
                source.push_file(&PathBuf::from_str("../rustcc/testdata/test.c").unwrap()).unwrap();
            }

        } else {
            break;
        }
    }


    
}

