use std::{io::Write, num::ParseIntError, path::Path};

use once_cell::sync::Lazy;
use pest::{
    iterators::{Pair, Pairs},
    Parser, Span,
};
use pest_derive::Parser;
use regex::Regex;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::ast::{
    Attribute, DataType, Dependency, Enum, EnumValue, Event, Function, Import, Namespace,
    OrderedMap, Service, SsdModule, TypeName,
};

use crate::ast::{AstElement, ServiceAstElement};

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
            span: format!("{span:?}"),
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
                write!(f, "Other({inner})")
            }
        }
    }
}

impl ParseError {
    fn from_dyn_error<T: std::error::Error>(err: T) -> Self {
        ParseError {
            error_type: ParseErrorType::OtherError(format!("{err}")),
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

fn parse_type(typ: &str) -> (&str, bool, Option<usize>) {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+)\s+of").unwrap());
    if let Some(stripped) = typ.strip_prefix("list of") {
        (stripped.trim(), true, None)
    } else if let Some(cap) = RE.captures(typ) {
        let count_str = cap.get(1).unwrap().as_str();
        let count = count_str.parse::<usize>().unwrap();
        (typ[count_str.len() + 3..].trim(), true, Some(count))
    } else {
        (typ, false, None)
    }
}

#[allow(clippy::too_many_lines)]
pub fn parse_raw(content: &str) -> Result<Vec<AstElement>, ParseError> {
    use ParseErrorType::{
        IncompleteArgumentIdent, IncompleteCall, IncompleteDatatype, IncompleteDepends,
        IncompleteEnum, IncompleteEnumValue, IncompleteEvent, IncompleteImport, IncompleteProperty,
        IncompleteService, InvalidEnumValue, MissingType, UnexpectedElement,
    };
    let pairs = FileParser::parse(Rule::file, content).map_err(ParseError::from_dyn_error)?;
    let mut result = Vec::new();

    for p in pairs {
        match p.as_rule() {
            Rule::import => {
                let span = p.as_span();
                let mut p = p.into_inner();
                let n = p
                    .next()
                    .ok_or_else(|| ParseError::new(IncompleteImport, span))?;
                let (name, attributes) = parse_name(&mut p, n)?;
                result.push(AstElement::Import(Import::new(
                    Namespace::new(&name),
                    attributes,
                )));
            }
            Rule::data => {
                let span = p.as_span();
                let mut p = p.into_inner();
                let n = p
                    .next()
                    .ok_or_else(|| ParseError::new(IncompleteDatatype, span))?;
                let (name, attributes) = parse_name(&mut p, n)?;

                let mut properties = OrderedMap::new();
                let mut comments = Vec::new();

                for p in p {
                    if let Rule::COMMENT = p.as_rule() {
                        comments.push(p.as_span().as_str()[3..].trim().to_string());
                        continue;
                    }
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
                    let (typ, is_list, count) = parse_type(typ.as_str());
                    properties.push((
                        name,
                        TypeName::new(Namespace::new(typ), is_list, count, attributes)
                            .with_comments(&mut comments),
                    ));
                    // properties.insert(
                    //     name,
                    //     TypeName::new(Namespace::new(&typ), attributes)
                    //         .with_comments(&mut comments),
                    // );
                }

                result.push(AstElement::DataType((
                    name,
                    DataType::new(properties, attributes),
                )));
            }
            Rule::enum_ => {
                let span = p.as_span();
                let mut p = p.into_inner();
                let n = p
                    .next()
                    .ok_or_else(|| ParseError::new(IncompleteEnum, span))?;
                let (name, attributes) = parse_name(&mut p, n)?;

                let mut values = OrderedMap::new();

                let mut comments = Vec::new();
                for p in p {
                    if let Rule::COMMENT = p.as_rule() {
                        comments.push(p.as_span().as_str()[3..].trim().to_string());
                        continue;
                    }
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
                    values.push((
                        name,
                        EnumValue::new(value, attributes).with_comments(&mut comments),
                    ));
                    // values.insert(
                    //     name,
                    //     EnumValue::new(value, attributes).with_comments(&mut comments),
                    // );
                }

                result.push(AstElement::Enum((name, Enum::new(values, attributes))));
            }
            Rule::service => {
                let span = p.as_span();
                let mut p = p.into_inner();
                let n = p
                    .next()
                    .ok_or_else(|| ParseError::new(IncompleteService, span))?;
                let (service_name, attributes) = parse_name(&mut p, n)?;

                let mut service_parts = Vec::new();

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
                            service_parts.push(ServiceAstElement::Dependency(Dependency::new(
                                Namespace::new(&name),
                                attributes,
                            )));
                        }
                        Rule::function | Rule::handler => {
                            if rule == Rule::handler {
                                const DEPRECATED: &str =  "Using 'handlers' is deprecated and will be removed in future versions. Use 'fn' instead.";
                                let mut stderr = StandardStream::stderr(ColorChoice::Always);
                                if stderr
                                    .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))
                                    .is_ok()
                                {
                                    writeln!(&mut stderr, "{DEPRECATED}").unwrap();

                                    let _ = stderr.set_color(&ColorSpec::default());
                                } else {
                                    eprintln!("{DEPRECATED}");
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
                                                    let (typ, is_list, count) = parse_type(typ.as_str());
                                                    arguments.push((name, TypeName::new(Namespace::new(typ), is_list, count, attributes.clone())));
                                                    // arguments.insert(name, TypeName::new(Namespace::new(&typ), attributes.clone()));
                                                    attributes.clear();
                                                }
                                                Rule::attributes => {
                                                    attributes = parse_attributes(n)?;
                                                }
                                                _ => Err(ParseError::new(
                                                    UnexpectedElement(format!(
                                                        "while parsing argument for call \"{call_name}\" in service \"{service_name}\"! {p}"
                                                    )),
                                                    span,
                                                ))?,
                                            }
                                        }
                                    }
                                    Rule::typ => {
                                        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());
                                        let typ = RE.replace_all(p.as_str(), " ");
                                        let (typ, is_list, count) = parse_type(&typ);
                                        return_type = Some(TypeName::new(
                                            Namespace::new(typ),
                                            is_list,
                                            count,
                                            Vec::new(),
                                        ));
                                    }
                                    _ => Err(ParseError::new(
                                        UnexpectedElement(format!(
                                            "while parsing call \"{call_name}\" in service \"{service_name}\"! {p}"
                                        )),
                                        p.as_span(),
                                    ))?,
                                }
                            }

                            if let Some(p) = p.next() {
                                if p.as_rule() == Rule::typ {
                                    let (typ, is_list, count) = parse_type(p.as_str());
                                    return_type = Some(TypeName::new(
                                        Namespace::new(typ),
                                        is_list,
                                        count,
                                        Vec::new(),
                                    ));
                                } else {
                                    Err(ParseError::new(
                                        UnexpectedElement(format!(
                                            "while parsing return type for call \"{call_name}\" in service \"{service_name}\"! {p}"
                                        )),
                                        p.as_span(),
                                    ))?;
                                }
                            }
                            service_parts.push(ServiceAstElement::Function((
                                call_name,
                                Function::new(arguments, return_type, call_attributes),
                            )));
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
                                                    let (typ, is_list, count) = parse_type(typ.as_str());
                                                    arguments.push((name, TypeName::new(Namespace::new(typ), is_list, count, attributes.clone())));
                                                    // arguments.insert(name, TypeName::new(Namespace::new(&typ), attributes.clone()));
                                                    attributes.clear();
                                                }
                                                Rule::attributes => {
                                                    attributes = parse_attributes(n)?;
                                                }
                                                _ => Err(ParseError::new(
                                                    UnexpectedElement(format!(
                                                        "while parsing argument for event \"{event_name}\" in service \"{service_name}\"! {p}"
                                                    )),
                                                    span,
                                                ))?,
                                            }
                                        }
                                    }
                                    _ => Err(ParseError::new(
                                        UnexpectedElement(format!(
                                            "while parsing event \"{event_name}\" in service \"{service_name}\"! {p}"
                                        )),
                                        p.as_span(),
                                    ))?,
                                }
                            }

                            service_parts.push(ServiceAstElement::Event((
                                event_name,
                                Event::new(arguments, event_attributes),
                            )));
                        }
                        Rule::COMMENT => service_parts.push(ServiceAstElement::Comment(
                            p.as_span().as_str()[3..].trim().to_string(),
                        )),
                        _ => Err(ParseError::new(
                            UnexpectedElement(format!(
                                "while parsing service \"{service_name}\"! {p}"
                            )),
                            p.as_span(),
                        ))?,
                    }
                }

                result.push(AstElement::Service((
                    service_name,
                    service_parts,
                    attributes,
                )));
            }
            Rule::EOI => {}
            Rule::COMMENT => {
                let span = p.as_span();
                result.push(AstElement::Comment(span.as_str()[3..].trim().to_string()));
            }
            _ => Err(ParseError::new(
                UnexpectedElement(format!("{p}")),
                p.as_span(),
            ))?,
        }
    }

    Ok(result)
}

#[allow(unused)]
pub fn parse(content: &str, namespace: Namespace) -> Result<SsdModule, ParseError> {
    let raw = parse_raw(content)?;
    Ok(raw_to_ssd_file(namespace, &raw))
}

pub(crate) fn raw_service_to_service(
    raw: &[ServiceAstElement],
    attributes: &[Attribute],
) -> Service {
    let mut dependencies = Vec::new();
    let mut functions = OrderedMap::new();
    let mut events = OrderedMap::new();

    let mut comments = Vec::new();
    for element in raw {
        match element {
            ServiceAstElement::Dependency(import) => {
                dependencies.push(import.clone().with_comments(&mut comments));
            }
            ServiceAstElement::Function((key, value)) => {
                assert!(
                    !functions.iter().any(|(name, _)| name == key),
                    "Duplicate function {key}!"
                );
                functions.push((key.clone(), value.clone().with_comments(&mut comments)));
                // assert!(
                //     functions
                //         .insert(key.clone(), value.clone().with_comments(&mut comments))
                //         .is_none(),
                //     "Duplicate function {key}!"
                // );
            }
            ServiceAstElement::Event((key, value)) => {
                assert!(
                    !events.iter().any(|(name, _)| name == key),
                    "Duplicate event {key}!"
                );
                events.push((key.clone(), value.clone().with_comments(&mut comments)));
                // assert!(
                //     events
                //         .insert(key.clone(), value.clone().with_comments(&mut comments))
                //         .is_none(),
                //     "Duplicate event {key}!"
                // );
            }
            ServiceAstElement::Comment(c) => comments.push(c.to_string()),
        }
    }

    Service::new(dependencies, functions, events, attributes.into())
}

pub(crate) fn raw_to_ssd_file(namespace: Namespace, raw: &[AstElement]) -> SsdModule {
    let mut imports = Vec::new();
    let mut datatypes = OrderedMap::new();
    let mut enums = OrderedMap::new();
    let mut services = OrderedMap::new();

    for element in raw {
        match element {
            AstElement::Import(import) => imports.push(import.clone()),
            AstElement::DataType((key, value)) => {
                assert!(
                    !datatypes.iter().any(|(name, _)| name == key),
                    "Duplicate datatype {key}!"
                );
                datatypes.push((key.clone(), value.clone()));
                // assert!(
                //     datatypes.insert(key.clone(), value.clone()).is_none(),
                //     "Duplicate datatype {key}!"
                // );
            }
            AstElement::Enum((key, value)) => {
                assert!(
                    !enums.iter().any(|(name, _)| name == key),
                    "Duplicate enum {key}!"
                );
                enums.push((key.clone(), value.clone()));
                // assert!(
                //     enums.insert(key.clone(), value.clone()).is_none(),
                //     "Duplicate enum {key}!"
                // );
            }

            AstElement::Service((key, value, attributes)) => {
                assert!(
                    !services.iter().any(|(name, _)| name == key),
                    "Duplicate service {key}!"
                );
                services.push((key.clone(), raw_service_to_service(value, attributes)));
                // assert!(
                //     services.insert(key.clone(), raw_service_to_service(value, attributes)).is_none(),
                //     "Duplicate service {key}!"
                // );
            }
            AstElement::Comment(_) => (),
        }
    }

    SsdModule::new(namespace, imports, datatypes, enums, services)
}

pub fn parse_file_raw<P: AsRef<Path>>(path: P) -> Result<Vec<AstElement>, ParseError> {
    let content = std::fs::read_to_string(path).map_err(ParseError::from_dyn_error)?;

    parse_raw(&content)
}

/// Parses the given file and returns the corresponding `SsdModule`.
///
/// The namespace of the file is taken from the file's path, with the base directory removed.
///
/// # Arguments
///
/// * `base` - The base path of the file.
/// * `path` - The path to the file to parse.
pub fn parse_file<P: AsRef<Path>>(base: &P, path: &P) -> Result<SsdModule, ParseError> {
    let base = base.as_ref();
    let path = path.as_ref();
    let mut components = if path.starts_with(base) {
        path.strip_prefix(base)
            .map_err(ParseError::from_dyn_error)?
            .to_owned()
    } else {
        path.to_owned()
    };

    components.set_extension("");
    let components = components
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();

    parse_file_with_namespace(path, Namespace::from_vec(components))
}

#[allow(unused)]
pub fn parse_file_with_namespace<P: AsRef<Path>>(
    path: P,
    namespace: Namespace,
) -> Result<SsdModule, ParseError> {
    let raw = parse_file_raw(path)?;

    Ok(raw_to_ssd_file(namespace, &raw))
}

#[test]
fn test_simple() {
    insta::assert_json_snapshot!(parse(
        include_str!("../data/test.svc"),
        Namespace::new("__test__")
    )
    .unwrap());
}

#[test]
fn test_raw() {
    insta::assert_json_snapshot!(parse_raw(include_str!("../data/test.svc"),).unwrap());
}
