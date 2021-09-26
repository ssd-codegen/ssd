#![allow(dead_code)]

use std::fmt::{Debug, Formatter};
use crate::map_vec::MapVec;

#[derive(Debug)]
pub struct Namespace {
    name: String,
    imports: Vec<Attributed<String>>,
    data_types: Vec<Attributed<DataType>>,
    services: Vec<Attributed<Service>>,
}

impl Namespace {
    pub fn new<I, DT, S>(
        name: String,
        imports: Vec<I>,
        data_types: Vec<DT>,
        services: Vec<S>,
    ) -> Self
        where
            I: Into<Attributed<String>>,
            DT: Into<Attributed<DataType>>,
            S: Into<Attributed<Service>>,
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
            self.imports.iter().map(|i| format!("import {};", i.to_string())).collect::<Vec<_>>().join("\n"),
            self.data_types.iter().map(|d| format!("{}", d.to_string())).collect::<Vec<_>>().join("\n"),
            self.services.iter().map(|s| format!("{}", s.to_string())).collect::<Vec<_>>().join("\n"),
        )
    }
}

#[derive(Debug)]
pub struct Attributed<T> {
    value: T,
    attributes: Vec<Attribute>,
}

impl<T> From<T> for Attributed<T> {
    fn from(value: T) -> Self {
        Self {
            value,
            attributes: Vec::new(),
        }
    }
}

impl<T: ToString> ToString for Attributed<T> {
    fn to_string(&self) -> String {
        let prefix = if !self.attributes.is_empty() {
            format!("{}\n", self.attributes.iter().map(ToString::to_string).collect::<Vec<_>>().join(","))
        } else {
            "".to_string()
        };
        format!("{}{}", prefix, self.value.to_string())
    }
}

#[derive(Debug)]
pub struct Attribute {
    name: String,
    parameters: Vec<String>,
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Service {
    name: String,
    dependencies: Vec<Attributed<String>>,
    handlers: Vec<Attributed<Handler>>,
}

impl Service {
    pub fn new<D, H>(name: String, dependencies: Vec<D>, handlers: Vec<H>) -> Self
        where
            D: Into<Attributed<String>>,
            H: Into<Attributed<Handler>>
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

#[derive(Debug)]
pub struct Handler {
    name: String,
    arguments: Vec<Attributed<NameTypePair>>,
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
            A: Into<Attributed<NameTypePair>>,
    {
        Self {
            name,
            arguments: arguments.map_vec(),
            return_type,
        }
    }
}

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
