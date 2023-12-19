#![allow(dead_code)]

#[cfg(feature = "liquid")]
use liquid::{ObjectView, ValueView};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type OrderedMap<T> = BTreeMap<String, T>;

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
#[derive(Serialize, Deserialize, Debug, Clone)]
/// The file containing definitions
pub struct SsdFile {
    /// The namespace of the file. This corresponds to the path the file is located, except with :: instead of /
    pub namespace: Namespace,
    /// The imports of the file.
    pub imports: Vec<Import>,
    /// The data types described in the file
    pub data_types: OrderedMap<DataType>,
    /// The enums described in the file
    pub enums: OrderedMap<Enum>,
    /// The services described in the file
    pub services: OrderedMap<Service>,
}

const INDENT: &str = "    ";

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// An import
pub struct Import {
    /// The import path as namespace
    pub path: Namespace,
    /// Attributes on the import
    pub attributes: Vec<Attribute>,
}

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// A dependency
pub struct Dependency {
    /// The name of the dependency as namespace
    pub name: Namespace,
    /// The attributes of the dependency
    pub attributes: Vec<Attribute>,
    /// The comments for the dependency
    pub comments: Vec<String>,
}

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// A parameter
pub struct Parameter {
    /// The name of the parameter
    pub name: String,
    /// The value of the parameter
    pub value: Option<String>,
}

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// An attribute
pub struct Attribute {
    /// The name of the attribute
    pub name: Namespace,
    /// The parameters of the attribute
    pub parameters: Vec<Parameter>,
}

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// A data type
pub struct DataType {
    /// The fields of the data type
    pub properties: OrderedMap<TypeName>,
    /// The attributes of the data type
    pub attributes: Vec<Attribute>,
}

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// An enum
pub struct Enum {
    /// The values of the enum
    pub values: OrderedMap<EnumValue>,
    /// The attributes of the enum
    pub attributes: Vec<Attribute>,
}

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// A service
pub struct Service {
    /// The dependencies of the service
    pub dependencies: Vec<Dependency>,
    /// The functions the service provides
    pub functions: OrderedMap<Function>,
    /// The events the service reacts to
    pub events: OrderedMap<Event>,
    /// The attributes of the service
    pub attributes: Vec<Attribute>,
}

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// A function
pub struct Function {
    /// The arguments of the function
    pub arguments: OrderedMap<TypeName>,
    /// The return type of the function, if any
    pub return_type: Option<Namespace>,
    /// the attributes of the function
    pub attributes: Vec<Attribute>,
    /// The comments for the function
    pub comments: Vec<String>,
}

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// An event
pub struct Event {
    /// The arguments for the event
    pub arguments: OrderedMap<TypeName>,
    /// The attributes for the event
    pub attributes: Vec<Attribute>,
    /// The comments for the event
    pub comments: Vec<String>,
}

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
/// A TypeName
pub struct TypeName {
    /// The name of the type as namespace
    pub typ: Namespace,
    /// The attribute of the type
    pub attributes: Vec<Attribute>,
    /// the comments of the type
    pub comments: Vec<String>,
}

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
/// An enum value
pub struct EnumValue {
    /// The optional value of the enum value
    pub value: Option<i64>,
    /// The attributes of the enum value
    pub attributes: Vec<Attribute>,
    /// The comments for the enum value
    pub comments: Vec<String>,
}

#[cfg_attr(feature = "liquid", derive(ObjectView, ValueView))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// A namespace
pub struct Namespace {
    /// The collection of parts making up a namespace
    pub components: Vec<String>,
}

impl IntoIterator for Namespace {
    type Item = String;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.components.into_iter()
    }
}
