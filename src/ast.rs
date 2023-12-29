use crate::parser::raw_service_to_service;

use serde::{Deserialize, Serialize};

pub use ssd_data::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub enum AstElement {
    Comment(String),
    Import(Import),
    DataType((String, DataType)),
    Enum((String, Enum)),
    Service((String, Vec<ServiceAstElement>, Vec<Attribute>)),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServiceAstElement {
    Comment(String),
    Dependency(Dependency),
    Function((String, Function)),
    Event((String, Event)),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ComparableAstElement {
    Comment(String),
    Import(Import),
    DataType((String, DataType)),
    Enum((String, Enum)),
    Service((String, Service)),
}

impl From<&AstElement> for ComparableAstElement {
    fn from(value: &AstElement) -> Self {
        match value {
            AstElement::Comment(c) => ComparableAstElement::Comment(c.clone()),
            AstElement::Import(i) => ComparableAstElement::Import(i.clone()),
            AstElement::DataType(dt) => ComparableAstElement::DataType(dt.clone()),
            AstElement::Enum(en) => ComparableAstElement::Enum(en.clone()),
            AstElement::Service((name, svc, attributes)) => ComparableAstElement::Service((
                name.clone(),
                raw_service_to_service(svc, attributes),
            )),
        }
    }
}
