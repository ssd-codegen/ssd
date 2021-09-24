#[macro_use]
extern crate lalrpop_util;

mod ast;
mod options;

use crate::options::{Command, Options};
use structopt::StructOpt;

lalrpop_mod!(pub grammar);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Options { command, file } = Options::from_args();

    let content = std::fs::read_to_string(file)?;
    let res = grammar::NamespaceParser::new().parse(&content);

    match &res.map_err(|e| format!("{}", e)) {
        Ok(res) => match command.unwrap_or(Command::Pretty) {
            Command::Debug => println!("{:#?}", res),
            Command::Pretty => println!("{}", res.pretty()),
        },
        Err(e) => println!("{}", e),
    };

    Ok(())
}
