use structopt::StructOpt;

use std::path::PathBuf;

#[derive(Debug, StructOpt)]
pub struct Options {
    /// which file to use
    pub file: PathBuf,

    #[structopt(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    Debug,
    Pretty,
}
