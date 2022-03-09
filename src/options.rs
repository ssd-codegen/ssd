use structopt::StructOpt;

use std::path::PathBuf;

#[derive(Debug, StructOpt)]
pub struct Options {
    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(Debug, StructOpt)]
pub struct BaseData {
    /// which file to use
    pub file: PathBuf,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    Debug(BaseData),
    Pretty(BaseData),
    Generate(GeneratorOptions),
}

#[derive(Debug, StructOpt)]
pub struct GeneratorOptions {
    pub script: PathBuf,
    #[structopt(flatten)]
    pub base: BaseData,
    #[structopt(long, short)]
    pub out: Option<PathBuf>,
    #[structopt(long, short)]
    pub debug: bool,
}
