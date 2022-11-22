use clap::Parser;
use clap_complete::Shell;

use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct BaseData {
    /// which file to use
    pub file: PathBuf,
}

#[derive(Debug, Parser)]
pub enum SubCommand {
    /// Print debug representation of the parsed file
    Debug(BaseData),
    // /// Pretty print the parsed file
    // Pretty(BaseData),
    /// Use a generator with the parsed file
    Generate(GeneratorParameters),
    /// Print metadata as json
    Metadata,
    /// Write language server file
    #[clap(hide = true)]
    LanguageServer { out: PathBuf },
    /// Print shell completions
    Completions { shell: Shell },
}

#[derive(Debug, Parser)]
pub struct GeneratorParameters {
    /// The script to use to generate the file
    pub script: PathBuf,
    #[clap(flatten)]
    pub base: BaseData,
    #[clap(long)]
    /// do not use type mappings
    pub no_map: bool,
    #[clap(long = "tm", long)]
    /// A file containing type mappings.
    ///
    /// If a file with the same name as the script file, but with the extension tym, it
    /// will be used automatically.
    /// e.g.: If there is a file `/generator/script.rhai` and a corresponding
    /// `/generator/script.tym`, it will get used automatically
    pub typemap: Option<PathBuf>,
    #[clap(long, short)]
    /// The file which should get written with the output from the generator
    pub out: Option<PathBuf>,
    #[clap(long, short)]
    /// Enables debug mode (print and debug function in the script)
    pub debug: bool,
}
