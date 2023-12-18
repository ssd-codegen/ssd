#![warn(clippy::pedantic)]

mod parser;
mod ast;

pub use parser::{parse, parse_raw, parse_file, ParseError};
pub use ast::AstElement;
pub use ssd_data::{
    Attribute, DataType, Dependency, Enum, EnumValue, Event, Function, Import, NameTypePair,
    Namespace, OrderedMap, Parameter, Service, SsdcFile,
};
