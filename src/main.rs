mod ccerror;
mod source;

use std::fs;
use std::path::PathBuf;
use std::process::exit;

use clap::Parser;

use ccerror::CcError;
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

fn open_source(name: &PathBuf) -> Result<SourceFile, CcError> {
    let text = match fs::read_to_string(&name) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("failed to read source file {}: {}", name.to_string_lossy(), e);
            exit(1);            
        }
    };

    let source = text.chars().collect();

    let source = Source::new(source);

    Ok(SourceFile{ name: name.clone(), source })
}

fn main() {
    let args = Args::parse();
    let source = open_source(&args.source_file);

    
}

