#![allow(dead_code)]

use std::fmt::{Debug, Formatter};

#[derive(Debug)]
pub struct Namespace {
    name: String,
    imports: Vec<String>,
    data_types: Vec<DataType>,
    services: Vec<Service>,
}

impl Namespace {
    pub fn new(
        name: String,
        imports: Vec<String>,
        data_types: Vec<DataType>,
        services: Vec<Service>,
    ) -> Self {
        Self {
            name,
            imports,
            data_types,
            services,
        }
    }

    pub fn pretty(&self) -> String {
            format!(
                "namespace {};\n\n{}\n\n{}\n\n{}",
                self.name,
                self.imports.iter().map(|i| format!("import {};", i)).collect::<Vec<_>>().join("\n"),
                self.data_types.iter().map(|d| format!("{}", d.pretty())).collect::<Vec<_>>().join("\n"),
                self.services.iter().map(|s| format!("{}", s.pretty())).collect::<Vec<_>>().join("\n"),
            )
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

    pub fn pretty(&self) -> String {
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
    dependencies: Vec<String>,
    handlers: Vec<Handler>,
}

impl Service {
    pub fn new(name: String, dependencies: Vec<String>, handlers: Vec<Handler>) -> Self {
        Self {
            name,
            dependencies,
            handlers,
        }
    }
    pub fn pretty(&self) -> String {
            format!(
                "service {} {{\n{}\n\n{}\n}};",
                self.name,
                &self.dependencies.iter().map(|d| format!("    depends on {};", d)).collect::<Vec<_>>().join("\n"),
                &self.handlers.iter().map(|h| format!("    {}", h.pretty())).collect::<Vec<_>>().join("\n"),
            )
    }
}

#[derive(Debug)]
pub struct Handler {
    name: String,
    arguments: Vec<NameTypePair>,
    return_type: Option<String>,
}

impl Handler {
    pub fn new(name: String, arguments: Vec<NameTypePair>, return_type: Option<String>) -> Self {
        Self {
            name,
            arguments,
            return_type,
        }
    }

    pub fn pretty(&self) -> String {
        format!(
            "handles {}({}){};",
            self.name,
            self.arguments.iter().map(|a| format!("{}: {}", a.name, a.type_)).collect::<Vec<_>>().join(", "),
            self.return_type.as_ref().map(|t| format!(" -> {}", t)).unwrap_or(String::new())
        )
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
