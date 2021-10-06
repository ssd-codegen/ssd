#![allow(dead_code)]

use std::fmt::{Debug, Formatter};
use crate::map_vec::MapVec;

#[derive(Debug, Clone)]
pub struct Namespace {
    name: String,
    imports: Vec<String>,
    data_types: Vec<DataType>,
    services: Vec<Service>,
}

impl Namespace {
    pub fn new<I, DT, S>(
        name: String,
        imports: Vec<I>,
        data_types: Vec<DT>,
        services: Vec<S>,
    ) -> Self
        where
            I: Into<String>,
            DT: Into<DataType>,
            S: Into<Service>,
    {
        Self {
            name,
            imports: imports.map_vec(),
            data_types: data_types.map_vec(),
            services: services.map_vec(),
        }
    }
}

impl ToString for Namespace {
    fn to_string(&self) -> String {
        format!(
            "namespace {};\n\n{}\n\n{}\n\n{}",
            self.name,
            self.imports.iter().map(|i| format!("{};", i.to_string())).collect::<Vec<_>>().join("\n"),
            self.data_types.iter().map(|d| format!("{}", d.to_string())).collect::<Vec<_>>().join("\n"),
            self.services.iter().map(|s| format!("{}", s.to_string())).collect::<Vec<_>>().join("\n"),
        )
    }
}

#[derive(Debug)]
pub struct Attributed<T> {
    value: T,
    attributes: Vec<Attribute>,
    prefix: Option<String>,
}

impl<T> Attributed<T> {
    pub fn new(value: T, attributes: Vec<Attribute>, prefix: Option<String>) -> Self {
        Self {
            value, attributes, prefix,
        }
    }
}

impl<T> From<T> for Attributed<T> {
    fn from(value: T) -> Self {
        Self {
            value,
            attributes: Vec::new(),
            prefix: None,
        }
    }
}

pub trait IsAttributed {
    const USE_NEWLINES: bool = true;
}

impl IsAttributed for String {
    const USE_NEWLINES: bool = true;
}

impl IsAttributed for Handler {
    const USE_NEWLINES: bool = true;
}

impl IsAttributed for Service {
    const USE_NEWLINES: bool = true;
}

impl IsAttributed for DataType {
    const USE_NEWLINES: bool = true;
}

impl IsAttributed for NameTypePair {
    const USE_NEWLINES: bool = false;
}

impl<T: ToString + IsAttributed> ToString for Attributed<T> {
    fn to_string(&self) -> String {
        let nl = if T::USE_NEWLINES { "\n" } else { " " };
        let prefix = self.prefix.as_ref().map(|s| format!("{} ", s)).unwrap_or("".to_string());
        let attributes = if !self.attributes.is_empty() {
            format!("[{}]{}", self.attributes.iter().map(ToString::to_string).collect::<Vec<_>>().join(","), nl)
        } else {
            "".to_string()
        };
        format!("{}{}{}", attributes, prefix, self.value.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    name: String,
    parameters: Vec<String>,
}

impl Attribute {
    pub fn new(name: String, parameters: Vec<String>) -> Self {
        Self { name, parameters }
    }
}

impl ToString for Attribute {
    fn to_string(&self) -> String {
        if self.parameters.is_empty() {
            format!("{}", self.name)
        } else {
            format!("{}({})", self.name, self.parameters.join(","))
        }
    }
}

#[derive(Debug, Clone)]
pub struct DataType {
    name: String,
    properties: Vec<NameTypePair>,
}

impl DataType {
    pub fn new(name: String, properties: Vec<NameTypePair>) -> Self {
        Self { name, properties }
    }
}

impl ToString for DataType {
    fn to_string(&self) -> String {
        format!(
            "type {} {{\n{}\n}};",
            self.name,
            self.properties.iter().map(|p| format!("    {}: {},", p.name, p.type_)).collect::<Vec<_>>().join("\n"),
        )
    }
}

#[derive(Debug, Clone)]
pub struct Service {
    name: String,
    dependencies: Vec<String>,
    handlers: Vec<Handler>,
}

impl Service {
    pub fn new<D, H>(name: String, dependencies: Vec<D>, handlers: Vec<H>) -> Self
        where
            D: Into<String>,
            H: Into<Handler>,
    {
        Self {
            name,
            dependencies: dependencies.map_vec(),
            handlers: handlers.map_vec(),
        }
    }
}

impl ToString for Service {
    fn to_string(&self) -> String {
        format!(
            "service {} {{\n{}\n\n{}\n}};",
            self.name,
            &self.dependencies.iter().map(|d| format!("    depends on {};", d.to_string())).collect::<Vec<_>>().join("\n"),
            &self.handlers.iter().map(|h| format!("    {}", h.to_string())).collect::<Vec<_>>().join("\n"),
        )
    }
}

#[derive(Debug, Clone)]
pub struct Handler {
    name: String,
    arguments: Vec<NameTypePair>,
    return_type: Option<String>,
}

impl ToString for Handler {
    fn to_string(&self) -> String {
        format!(
            "handles {}({}){};",
            self.name,
            self.arguments.iter().map(|a| format!("{}", a.to_string())).collect::<Vec<_>>().join(", "),
            self.return_type.as_ref().map(|t| format!(" -> {}", t)).unwrap_or(String::new())
        )
    }
}

impl Handler {
    pub fn new<A>(name: String, arguments: Vec<A>, return_type: Option<String>) -> Self
        where
            A: Into<NameTypePair>,
    {
        Self {
            name,
            arguments: arguments.map_vec(),
            return_type,
        }
    }
}

#[derive(Clone)]
pub struct NameTypePair {
    name: String,
    type_: String,
}

impl Debug for NameTypePair {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        if fmt.alternate() {
            write!(
                fmt,
                "{}: {}",
                self.name, self.type_
            )
        } else {
            write!(fmt, "{{ name: {}, type: {} }}", self.name, self.type_)
        }
    }
}

impl ToString for NameTypePair {
    fn to_string(&self) -> String {
        format!("{}: {}", self.name, self.type_)
    }
}

impl NameTypePair {
    pub fn new(name: String, type_: String) -> Self {
        Self { name, type_ }
    }
}

// impl Debug for Help {
//     fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
//         use self::Help::*;
//         match *self {
//             Opcode(o) => write!(fmt, "{:?}", o),
//             PrefixOpcode(o) => write!(fmt, "{:?}", o),
//             PostfixOpcode(o) => write!(fmt, "{:?}", o),
//             Number(n) => write!(fmt, "{}", n),
//             Constant(c) => write!(fmt, "{:?}", c),
//             Show => write!(fmt, "show"),
//             Help => write!(fmt, "?"),
//         }
//     }
// }
