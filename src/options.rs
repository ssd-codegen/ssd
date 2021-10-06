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
    #[structopt(flatten)]
    pub base: BaseData,
    pub script: PathBuf,
    pub out: PathBuf,
}
