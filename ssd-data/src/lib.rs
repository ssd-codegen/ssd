#![allow(dead_code)]

#[cfg(feature = "liquid")]
use liquid::{ObjectView, ValueView};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug, io::Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub type OrderedMap<T> = BTreeMap<String, T>;

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SsdFile {
    pub namespace: Namespace,
    pub imports: Vec<Import>,
    pub data_types: OrderedMap<DataType>,
    pub enums: OrderedMap<Enum>,
    pub services: OrderedMap<Service>,
}

const INDENT: &str = "    ";

impl SsdFile {
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

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
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

    pub fn path(&mut self) -> Namespace {
        self.path.clone()
    }

    pub fn attributes(&mut self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Dependency {
    pub name: Namespace,
    pub attributes: Vec<Attribute>,
}

impl Dependency {
    #[must_use]
    pub fn new(name: Namespace, attributes: Vec<Attribute>) -> Self {
        Dependency {
            name,
            attributes,
        }
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

fn format_attributes(v: &Vec<Attribute>, suffix: Option<&str>) -> String {
    if v.is_empty() {
        String::new()
    } else {
        format!(
            "@[{}]{}",
            v.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", "),
            suffix.unwrap_or_default()
        )
    }
}

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub value: Option<String>,
}

impl Parameter {
    pub fn name(&mut self) -> String {
        self.name.clone()
    }

    pub fn value(&mut self) -> Option<String> {
        self.value.clone()
    }
}

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
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

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
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

    pub fn properties(&mut self) -> OrderedMap<NameTypePair> {
        self.properties.clone()
    }

    pub fn attributes(&mut self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
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

    pub fn values(&mut self) -> OrderedMap<EnumValue> {
        self.values.clone()
    }

    pub fn attributes(&mut self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
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

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Function {
    pub arguments: OrderedMap<NameTypePair>,
    pub return_type: Option<Namespace>,
    pub attributes: Vec<Attribute>,
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
        }
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

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Event {
    pub arguments: OrderedMap<NameTypePair>,
    pub attributes: Vec<Attribute>,
}

impl Event {
    #[must_use]
    pub fn new(arguments: OrderedMap<NameTypePair>, attributes: Vec<Attribute>) -> Self {
        Self {
            arguments,
            attributes,
        }
    }

    pub fn arguments(&mut self) -> OrderedMap<NameTypePair> {
        self.arguments.clone()
    }

    pub fn attributes(&mut self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NameTypePair {
    pub typ: Namespace,
    pub attributes: Vec<Attribute>,
}

impl NameTypePair {
    #[must_use]
    pub fn new(typ: Namespace, attributes: Vec<Attribute>) -> Self {
        Self {
            typ,
            attributes,
        }
    }

    pub fn typ(&mut self) -> Namespace {
        self.typ.clone()
    }

    pub fn attributes(&mut self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EnumValue {
    pub value: Option<i64>,
    pub attributes: Vec<Attribute>,
}

impl EnumValue {
    #[must_use]
    pub fn new(value: Option<i64>, attributes: Vec<Attribute>) -> Self {
        Self {
            value,
            attributes,
        }
    }

    pub fn value(&mut self) -> Option<i64> {
        self.value.clone()
    }

    pub fn attributes(&mut self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
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
