mod tokenizer;
mod runtime;
mod parser;

use crate::runtime::Runtime;

fn main() -> Result<(), String> {
    let mut runtime = Runtime::new();
    runtime.run()
}
