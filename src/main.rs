mod tokenizer;
mod runtime;
mod parser;
mod generator;
mod analyzer;

use crate::runtime::Runtime;
use std::env;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let mut runtime = Runtime::new();

    match args.len() {
        // No arguments - run REPL
        1 => runtime.run_repl(),
        
        // File argument provided
        2 => {
            let file_path = &args[1];
            runtime.run_file(file_path)
        },
        
        // Invalid number of arguments
        _ => Err("Usage: nair [script]".to_string()),
    }
}
