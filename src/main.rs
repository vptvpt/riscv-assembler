extern crate core;

mod parser;
mod tokenizer;

use parser::Parser;
use std::env;

fn main() {
    for source_file_name in env::args().skip(1) {
        let mut parser = Parser::new(source_file_name);
        parser.parse();
        parser.write_to_file();
    }
}
