#![allow(dead_code)]

use std::fmt::{Debug, Formatter};

#[derive(Debug, Clone)]
pub struct SsdcFile {
    pub namespace: Namespace,
    pub imports: Vec<Import>,
    pub data_types: Vec<DataType>,
    pub services: Vec<Service>,
}

const INDENT: &str = "    ";

impl SsdcFile {
    pub fn new(
        namespace: Namespace,
        imports: Vec<Import>,
        data_types: Vec<DataType>,
        services: Vec<Service>,
    ) -> Self {
        Self {
            namespace,
            imports,
            data_types,
            services,
        }
    }

    pub fn namespace(&mut self) -> Namespace {
        self.namespace.clone()
    }

    pub fn imports(&mut self) -> Vec<Import> {
        self.imports.clone()
    }

    pub fn data_types(&mut self) -> Vec<DataType> {
        self.data_types.clone()
    }

    pub fn services(&mut self) -> Vec<Service> {
        self.services.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Import {
    pub path: Namespace,
    pub attributes: Vec<Attribute>,
}

impl Import {
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

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: Namespace,
    pub attributes: Vec<Attribute>,
}

impl Dependency {
    pub fn new(name: Namespace, attributes: Vec<Attribute>) -> Self {
        Dependency { name, attributes }
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
        "".to_string()
    } else {
        format!(
            "@[{}]{}",
            v.iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            suffix.unwrap_or_default()
        )
    }
}

impl ToString for SsdcFile {
    fn to_string(&self) -> String {
        format!(
            "{}\n\n{}\n\n{}",
            self.imports
                .iter()
                .map(|i| format!(
                    "{}import {};",
                    format_attributes(&i.attributes, Some("\n")),
                    i.path.to_string()
                ))
                .collect::<Vec<_>>()
                .join("\n"),
            self.data_types
                .iter()
                .map(|d| format!(
                    "{}{}",
                    format_attributes(&d.attributes, Some("\n")),
                    d.to_string()
                ))
                .collect::<Vec<_>>()
                .join("\n"),
            self.services
                .iter()
                .map(|s| format!(
                    "{}{}",
                    format_attributes(&s.attributes, Some("\n")),
                    s.to_string()
                ))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: Namespace,
    pub parameters: Vec<Parameter>,
}

impl Attribute {
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
            format!("{}", self.name.to_string())
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

#[derive(Debug, Clone)]
pub struct DataType {
    pub name: String,
    pub properties: Vec<NameTypePair>,
    pub attributes: Vec<Attribute>,
}

impl DataType {
    pub fn new(name: String, properties: Vec<NameTypePair>, attributes: Vec<Attribute>) -> Self {
        Self {
            name,
            properties,
            attributes,
        }
    }

    pub fn name(&mut self) -> String {
        self.name.clone()
    }

    pub fn properties(&mut self) -> Vec<NameTypePair> {
        self.properties.clone()
    }

    pub fn attributes(&mut self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

impl ToString for DataType {
    fn to_string(&self) -> String {
        format!(
            "type {} {{\n{}\n}};\n",
            self.name,
            self.properties
                .iter()
                .map(|p| format!(
                    "{}{}{}: {},",
                    INDENT,
                    format_attributes(&p.attributes, Some("\n    ")),
                    p.name,
                    p.typ.to_string()
                ))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}

#[derive(Debug, Clone)]
pub struct Service {
    pub name: String,
    pub dependencies: Vec<Dependency>,
    pub handlers: Vec<Handler>,
    pub attributes: Vec<Attribute>,
}

impl Service {
    pub fn new(
        name: String,
        dependencies: Vec<Dependency>,
        handlers: Vec<Handler>,
        attributes: Vec<Attribute>,
    ) -> Self {
        Self {
            name,
            dependencies,
            handlers,
            attributes,
        }
    }

    pub fn name(&mut self) -> String {
        self.name.clone()
    }

    pub fn dependencies(&mut self) -> Vec<Dependency> {
        self.dependencies.clone()
    }

    pub fn handlers(&mut self) -> Vec<Handler> {
        self.handlers.clone()
    }

    pub fn attributes(&mut self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

impl ToString for Service {
    fn to_string(&self) -> String {
        format!(
            "service {} {{\n{}\n\n{}\n}};",
            self.name,
            &self
                .dependencies
                .iter()
                .map(|d| format!(
                    "{}{}depends on {};",
                    INDENT,
                    format_attributes(&d.attributes, Some(&format!("\n{}", INDENT))),
                    d.to_string()
                ))
                .collect::<Vec<_>>()
                .join("\n"),
            &self
                .handlers
                .iter()
                .map(|h| format!(
                    "{}{}{}",
                    INDENT,
                    format_attributes(&h.attributes, Some(&format!("\n{}", INDENT))),
                    h.to_string()
                ))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}

#[derive(Debug, Clone)]
pub struct Handler {
    pub name: String,
    pub arguments: Vec<NameTypePair>,
    pub return_type: Option<Namespace>,
    pub attributes: Vec<Attribute>,
}

impl ToString for Handler {
    fn to_string(&self) -> String {
        format!(
            "handles {}({}){};",
            self.name,
            self.arguments
                .iter()
                .map(|a| format!(
                    "{}{}: {}",
                    format_attributes(&a.attributes, Some(" ")),
                    a.name,
                    a.typ.to_string(),
                ))
                .collect::<Vec<_>>()
                .join(", "),
            self.return_type
                .as_ref()
                .map(|t| format!(" -> {}", t.to_string()))
                .unwrap_or_else(|| String::new())
        )
    }
}

impl Handler {
    pub fn new(
        name: String,
        arguments: Vec<NameTypePair>,
        return_type: Option<Namespace>,
        attributes: Vec<Attribute>,
    ) -> Self {
        Self {
            name,
            arguments,
            return_type,
            attributes,
        }
    }

    pub fn name(&mut self) -> String {
        self.name.clone()
    }

    pub fn arguments(&mut self) -> Vec<NameTypePair> {
        self.arguments.clone()
    }

    pub fn return_type(&mut self) -> Option<Namespace> {
        self.return_type.clone()
    }

    pub fn attributes(&mut self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

#[derive(Clone)]
pub struct NameTypePair {
    pub name: String,
    pub typ: Namespace,
    pub attributes: Vec<Attribute>,
}

impl Debug for NameTypePair {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        if fmt.alternate() {
            write!(
                fmt,
                "{}{}: {}",
                format_attributes(&self.attributes, None),
                self.name,
                self.typ.to_string()
            )
        } else {
            write!(
                fmt,
                "{{ name: {}, type: {:?}, attributes: {:?} }}",
                self.name, self.typ, self.attributes
            )
        }
    }
}

impl NameTypePair {
    pub fn new(name: String, typ: Namespace, attributes: Vec<Attribute>) -> Self {
        Self {
            name,
            typ,
            attributes,
        }
    }

    pub fn name(&mut self) -> String {
        self.name.clone()
    }

    pub fn typ(&mut self) -> Namespace {
        self.typ.clone()
    }

    pub fn attributes(&mut self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

#[derive(Debug, Clone)]
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
    pub fn new(v: String) -> Self {
        Namespace {
            components: v.split("::").map(ToOwned::to_owned).collect(),
        }
    }

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
