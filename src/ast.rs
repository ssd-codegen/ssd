use crate::parser::raw_service_to_service;

use std::{fmt::Debug, io::Write};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub type OrderedMap<T> = IndexMap<String, T>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SsdcFile {
    pub namespace: Namespace,
    pub imports: Vec<Import>,
    pub data_types: OrderedMap<DataType>,
    pub enums: OrderedMap<Enum>,
    pub services: OrderedMap<Service>,
}

impl SsdcFile {
    #[must_use]
    pub fn new(
        namespace: Namespace,
        imports: Vec<Import>,
        data_types: OrderedMap<DataType>,
        enums: OrderedMap<Enum>,
        services: OrderedMap<Service>,
    ) -> Self {
        Self {
            namespace,
            imports,
            data_types,
            enums,
            services,
        }
    }

    pub fn to_external(self) -> ssd_data::SsdcFile {
        ssd_data::SsdcFile {
            namespace: self.namespace.to_external(),
            imports: self
                .imports
                .into_iter()
                .map(|import| import.to_external())
                .collect(),
            data_types: self
                .data_types
                .into_iter()
                .map(|(name, value)| (name, value.to_external()))
                .collect(),
            enums: self
                .enums
                .into_iter()
                .map(|(name, value)| (name, value.to_external()))
                .collect(),
            services: self
                .services
                .into_iter()
                .map(|(name, value)| (name, value.to_external()))
                .collect(),
        }
    }

    pub fn namespace(&mut self) -> Namespace {
        self.namespace.clone()
    }

    pub fn imports(&mut self) -> Vec<Import> {
        self.imports.clone()
    }

    pub fn data_types(&mut self) -> OrderedMap<DataType> {
        self.data_types.clone()
    }

    pub fn enums(&mut self) -> OrderedMap<Enum> {
        self.enums.clone()
    }

    pub fn services(&mut self) -> OrderedMap<Service> {
        self.services.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Import {
    pub path: Namespace,
    pub attributes: Vec<Attribute>,
}

impl Import {
    #[must_use]
    pub fn new(path: Namespace, attributes: Vec<Attribute>) -> Self {
        Import { path, attributes }
    }

    pub fn to_external(self) -> ssd_data::Import {
        ssd_data::Import::new(
            self.path.to_external(),
            self.attributes
                .into_iter()
                .map(|a| a.to_external())
                .collect(),
        )
    }

    pub fn path(&mut self) -> Namespace {
        self.path.clone()
    }

    pub fn attributes(&mut self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Dependency {
    pub name: Namespace,
    pub attributes: Vec<Attribute>,
    pub comments: Vec<String>,
}

impl Dependency {
    #[must_use]
    pub fn new(name: Namespace, attributes: Vec<Attribute>) -> Self {
        Dependency {
            name,
            attributes,
            comments: Vec::new(),
        }
    }

    pub fn to_external(self) -> ssd_data::Dependency {
        ssd_data::Dependency {
            name: self.name.to_external(),
            attributes: self
                .attributes
                .into_iter()
                .map(|a| a.to_external())
                .collect(),
        }
    }

    pub fn with_comments(mut self, comments: &mut Vec<String>) -> Self {
        self.comments.append(comments);
        self
    }

    pub fn name(&mut self) -> Namespace {
        self.name.clone()
    }

    pub fn attributes(&mut self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

impl ToString for Dependency {
    fn to_string(&self) -> String {
        self.name.to_string()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub value: Option<String>,
}

impl Parameter {
    pub fn name(&mut self) -> String {
        self.name.clone()
    }

    pub fn to_external(self) -> ssd_data::Parameter {
        let Parameter { name, value } = self;
        ssd_data::Parameter { name, value }
    }

    pub fn value(&mut self) -> Option<String> {
        self.value.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: Namespace,
    pub parameters: Vec<Parameter>,
}

impl Attribute {
    #[must_use]
    pub fn new(name: Namespace, parameters: Vec<(String, Option<String>)>) -> Self {
        Self {
            name,
            parameters: parameters
                .into_iter()
                .map(|(name, value)| Parameter { name, value })
                .collect(),
        }
    }

    pub fn to_external(self) -> ssd_data::Attribute {
        ssd_data::Attribute {
            name: self.name.to_external(),
            parameters: self
                .parameters
                .into_iter()
                .map(|p| p.to_external())
                .collect(),
        }
    }

    pub fn name(&mut self) -> Namespace {
        self.name.clone()
    }

    pub fn parameters(&mut self) -> Vec<Parameter> {
        self.parameters.clone()
    }
}

impl ToString for Attribute {
    fn to_string(&self) -> String {
        if self.parameters.is_empty() {
            self.name.to_string()
        } else {
            format!(
                "{}({})",
                self.name.to_string(),
                self.parameters
                    .iter()
                    .map(|p| format!(
                        "{}{}",
                        p.name,
                        p.value
                            .clone()
                            .map(|v| format!(" = {v}"))
                            .unwrap_or_default()
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DataType {
    pub properties: OrderedMap<NameTypePair>,
    pub attributes: Vec<Attribute>,
}

impl DataType {
    #[must_use]
    pub fn new(properties: OrderedMap<NameTypePair>, attributes: Vec<Attribute>) -> Self {
        Self {
            properties,
            attributes,
        }
    }

    pub fn to_external(self) -> ssd_data::DataType {
        ssd_data::DataType {
            properties: self
                .properties
                .into_iter()
                .map(|(name, p)| (name, p.to_external()))
                .collect(),
            attributes: self
                .attributes
                .into_iter()
                .map(|a| a.to_external())
                .collect(),
        }
    }

    pub fn properties(&mut self) -> OrderedMap<NameTypePair> {
        self.properties.clone()
    }

    pub fn attributes(&mut self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Enum {
    pub values: OrderedMap<EnumValue>,
    pub attributes: Vec<Attribute>,
}

impl Enum {
    #[must_use]
    pub fn new(values: OrderedMap<EnumValue>, attributes: Vec<Attribute>) -> Self {
        Self { values, attributes }
    }

    pub fn to_external(self) -> ssd_data::Enum {
        ssd_data::Enum {
            values: self
                .values
                .into_iter()
                .map(|(name, a)| (name, a.to_external()))
                .collect(),
            attributes: self
                .attributes
                .into_iter()
                .map(|a| a.to_external())
                .collect(),
        }
    }

    pub fn values(&mut self) -> OrderedMap<EnumValue> {
        self.values.clone()
    }

    pub fn attributes(&mut self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Service {
    pub dependencies: Vec<Dependency>,
    pub functions: OrderedMap<Function>,
    pub events: OrderedMap<Event>,
    pub attributes: Vec<Attribute>,
}

impl Service {
    #[must_use]
    pub fn new(
        dependencies: Vec<Dependency>,
        functions: OrderedMap<Function>,
        events: OrderedMap<Event>,
        attributes: Vec<Attribute>,
    ) -> Self {
        Self {
            dependencies,
            functions,
            events,
            attributes,
        }
    }

    pub fn to_external(self) -> ssd_data::Service {
        ssd_data::Service {
            dependencies: self
                .dependencies
                .into_iter()
                .map(|a| a.to_external())
                .collect(),
            functions: self
                .functions
                .into_iter()
                .map(|(name, a)| (name, a.to_external()))
                .collect(),
            events: self
                .events
                .into_iter()
                .map(|(name, a)| (name, a.to_external()))
                .collect(),
            attributes: self
                .attributes
                .into_iter()
                .map(|a| a.to_external())
                .collect(),
        }
    }

    pub fn dependencies(&mut self) -> Vec<Dependency> {
        self.dependencies.clone()
    }

    pub fn functions(&mut self) -> OrderedMap<Function> {
        self.functions.clone()
    }

    pub fn handlers(&mut self) -> OrderedMap<Function> {
        const DEPRECATED: &str =  "Using the property 'handlers' is deprecated and will be removed in future versions. Use 'functions' instead.";
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
        self.functions()
    }

    pub fn events(&mut self) -> OrderedMap<Event> {
        self.events.clone()
    }

    pub fn attributes(&mut self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Function {
    pub arguments: OrderedMap<NameTypePair>,
    pub return_type: Option<Namespace>,
    pub attributes: Vec<Attribute>,
    pub comments: Vec<String>,
}

impl Function {
    #[must_use]
    pub fn new(
        arguments: OrderedMap<NameTypePair>,
        return_type: Option<Namespace>,
        attributes: Vec<Attribute>,
    ) -> Self {
        Self {
            arguments,
            return_type,
            attributes,
            comments: Vec::new(),
        }
    }

    pub fn to_external(self) -> ssd_data::Function {
        ssd_data::Function {
            arguments: self
                .arguments
                .into_iter()
                .map(|(name, a)| (name, a.to_external()))
                .collect(),
            attributes: self
                .attributes
                .into_iter()
                .map(|a| a.to_external())
                .collect(),
            return_type: self.return_type.map(|r| r.to_external()),
        }
    }

    pub fn with_comments(mut self, comments: &mut Vec<String>) -> Self {
        self.comments.append(comments);
        self
    }

    pub fn arguments(&mut self) -> OrderedMap<NameTypePair> {
        self.arguments.clone()
    }

    pub fn return_type(&mut self) -> Option<Namespace> {
        self.return_type.clone()
    }

    pub fn attributes(&mut self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Event {
    pub arguments: OrderedMap<NameTypePair>,
    pub attributes: Vec<Attribute>,
    pub comments: Vec<String>,
}

impl Event {
    #[must_use]
    pub fn new(arguments: OrderedMap<NameTypePair>, attributes: Vec<Attribute>) -> Self {
        Self {
            arguments,
            attributes,
            comments: Vec::new(),
        }
    }

    pub fn to_external(self) -> ssd_data::Event {
        ssd_data::Event {
            arguments: self
                .arguments
                .into_iter()
                .map(|(name, a)| (name, a.to_external()))
                .collect(),
            attributes: self
                .attributes
                .into_iter()
                .map(|a| a.to_external())
                .collect(),
        }
    }

    pub fn with_comments(mut self, comments: &mut Vec<String>) -> Self {
        self.comments.append(comments);
        self
    }

    pub fn arguments(&mut self) -> OrderedMap<NameTypePair> {
        self.arguments.clone()
    }

    pub fn attributes(&mut self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct NameTypePair {
    pub typ: Namespace,
    pub attributes: Vec<Attribute>,
    pub comments: Vec<String>,
}

impl NameTypePair {
    #[must_use]
    pub fn new(typ: Namespace, attributes: Vec<Attribute>) -> Self {
        Self {
            typ,
            attributes,
            comments: Vec::new(),
        }
    }

    pub fn to_external(self) -> ssd_data::NameTypePair {
        ssd_data::NameTypePair {
            typ: self.typ.to_external(),
            attributes: self
                .attributes
                .into_iter()
                .map(|a| a.to_external())
                .collect(),
        }
    }

    pub fn with_comments(mut self, comments: &mut Vec<String>) -> Self {
        self.comments.append(comments);
        self
    }

    pub fn typ(&mut self) -> Namespace {
        self.typ.clone()
    }

    pub fn attributes(&mut self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct EnumValue {
    pub value: Option<i64>,
    pub attributes: Vec<Attribute>,
    pub comments: Vec<String>,
}

impl EnumValue {
    #[must_use]
    pub fn new(value: Option<i64>, attributes: Vec<Attribute>) -> Self {
        Self {
            value,
            attributes,
            comments: Vec::new(),
        }
    }

    pub fn to_external(self) -> ssd_data::EnumValue {
        ssd_data::EnumValue {
            value: self.value,
            attributes: self
                .attributes
                .into_iter()
                .map(|a| a.to_external())
                .collect(),
        }
    }

    pub fn with_comments(mut self, comments: &mut Vec<String>) -> Self {
        self.comments.append(comments);
        self
    }

    pub fn value(&mut self) -> Option<i64> {
        self.value.clone()
    }

    pub fn attributes(&mut self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Namespace {
    components: Vec<String>,
}

impl IntoIterator for Namespace {
    type Item = String;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.components.into_iter()
    }
}

impl Namespace {
    #[must_use]
    pub fn new(v: &str) -> Self {
        Namespace {
            components: v.split("::").map(ToOwned::to_owned).collect(),
        }
    }

    pub fn to_external(self) -> ssd_data::Namespace {
        ssd_data::Namespace::from_vec(self.components)
    }

    #[must_use]
    pub fn from_vec(components: Vec<String>) -> Self {
        Namespace { components }
    }

    pub fn components(&mut self) -> Vec<String> {
        self.components.clone()
    }
}

impl ToString for Namespace {
    fn to_string(&self) -> String {
        self.components.join("::")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
                raw_service_to_service(&svc, &attributes),
            )),
        }
    }
}
