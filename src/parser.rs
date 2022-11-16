use std::path::PathBuf;

use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::Parser;
use regex::Regex;

use crate::ast::{
    Attribute, DataType, Dependency, Handler, Import, NameTypePair, Namespace, Service, SsdcFile,
};

fn parse_attribute_arg(node: Pair<Rule>) -> (String, Option<String>) {
    let mut p = node.into_inner();
    let name = p.next().unwrap().as_str().to_string();
    let value = p.next().map(|p| p.as_str().to_string());
    (name, value)
}

fn parse_attribute(node: Pair<Rule>) -> Attribute {
    let mut p = node.into_inner();
    let name = p.next();
    let mut args = Vec::new();
    for p in p {
        args.push(parse_attribute_arg(p));
    }
    Attribute::new(Namespace::new(name.unwrap().as_str()), args)
}

fn parse_attributes(node: Pair<Rule>) -> Vec<Attribute> {
    node.into_inner().map(parse_attribute).collect()
}

fn parse_name(p: &mut Pairs<Rule>, n: Pair<Rule>) -> (String, Vec<Attribute>) {
    if n.as_rule() == Rule::attributes {
        let attributes = parse_attributes(n);
        let name = p.next().unwrap().as_str().to_string();
        (name, attributes)
    } else {
        let name = n.as_str().to_string();
        (name, Vec::new())
    }
}

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub(crate) struct FileParser;

#[allow(clippy::too_many_lines)]
pub fn parse_file(base: &PathBuf, path: PathBuf) -> anyhow::Result<SsdcFile> {
    let file = std::fs::read_to_string(&path)?;

    let mut path = if path.starts_with(base) {
        path.strip_prefix(base)?.to_owned()
    } else {
        path
    };

    path.set_extension("");
    let components = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();

    let pairs = FileParser::parse(Rule::file, &file)?;
    let mut imports = Vec::new();
    let mut datatypes = Vec::new();
    let mut services = Vec::new();

    for p in pairs {
        match p.as_rule() {
            Rule::import => {
                let mut p = p.into_inner();
                let n = p.next().unwrap();
                let (name, attributes) = parse_name(&mut p, n);
                imports.push(Import::new(Namespace::new(&name), attributes));
            }
            Rule::data => {
                let mut p = p.into_inner();
                let n = p.next().unwrap();
                let (name, attributes) = parse_name(&mut p, n);

                let mut properties = Vec::new();

                for p in p {
                    let mut p = p.into_inner();
                    let n = p.next().unwrap();
                    let (name, attributes) = parse_name(&mut p, n);
                    let typ = p.next().unwrap().as_str().to_string();
                    properties.push(NameTypePair::new(name, Namespace::new(&typ), attributes));
                }

                datatypes.push(DataType::new(name, properties, attributes));
            }
            Rule::service => {
                let mut p = p.into_inner();
                let n = p.next().unwrap();
                let (service_name, attributes) = parse_name(&mut p, n);

                let mut dependencies = Vec::new();
                let mut handlers = Vec::new();

                for p in p {
                    match p.as_rule() {
                        Rule::depends => {
                            let mut p = p.into_inner();
                            let n = p.next().unwrap();
                            let (name, attributes) = parse_name(&mut p, n);
                            dependencies.push(Dependency::new(Namespace::new(&name), attributes));
                        }
                        Rule::handler => {
                            let mut p = p.into_inner();
                            let n = p.next().unwrap();
                            let (handler_name, handler_attributes) = parse_name(&mut p, n);
                            let mut arguments = Vec::new();
                            let mut return_type = None;
                            let mut attributes = Vec::new();
                            for p in p.by_ref() {
                                match p.as_rule() {
                                    Rule::argument => {
                                        let mut p = p.clone().into_inner();
                                        while let Some(n) = p.next() {
                                            match n.as_rule() {
                                                Rule::ident => {
                                                    let name = n.as_str().to_string();
                                                    let typ = p.next().unwrap().as_str().to_string();
                                                    arguments.push(NameTypePair::new(name, Namespace::new(&typ), attributes.clone()));
                                                    attributes.clear();
                                                }
                                                Rule::attributes => {
                                                    attributes = parse_attributes(n);
                                                }
                                                _ => anyhow::bail!(
                                                    "Unexpected element while parsing argument for handler \"{}\" in service \"{}\"! {}",
                                                    handler_name,
                                                    service_name,
                                                    p
                                                ),
                                            }
                                        }                                    },
                                    Rule::typ => {
                                        let re = Regex::new(r"\s+").unwrap();
                                        return_type = Some(Namespace::new(&re.replace_all(p.as_str(), " ")));
                                    }
                                    _ => anyhow::bail!(
                                        "Unexpected element while parsing handler \"{}\" in service \"{}\"! {}",
                                        handler_name,
                                        service_name,
                                        p
                                    ),
                                }
                            }

                            if let Some(p) = p.next() {
                                if p.as_rule() == Rule::typ {
                                    return_type = Some(Namespace::new(p.as_str()));
                                } else {
                                    anyhow::bail!(
                                        "Unexpected element while parsing return type for handler \"{}\" in service \"{}\"! {}",
                                        handler_name,
                                        service_name,
                                        p
                                    );
                                }
                            }
                            handlers.push(Handler::new(
                                handler_name,
                                arguments,
                                return_type,
                                handler_attributes,
                            ));
                        }
                        _ => anyhow::bail!(
                            "Unexpected element while parsing service \"{}\"! {}",
                            service_name,
                            p
                        ),
                    }
                }

                services.push(Service::new(
                    service_name,
                    dependencies,
                    handlers,
                    attributes,
                ));
            }
            Rule::EOI => {}
            _ => anyhow::bail!("Unexpected element! {}", p),
        }
    }

    Ok(SsdcFile::new(
        Namespace::from_vec(components),
        imports,
        datatypes,
        services,
    ))
}
