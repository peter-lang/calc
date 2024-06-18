use std::env;

use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use crate::error::CalcError;
use crate::parser::lexer::Lexer;
use crate::parser::parser::Parser;

mod currency;
mod debug;
mod error;
mod files;
mod node;
mod number;
mod number_op;
mod parser;
mod rational;
mod unit;
mod value;
mod value_op;

fn main() -> Result<(), CalcError> {
    let path_cache_history = files::cache("history.txt");

    let lexer = Lexer::new();
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        let mut rl = DefaultEditor::new()?;
        if path_cache_history.exists() {
            rl.load_history(&path_cache_history)?;
        }

        let mut parser = Parser::new();
        let mut line_buffer = String::new();
        loop {
            match rl.readline(if parser.is_empty() { ">> " } else { ".. " }) {
                Ok(line) => {
                    if !line_buffer.is_empty() {
                        line_buffer.push_str(" ");
                    }
                    line_buffer.push_str(line.as_str());
                    parser.extend(lexer.parse(line.as_str()));

                    if let Some(node) = parser.parse() {
                        let eval = node.eval();
                        match eval {
                            Ok(res) => println!("{res}"),
                            Err(error) => println!("{error}"),
                        }

                        let _ = rl.add_history_entry(line_buffer.as_str());
                        line_buffer.clear();
                        parser.reset();
                    }
                }
                Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                    if parser.is_empty() {
                        break;
                    } else {
                        line_buffer.clear();
                        parser.reset();
                    }
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
        rl.save_history(&path_cache_history)?;
        Ok(())
    } else {
        let input = args[1..].join(" ");
        let mut parser = Parser::new();
        parser.extend(lexer.parse(input.as_str()));
        if let Some(node) = parser.parse() {
            let eval = node.eval();
            match eval {
                Ok(res) => println!("{res}"),
                Err(error) => println!("{error}"),
            }
        }
        Ok(())
    }
}
