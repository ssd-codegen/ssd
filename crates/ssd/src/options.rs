use clap::{Parser, ValueEnum};
use clap_complete::Shell;

use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct PrettyData {
    #[clap(short, long)]
    /// if true, write directly into file
    pub in_place: bool,

    #[clap(flatten)]
    pub input: BaseInputData,
}

#[derive(Debug, Parser)]
pub struct BaseInputData {
    #[clap(long)]
    /// do not use type mappings
    pub no_map: bool,
    #[clap(long = "tm", long)]
    /// A file containing type mappings.
    ///
    /// If a file with the same name as the script file, but with the extension tym, it
    /// will be used automatically.
    /// e.g.: If there is a file `/generator/script.rhai` and a corresponding
    /// `/generator/script.tym`, it will get used automatically.
    pub typemap: Option<PathBuf>,
    #[clap(short, long)]
    /// use raw data file as input instead of the ssd data format
    pub raw: bool,
    /// which file to use.
    pub file: PathBuf,
}

#[derive(Debug, Parser)]
pub struct BaseOutputData {
    #[clap(long, short)]
    /// The file which should get written with the output from the generator.
    pub out: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum DataFormat {
    Json,
    JsonPretty,
    Yaml,
    Toml,
    TomlPretty,
    #[cfg(feature = "ron")]
    /// only available with feature "ron" enabled
    Ron,
    #[cfg(feature = "ron")]
    /// only available with feature "ron" enabled
    RonPretty,
    Rsn,
    RsnPretty,
}

#[derive(Debug, Parser)]
pub struct DataParameters {
    /// The output format that should be used
    pub format: DataFormat,
    #[clap(flatten)]
    pub input: BaseInputData,
    #[clap(flatten)]
    pub out: BaseOutputData,
}

#[derive(Debug, Parser)]
pub enum Generator {
    /// Use a rhai based generator.
    #[cfg(feature = "rhai")]
    Rhai(crate::generators::rhai::Parameters),
    /// Use a handlebars based template.
    /// https://handlebarsjs.com/
    #[cfg(feature = "handlebars")]
    #[clap(aliases=["hbs"])]
    Handlebars(crate::generators::handlebars::Parameters),
    /// Use a tera based template.
    /// https://tera.netlify.app/
    #[cfg(feature = "tera")]
    Tera(crate::generators::tera::Parameters),
    /// Use a wasm based generator
    #[cfg(feature = "wasm")]
    Wasm(crate::generators::wasm::Parameters),
    /// Output as serialized data for external use
    Data(DataParameters),
}

type KV = (String, String);
#[allow(clippy::unnecessary_wraps)]
fn parse_key_val(env: &str) -> anyhow::Result<KV> {
    if let Some((var, value)) = env.split_once('=') {
        Ok((var.to_owned(), value.to_owned()))
    } else {
        Ok((env.to_owned(), String::new()))
    }
}

#[derive(Debug, Parser)]
#[clap(name = "ssd", about = "Simple Service Description")]
pub struct Args {
    #[arg(global=true, num_args(0..))]
    #[clap(short = 'D', value_parser = parse_key_val, required = false)]
    pub defines: Vec<(String, String)>,
    #[clap(subcommand)]
    pub command: SubCommand,
}

#[derive(Debug, Parser)]
pub enum SubCommand {
    /// Print debug representation of the parsed file.
    Debug(BaseInputData),
    /// Pretty print the parsed file.
    Pretty(PrettyData),
    /// Generate source code.
    #[command(subcommand)]
    Generate(Generator),
    /// Write language server file.
    #[clap(hide = true)]
    #[cfg(feature = "rhai")]
    LanguageServer { out: PathBuf },
    /// Print shell completions.
    #[clap(hide = true)]
    Completions { shell: Shell },
}
