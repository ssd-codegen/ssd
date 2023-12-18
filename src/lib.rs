#![warn(clippy::pedantic)]

mod parser;

pub use parser::{parse, parse_file, ParseError};
pub use ssd_data::ast::{
    Attribute, DataType, Dependency, Enum, EnumValue, Event, Function, Import, NameTypePair,
    Namespace, OrderedMap, Parameter, Service, SsdcFile,
};
