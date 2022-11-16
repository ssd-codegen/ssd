use structopt::StructOpt;

use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(about = "Simple Service Description & Code Generator")]
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
    /// Print debug representation of the parsed file
    Debug(BaseData),
    /// Pretty print the parsed file
    Pretty(BaseData),
    /// Use a generator with the parsed file
    Generate(GeneratorOptions),
}

#[derive(Debug, StructOpt)]
pub struct GeneratorOptions {
    /// The script to use to generate the file
    pub script: PathBuf,
    #[structopt(flatten)]
    pub base: BaseData,
    #[structopt(long)]
    /// do not use type mappings
    pub no_map: bool,
    #[structopt(long = "tm", long)]
    /// A file containing type mappings.
    ///
    /// If a file with the same name as the script file, but with the extension tym, it
    /// will be used automatically.
    /// e.g.: If there is a file `/generator/script.rhai` and a corresponding
    /// `/generator/script.tym`, it will get used automatically
    pub typemap: Option<PathBuf>,
    #[structopt(long, short)]
    /// The file which should get written with the output from the generator
    pub out: Option<PathBuf>,
    #[structopt(long, short)]
    /// Enables debug mode (print and debug function in the script)
    pub debug: bool,
}
