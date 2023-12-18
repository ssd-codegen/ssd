#![warn(clippy::pedantic)]

mod ast;
mod parser;

pub use parser::{parse, parse_file, parse_file_raw, parse_raw, ParseError};
pub use ssd_data::{
    Attribute, DataType, Dependency, Enum, EnumValue, Event, Function, Import, NameTypePair,
    Namespace, OrderedMap, Parameter, Service, SsdcFile,
};
