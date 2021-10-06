#[macro_use]
extern crate lalrpop_util;

mod ast;
mod options;
mod map_vec;

use std::path::PathBuf;

use crate::options::{Command, Options};
use crate::ast::Namespace;
use structopt::StructOpt;

lalrpop_mod!(pub grammar);

fn parse_file(file: PathBuf) -> Result<Namespace, String> {
    let content = std::fs::read_to_string(file).map_err(|e| format!("{}", e))?;
    let ns = grammar::NamespaceParser::new().parse(&content).map_err(|e| format!("{}", e));
    Ok(ns?)
}

fn execute<S: Fn(Namespace)>(ns: Result<Namespace, String>, s: S) {
    match ns {
        Ok(ns) => s(ns),
        Err(e) => println!("{}", e),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Options { command } = Options::from_args();

    match command {
        Command::Debug(data) => execute(parse_file(data.file), |ns| println!("{:#?}", ns)),
        Command::Pretty(data) => execute(parse_file(data.file), |ns| println!("{}", ns.to_string())),
        Command::Generate(options) => println!("Generating is not supported: {:?}", options),
    };

    Ok(())
}
