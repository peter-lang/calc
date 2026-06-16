use std::env;

use crate::error::CalcError;

mod config;
mod currency;
mod debug;
mod error;
mod files;
mod node;
mod number;
mod number_op;
mod parser;
mod rational;
mod repl;
mod unit;
mod value;
mod value_op;

fn main() -> Result<(), CalcError> {
    config::init()?;
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        repl::run()
    } else {
        repl::run_once(&args[1..].join(" "))
    }
}
