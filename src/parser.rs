use std::{io::Write, collections::BTreeMap, num::ParseIntError, path::PathBuf};

use pest::{
    iterators::{Pair, Pairs},
    Parser, Span,
};
use pest_derive::Parser;
use regex::Regex;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::ast::{
    Attribute, DataType, Dependency, Enum, EnumValue, Event, Function, Import, NameTypePair,
    Namespace, OrderedMap, Service, SsdcFile,
};

fn parse_attribute_arg(node: Pair<Rule>) -> Result<(String, Option<String>), ParseError> {
    let span = node.as_span();
    let mut p = node.into_inner();
    let name = p
        .next()
        .ok_or_else(|| ParseError::new(ParseErrorType::IncompleteAttributeArg, span))?
        .as_str()
        .to_string();
    let value = p.next().map(|p| p.into_inner().as_str().to_string());
    Ok((name, value))
}

fn parse_attribute(node: Pair<Rule>) -> Result<Attribute, ParseError> {
    let span = node.as_span();
    let mut p = node.into_inner();
    let name = p.next();
    let mut args = Vec::new();
    for p in p {
        args.push(parse_attribute_arg(p)?);
    }
    Ok(Attribute::new(
        Namespace::new(
            name.ok_or_else(|| ParseError::new(ParseErrorType::IncompleteAttribute, span))?
                .as_str(),
        ),
        args,
    ))
}

fn parse_attributes(node: Pair<Rule>) -> Result<Vec<Attribute>, ParseError> {
    node.into_inner().map(parse_attribute).collect()
}

fn parse_name(p: &mut Pairs<Rule>, n: Pair<Rule>) -> Result<(String, Vec<Attribute>), ParseError> {
    let span = n.as_span();
    if n.as_rule() == Rule::attributes {
        let attributes = parse_attributes(n)?;
        let name = p
            .next()
            .ok_or_else(|| ParseError::new(ParseErrorType::IncompleteName, span))?
            .as_str()
            .to_string();
        Ok((name, attributes))
    } else {
        let name = n.as_str().to_string();
        Ok((name, Vec::new()))
    }
}

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub(crate) struct FileParser;

#[derive(Debug)]
pub struct ParseError {
    pub error_type: ParseErrorType,
    pub span: String,
}

impl ParseError {
    fn new(error_type: ParseErrorType, span: Span) -> Self {
        Self {
            error_type,
            span: format!("{:?}", span),
        }
    }
}

#[derive(Debug)]
pub enum ParseErrorType {
    IncompleteImport,
    IncompleteDatatype,
    IncompleteProperty,
    MissingType(String),
    IncompleteEnum,
    IncompleteEnumValue,
    InvalidEnumValue(String),
    IncompleteService,
    IncompleteDepends,
    IncompleteCall,
    IncompleteEvent,
    IncompleteArgumentIdent,
    IncompleteAttributeArg,
    IncompleteAttribute,
    IncompleteName,
    UnexpectedElement(String),
    OtherError(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.error_type {
            ParseErrorType::IncompleteImport => write!(f, "Import incomplete. ({})", self.span),
            ParseErrorType::IncompleteDatatype => write!(f, "Datatype incomplete. ({})", self.span),
            ParseErrorType::IncompleteProperty => write!(f, "Property incomplete. ({})", self.span),
            ParseErrorType::MissingType(name) => {
                write!(f, "Type missing after {}. ({:?})", name, self.span)
            }
            ParseErrorType::IncompleteService => write!(f, "Service incomplete. ({})", self.span),
            ParseErrorType::IncompleteDepends => write!(f, "Depends incomplete. ({})", self.span),
            ParseErrorType::IncompleteCall => write!(f, "Call incomplete. ({})", self.span),
            ParseErrorType::IncompleteEvent => write!(f, "Event incomplete. ({})", self.span),
            ParseErrorType::IncompleteArgumentIdent => {
                write!(f, "Argument ident incomplete. ({})", self.span)
            }
            ParseErrorType::IncompleteAttributeArg => {
                write!(f, "Attribute argument incomplete. ({})", self.span)
            }
            ParseErrorType::IncompleteAttribute => {
                write!(f, "Attribute incomplete. ({})", self.span)
            }
            ParseErrorType::IncompleteName => {
                write!(f, "Name incomplete. ({})", self.span)
            }
            ParseErrorType::UnexpectedElement(info) => {
                write!(f, "Unexpected element {} ({})", info, self.span)
            }
            ParseErrorType::IncompleteEnum => write!(f, "Incomplete enum. ({})", self.span),
            ParseErrorType::IncompleteEnumValue => {
                write!(f, "Incomplete enum value. ({})", self.span)
            }
            ParseErrorType::InvalidEnumValue(info) => {
                write!(f, "Invalid enum value. {} ({})", info, self.span)
            }
            ParseErrorType::OtherError(inner) => {
                write!(f, "Other({})", inner)
            }
        }
    }
}

impl ParseError {
    fn from_dyn_error<T: std::error::Error>(err: T) -> Self {
        ParseError {
            error_type: ParseErrorType::OtherError(format!("{}", err)),
            span: String::new(),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

#[allow(clippy::too_many_lines)]
pub fn parse(content: &str, namespace: Namespace) -> Result<SsdcFile, ParseError> {
    use ParseErrorType::{
        IncompleteArgumentIdent, IncompleteCall, IncompleteDatatype, IncompleteDepends,
        IncompleteEnum, IncompleteEnumValue, IncompleteEvent, IncompleteImport, IncompleteProperty,
        IncompleteService, InvalidEnumValue, MissingType, UnexpectedElement,
    };
    let pairs = FileParser::parse(Rule::file, content).map_err(ParseError::from_dyn_error)?;
    let mut imports = Vec::new();
    let mut datatypes = OrderedMap::new();
    let mut enums = OrderedMap::new();
    let mut services = OrderedMap::new();

    for p in pairs {
        match p.as_rule() {
            Rule::import => {
                let span = p.as_span();
                let mut p = p.into_inner();
                let n = p
                    .next()
                    .ok_or_else(|| ParseError::new(IncompleteImport, span))?;
                let (name, attributes) = parse_name(&mut p, n)?;
                imports.push(Import::new(Namespace::new(&name), attributes));
            }
            Rule::data => {
                let span = p.as_span();
                let mut p = p.into_inner();
                let n = p
                    .next()
                    .ok_or_else(|| ParseError::new(IncompleteDatatype, span))?;
                let (name, attributes) = parse_name(&mut p, n)?;

                let mut properties = OrderedMap::new();

                for p in p {
                    let span = p.as_span();
                    let mut p = p.into_inner();
                    let n = p
                        .next()
                        .ok_or_else(|| ParseError::new(IncompleteProperty, span))?;
                    let (name, attributes) = parse_name(&mut p, n)?;
                    let typ = p
                        .next()
                        .ok_or_else(|| ParseError::new(MissingType(name.clone()), span))?
                        .as_str()
                        .to_string();
                    properties.insert(name, NameTypePair::new(Namespace::new(&typ), attributes));
                }

                datatypes.insert(name, DataType::new(properties, attributes));
            }
            Rule::enum_ => {
                let span = p.as_span();
                let mut p = p.into_inner();
                let n = p
                    .next()
                    .ok_or_else(|| ParseError::new(IncompleteEnum, span))?;
                let (name, attributes) = parse_name(&mut p, n)?;

                let mut values = BTreeMap::new();

                for p in p {
                    let span = p.as_span();
                    let mut p = p.into_inner();
                    let n = p
                        .next()
                        .ok_or_else(|| ParseError::new(IncompleteEnumValue, span))?;
                    let (name, attributes) = parse_name(&mut p, n)?;
                    let value = if let Some(v) = p.next() {
                        Some(v.as_str().parse().map_err(|err: ParseIntError| {
                            ParseError::new(InvalidEnumValue(err.to_string()), span)
                        })?)
                    } else {
                        None
                    };
                    values.insert(name, EnumValue::new(value, attributes));
                }

                enums.insert(name, Enum::new(values, attributes));
            }
            Rule::service => {
                let span = p.as_span();
                let mut p = p.into_inner();
                let n = p
                    .next()
                    .ok_or_else(|| ParseError::new(IncompleteService, span))?;
                let (service_name, attributes) = parse_name(&mut p, n)?;

                let mut dependencies = Vec::new();
                let mut calls = OrderedMap::new();
                let mut events = OrderedMap::new();

                for p in p {
                    let rule = p.as_rule();
                    match rule {
                        Rule::depends => {
                            let span = p.as_span();
                            let mut p = p.into_inner();
                            let n = p
                                .next()
                                .ok_or_else(|| ParseError::new(IncompleteDepends, span))?;
                            let (name, attributes) = parse_name(&mut p, n)?;
                            dependencies.push(Dependency::new(Namespace::new(&name), attributes));
                        }
                        Rule::function | Rule::handler => {
                            if rule == Rule::handler {
                                const DEPRECATED: &str =  "Using 'handles' is deprecated and will be removed in future versions. Use 'fn' instead.";
                                let mut stderr = StandardStream::stderr(ColorChoice::Always);
                                if stderr
                                    .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))
                                    .is_ok()
                                {
                                    writeln!(&mut stderr, "{}", DEPRECATED).unwrap();

                                    let _ = stderr.set_color(&ColorSpec::default());
                                } else {
                                    eprintln!("{}", DEPRECATED);
                                }
                            }
                            let span = p.as_span();
                            let mut p = p.into_inner();
                            let n = p
                                .next()
                                .ok_or_else(|| ParseError::new(IncompleteCall, span))?;
                            let (call_name, call_attributes) = parse_name(&mut p, n)?;
                            let mut arguments = OrderedMap::new();
                            let mut return_type = None;
                            let mut attributes = Vec::new();
                            for p in p.by_ref() {
                                match p.as_rule() {
                                    Rule::argument => {
                                        let span = p.as_span();
                                        let mut p = p.clone().into_inner();
                                        while let Some(n) = p.next() {
                                            match n.as_rule() {
                                                Rule::ident => {
                                                    let name = n.as_str().to_string();
                                                    let typ = p.next().ok_or_else(|| ParseError::new(IncompleteArgumentIdent, span))?.as_str().to_string();
                                                    arguments.insert(name, NameTypePair::new(Namespace::new(&typ), attributes.clone()));
                                                    attributes.clear();
                                                }
                                                Rule::attributes => {
                                                    attributes = parse_attributes(n)?;
                                                }
                                                _ => Err(ParseError::new(
                                                    UnexpectedElement(format!(
                                                        "while parsing argument for call \"{}\" in service \"{}\"! {}",
                                                        call_name,service_name, p
                                                    )),
                                                    span,
                                                ))?,
                                            }
                                        }
                                    }
                                    Rule::typ => {
                                        let re = Regex::new(r"\s+").expect("invalid regex");
                                        return_type =
                                            Some(Namespace::new(&re.replace_all(p.as_str(), " ")));
                                    }
                                    _ => Err(ParseError::new(
                                        UnexpectedElement(format!(
                                            "while parsing call \"{}\" in service \"{}\"! {}",
                                            call_name, service_name, p
                                        )),
                                        p.as_span(),
                                    ))?,
                                }
                            }

                            if let Some(p) = p.next() {
                                if p.as_rule() == Rule::typ {
                                    return_type = Some(Namespace::new(p.as_str()));
                                } else {
                                    Err(ParseError::new(
                                        UnexpectedElement(format!(
                                            "while parsing return type for call \"{}\" in service \"{}\"! {}",
                                            call_name,service_name, p
                                        )),
                                        p.as_span(),
                                    ))?;
                                }
                            }
                            calls.insert(
                                call_name,
                                Function::new(arguments, return_type, call_attributes),
                            );
                        }
                        Rule::event => {
                            let span = p.as_span();
                            let mut p = p.into_inner();
                            let n = p
                                .next()
                                .ok_or_else(|| ParseError::new(IncompleteEvent, span))?;
                            let (event_name, event_attributes) = parse_name(&mut p, n)?;
                            let mut arguments = OrderedMap::new();
                            let mut attributes = Vec::new();
                            for p in p.by_ref() {
                                match p.as_rule() {
                                    Rule::argument => {
                                        let span = p.as_span();
                                        let mut p = p.clone().into_inner();
                                        while let Some(n) = p.next() {
                                            match n.as_rule() {
                                                Rule::ident => {
                                                    let name = n.as_str().to_string();
                                                    let typ = p.next().ok_or_else(|| ParseError::new(IncompleteArgumentIdent, span))?.as_str().to_string();
                                                    arguments.insert(name, NameTypePair::new(Namespace::new(&typ), attributes.clone()));
                                                    attributes.clear();
                                                }
                                                Rule::attributes => {
                                                    attributes = parse_attributes(n)?;
                                                }
                                                _ => Err(ParseError::new(
                                                    UnexpectedElement(format!(
                                                        "while parsing argument for event \"{}\" in service \"{}\"! {}",
                                                        event_name,service_name, p
                                                    )),
                                                    span,
                                                ))?,
                                            }
                                        }
                                    }
                                    _ => Err(ParseError::new(
                                        UnexpectedElement(format!(
                                            "while parsing event \"{}\" in service \"{}\"! {}",
                                            event_name, service_name, p
                                        )),
                                        p.as_span(),
                                    ))?,
                                }
                            }

                            events.insert(event_name, Event::new(arguments, event_attributes));
                        }
                        _ => Err(ParseError::new(
                            UnexpectedElement(format!(
                                "while parsing service \"{}\"! {}",
                                service_name, p
                            )),
                            p.as_span(),
                        ))?,
                    }
                }

                services.insert(
                    service_name,
                    Service::new(dependencies, calls, events, attributes),
                );
            }
            Rule::EOI => {}
            _ => Err(ParseError::new(
                UnexpectedElement(format!("{}", p)),
                p.as_span(),
            ))?,
        }
    }

    Ok(SsdcFile::new(
        namespace, imports, datatypes, enums, services,
    ))
}

pub fn parse_file(base: &PathBuf, path: PathBuf) -> Result<SsdcFile, ParseError> {
    let content = std::fs::read_to_string(&path).map_err(ParseError::from_dyn_error)?;

    let mut path = if path.starts_with(base) {
        path.strip_prefix(base)
            .map_err(ParseError::from_dyn_error)?
            .to_owned()
    } else {
        path
    };

    path.set_extension("");
    let components = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();

    parse(&content, Namespace::from_vec(components))
}

#[test]
fn test_simple() {
    insta::assert_ron_snapshot!(parse(
        include_str!("../data/test.svc"),
        Namespace::new("__test__")
    )
    .unwrap());
}
