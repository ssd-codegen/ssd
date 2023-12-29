mod ast;
mod generators;
mod map_vec;
mod options;
mod parser;
mod pretty;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use generators::rhai::build_engine;
use options::{Args, DataFormat, DataParameters, Generator, PrettyData};
#[cfg(feature = "ron")]
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

use parser::parse_file;
use ssd_data::TypeName;

use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

use crate::ast::ComparableAstElement;
use crate::options::SubCommand;
use crate::parser::{parse_file_raw, parse_raw};
use crate::pretty::pretty;

use crate::ast::{Namespace, SsdModule};

#[derive(Serialize, Deserialize, Debug)]
struct RawModel {
    raw: serde_value::Value,
    defines: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SsdModel {
    module: SsdModule,
    defines: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(untagged)]
enum StringOrVec {
    String(String),
    Vec(Vec<String>),
}

fn update_types(
    mut module: SsdModule,
    no_map: bool,
    typemap: Option<PathBuf>,
    script: Option<&PathBuf>,
) -> anyhow::Result<SsdModule> {
    if let (false, Some(map_file)) = (
        no_map,
        typemap.or_else(|| {
            script.and_then(|script| {
                let mut typemap = script.clone();
                typemap.set_extension("tym");
                typemap.exists().then_some(typemap)
            })
        }),
    ) {
        let mappings: HashMap<StringOrVec, StringOrVec> =
            toml::from_str(&std::fs::read_to_string(map_file)?)?;
        let mappings: HashMap<String, String> = mappings
            .iter()
            .map(|(k, v)| match (k, v) {
                (StringOrVec::Vec(k), StringOrVec::Vec(v)) => (k.join("::"), v.join("::")),
                (StringOrVec::Vec(k), StringOrVec::String(v)) => (k.join("::"), v.clone()),
                (StringOrVec::String(k), StringOrVec::Vec(v)) => (k.clone(), v.join("::")),
                (StringOrVec::String(k), StringOrVec::String(v)) => (k.clone(), v.clone()),
            })
            .collect();
        for (_dt_name, dt) in &mut module.data_types {
            for (_name, prop) in &mut dt.properties {
                let name = prop.typ.to_string();
                if let Some(v) = mappings.get(&name) {
                    prop.typ = Namespace::new(v);
                }
            }
        }

        for (_service_name, service) in &mut module.services {
            for (_handler_name, h) in &mut service.functions {
                if let Some(TypeName {
                    typ,
                    is_list,
                    count,
                    attributes,
                    comments,
                }) = &h.return_type
                {
                    let name = typ.to_string();
                    let mut comments = comments.clone();
                    if let Some(v) = mappings.get(&name) {
                        h.return_type = Some(
                            TypeName::new(Namespace::new(v), *is_list, *count, attributes.clone())
                                .with_comments(&mut comments),
                        );
                    }
                }
                for (_arg_name, arg) in &mut h.arguments {
                    let name = arg.typ.to_string();
                    if let Some(v) = mappings.get(&name) {
                        arg.typ = Namespace::new(v);
                    }
                }
            }
            for (_event_name, h) in &mut service.events {
                for (_arg_name, arg) in &mut h.arguments {
                    let name = arg.typ.to_string();
                    if let Some(v) = mappings.get(&name) {
                        arg.typ = Namespace::new(v);
                    }
                }
            }
        }
    }

    Ok(module)
}

fn print_or_write(out: Option<PathBuf>, result: &str) -> anyhow::Result<()> {
    if let Some(out) = out {
        std::fs::write(out, result)?;
    } else {
        println!("{result}");
    }
    Ok(())
}

fn parse_raw_data(file: PathBuf) -> anyhow::Result<serde_value::Value> {
    let content = std::fs::read_to_string(file)?;
    let result = serde_json::from_str(&content)
        .or_else(|_| toml::from_str(&content))
        .or_else(|_| serde_yaml::from_str(&content))
        .or_else(|_| rsn::from_str(&content));
    #[cfg(feature = "ron")]
    let result = result.or_else(|_| ron::from_str(&content));
    Ok(result?)
}

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
        let raw = parse_raw_data(input.file)?;
        serialize(format, raw)?
    } else {
        let module = parse_file(base, &input.file)?;
        let module = update_types(module, input.no_map, input.typemap, None)?;
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
