use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[cfg(feature = "_access_functions")]
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[cfg(feature = "_access_functions")]
use std::io::Write;

pub type OrderedMap<T> = IndexMap<String, T>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SsdFile {
    pub namespace: Namespace,
    pub imports: Vec<Import>,
    pub data_types: OrderedMap<DataType>,
    pub enums: OrderedMap<Enum>,
    pub services: OrderedMap<Service>,
}

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
}

#[cfg(feature = "_access_functions")]
impl SsdFile {
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
}

#[cfg(feature = "_access_functions")]
impl Import {
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

    pub fn with_comments(mut self, comments: &mut Vec<String>) -> Self {
        self.comments.append(comments);
        self
    }
}

#[cfg(feature = "_access_functions")]
impl Dependency {
    pub fn name(&mut self) -> Namespace {
        self.name.clone()
    }

    pub fn attributes(&mut self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub value: Option<String>,
}

#[cfg(feature = "_access_functions")]
impl Parameter {
    pub fn name(&mut self) -> String {
        self.name.clone()
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
}

#[cfg(feature = "_access_functions")]
impl Attribute {
    pub fn name(&mut self) -> Namespace {
        self.name.clone()
    }

    pub fn parameters(&mut self) -> Vec<Parameter> {
        self.parameters.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DataType {
    pub properties: OrderedMap<TypeName>,
    pub attributes: Vec<Attribute>,
}

impl DataType {
    #[must_use]
    pub fn new(properties: OrderedMap<TypeName>, attributes: Vec<Attribute>) -> Self {
        Self {
            properties,
            attributes,
        }
    }
}

#[cfg(feature = "_access_functions")]
impl DataType {
    pub fn properties(&mut self) -> OrderedMap<TypeName> {
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
}

#[cfg(feature = "_access_functions")]
impl Enum {
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
}

#[cfg(feature = "_access_functions")]
impl Service {
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
    pub arguments: OrderedMap<TypeName>,
    pub return_type: Option<Namespace>,
    pub attributes: Vec<Attribute>,
    pub comments: Vec<String>,
}

impl Function {
    #[must_use]
    pub fn new(
        arguments: OrderedMap<TypeName>,
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

    pub fn with_comments(mut self, comments: &mut Vec<String>) -> Self {
        self.comments.append(comments);
        self
    }
}

#[cfg(feature = "_access_functions")]
impl Function {
    pub fn arguments(&mut self) -> OrderedMap<TypeName> {
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
    pub arguments: OrderedMap<TypeName>,
    pub attributes: Vec<Attribute>,
    pub comments: Vec<String>,
}

impl Event {
    #[must_use]
    pub fn new(arguments: OrderedMap<TypeName>, attributes: Vec<Attribute>) -> Self {
        Self {
            arguments,
            attributes,
            comments: Vec::new(),
        }
    }

    pub fn with_comments(mut self, comments: &mut Vec<String>) -> Self {
        self.comments.append(comments);
        self
    }
}

#[cfg(feature = "_access_functions")]
impl Event {
    pub fn arguments(&mut self) -> OrderedMap<TypeName> {
        self.arguments.clone()
    }

    pub fn attributes(&mut self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct TypeName {
    pub typ: Namespace,
    pub attributes: Vec<Attribute>,
    pub comments: Vec<String>,
}

impl TypeName {
    #[must_use]
    pub fn new(typ: Namespace, attributes: Vec<Attribute>) -> Self {
        Self {
            typ,
            attributes,
            comments: Vec::new(),
        }
    }

    pub fn with_comments(mut self, comments: &mut Vec<String>) -> Self {
        self.comments.append(comments);
        self
    }
}

#[cfg(feature = "_access_functions")]
impl TypeName {
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

    pub fn with_comments(mut self, comments: &mut Vec<String>) -> Self {
        self.comments.append(comments);
        self
    }
}

#[cfg(feature = "_access_functions")]
impl EnumValue {
    pub fn value(&mut self) -> Option<i64> {
        self.value
    }

    pub fn attributes(&mut self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Namespace {
    pub components: Vec<String>,
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
}

#[cfg(feature = "_access_functions")]
impl Namespace {
    pub fn components(&mut self) -> Vec<String> {
        self.components.clone()
    }
}

impl ToString for Namespace {
    fn to_string(&self) -> String {
        self.components.join("::")
    }
}
