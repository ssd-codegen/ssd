mod ast;
mod generators;
mod helper;
mod map_vec;
#[cfg(feature = "_bin")]
mod options;
mod parser;
mod pretty;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use generators::rhai::build_engine;
use options::{Args, DataFormat, DataParameters, Generator, PrettyData};
#[cfg(feature = "ron")]
use ron::ser::PrettyConfig;
use serde::Serialize;

use parser::parse_file;

use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

use crate::ast::ComparableAstElement;
use crate::helper::parse_raw_data;
use crate::helper::print_or_write;
use crate::options::SubCommand;
use crate::parser::{parse_file_raw, parse_raw};
use crate::pretty::pretty;
use crate::helper::update_types_from_file;

fn serialize<T: Serialize>(format: DataFormat, value: T) -> anyhow::Result<String> {
    let result = match format {
        options::DataFormat::Json => serde_json::to_string(&value)?,
        options::DataFormat::JsonPretty => serde_json::to_string_pretty(&value)?,
        options::DataFormat::Yaml => serde_yaml::to_string(&value)?,
        options::DataFormat::Toml => toml::to_string(&value)?,
        options::DataFormat::TomlPretty => toml::to_string_pretty(&value)?,
        #[cfg(feature = "ron")]
        options::DataFormat::Ron => ron::to_string(&value)?,
        #[cfg(feature = "ron")]
        options::DataFormat::RonPretty => {
            ron::ser::to_string_pretty(&value, PrettyConfig::default())?
        }
        options::DataFormat::Rsn => rsn::to_string(&value),
        options::DataFormat::RsnPretty => rsn::to_string_pretty(&value),
    };
    Ok(result)
}

fn generate_data(
    base: &PathBuf,
    DataParameters { format, input, out }: DataParameters,
) -> Result<(), Box<dyn Error>> {
    let result = if input.raw {
        let raw = crate::parse_raw_data(input.file)?;
        serialize(format, raw)?
    } else {
        let module = parse_file(base, &input.file)?;
        let module = update_types_from_file(module, input.no_map, input.typemap, None)?;
        serialize(format, module)?
    };

    print_or_write(out.out, &result)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Args::parse();

    let base = std::fs::canonicalize(
        shellexpand::full(std::env::current_dir()?.to_str().unwrap())?.to_string(),
    )?;
    let defines: HashMap<String, String> = cli.defines.into_iter().collect();
    match cli.command {
        SubCommand::Debug(data) => {
            let path =
                std::fs::canonicalize(shellexpand::full(data.file.to_str().unwrap())?.to_string())?;

            match parse_file(&base, &path) {
                Ok(ns) => println!("{ns:#?}"),
                Err(e) => eprintln!("{e}"),
            }
        }

        SubCommand::Pretty(PrettyData { in_place, input }) => {
            let raw = parse_file_raw(&input.file)?;
            let pretty = pretty(&raw);
            let pretty_raw = parse_raw(&pretty)?;
            assert_eq!(
                raw.iter()
                    .map(ComparableAstElement::from)
                    .collect::<Vec<_>>(),
                pretty_raw
                    .iter()
                    .map(ComparableAstElement::from)
                    .collect::<Vec<_>>(),
            );
            if in_place {
                std::fs::write(input.file, pretty)?;
            } else {
                println!("{pretty}");
            }
        }

        SubCommand::Completions { shell } => {
            let mut cli = Args::command();
            let name = cli.get_name().to_string();
            generate(shell, &mut cli, name, &mut std::io::stdout());
        }

        #[cfg(feature = "rhai")]
        SubCommand::LanguageServer { out } => {
            use std::{cell::RefCell, rc::Rc};
            let messages = Rc::new(RefCell::new(Vec::new()));

            let engine = build_engine(messages.clone(), false);
            engine.definitions().write_to_file(out).unwrap();
        }

        SubCommand::Generate(generator) => match generator {
            #[cfg(feature = "handlebars")]
            Generator::Handlebars(params) => {
                crate::generators::handlebars::generate(&base, defines, params)?;
            }

            #[cfg(feature = "tera")]
            Generator::Tera(params) => {
                crate::generators::tera::generate(&base, defines, params)?;
            }

            #[cfg(feature = "rhai")]
            Generator::Rhai(params) => {
                crate::generators::rhai::generate(&base, defines, params)?;
            }

            Generator::Data(params) => {
                generate_data(&base, params)?;
            }

            #[cfg(feature = "wasm")]
            Generator::Wasm(params) => {
                crate::generators::wasm::generate(&base, defines, params)?;
            }
        },
    };

    Ok(())
}
