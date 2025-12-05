mod interpreter;

use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "trainfuck")]
#[command(author = "Hitesh")]
#[command(version = "0.1.0")]
#[command(about = "Trainfuck interpreter - Brainfuck with networking extensions")]
struct Args {
    /// The Trainfuck source file to execute
    #[arg(required = true)]
    file: PathBuf,

    /// Enable debug mode (prints tape state)
    #[arg(short, long)]
    debug: bool,
}

fn main() {
    let args = Args::parse();

    // Read source file
    let source = match fs::read_to_string(&args.file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file {:?}: {}", args.file, e);
            std::process::exit(1);
        }
    };

    if args.debug {
        eprintln!(
            "[trainfuck] Loaded {} bytes from {:?}",
            source.len(),
            args.file
        );
    }

    // Parse
    let ops = match interpreter::parse(&source) {
        Ok(ops) => ops,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    };

    if args.debug {
        eprintln!("[trainfuck] Parsed {} operations", ops.len());
    }

    // Execute
    let mut vm = interpreter::VM::new();
    if let Err(e) = vm.execute(&ops) {
        eprintln!("Runtime error: {}", e);
        std::process::exit(1);
    }
}
