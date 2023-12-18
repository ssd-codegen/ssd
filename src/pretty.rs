use crate::ast::{AstElement, ServiceAstElement};
use crate::parser::raw_service_to_service;
use ssd_data::{
    Attribute, DataType, Dependency, Enum, EnumValue, Event, Function, NameTypePair, Namespace,
    Parameter,
};

fn namespace_to_string(namespace: Namespace) -> String {
    namespace.into_iter().collect::<Vec<_>>().join("::")
}

fn parameters_to_string(parameters: &[Parameter]) -> String {
    parameters
        .iter()
        .map(|p| {
            p.value
                .as_ref()
                .map(|v| format!("{} = \"{v}\"", p.name))
                .unwrap_or_else(|| p.name.clone())
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn attributes_to_string(attributes: &[Attribute]) -> String {
    let attr_string = attributes
        .iter()
        .map(|attribute| {
            let name = namespace_to_string(attribute.name.clone());
            if attribute.parameters.is_empty() {
                name
            } else {
                format!("{name}({})", parameters_to_string(&attribute.parameters))
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("#[{attr_string}]")
}

fn datatype_to_string(name: &str, datatype: &DataType) -> String {
    let mut result = Vec::new();

    if !datatype.attributes.is_empty() {
        result.push(attributes_to_string(&datatype.attributes));
    }
    result.push(format!("data {name} {{"));
    for (name, NameTypePair { typ, attributes }) in &datatype.properties {
        if !attributes.is_empty() {
            result.push(format!("\t{}", attributes_to_string(attributes)));
        }
        result.push(format!("\t{name}: {},", namespace_to_string(typ.clone())))
    }
    result.push(format!("}};"));
    result.join("\n")
}

fn enum_to_string(name: &str, en: &Enum) -> String {
    let mut result = Vec::new();

    if !en.attributes.is_empty() {
        result.push(attributes_to_string(&en.attributes));
    }
    result.push(format!("enum {name} {{"));
    for (name, EnumValue { value, attributes }) in &en.values {
        let mut attr_string = "".to_string();

        if !attributes.is_empty() {
            attr_string = format!("{} ", attributes_to_string(&attributes));
        }
        if let Some(value) = value {
            result.push(format!("\t{attr_string}{name} = {},", value));
        } else {
            result.push(format!("\t{attr_string}{name},"));
        }
    }
    result.push(format!("}};"));
    result.join("\n")
}

fn argument_to_string(name: &str, arg: &NameTypePair) -> String {
    let mut attr_string = "".to_string();

    if !arg.attributes.is_empty() {
        attr_string = format!("{} ", attributes_to_string(&arg.attributes));
    }

    format!(
        "{attr_string}{name}: {}",
        namespace_to_string(arg.typ.clone())
    )
}

fn service_to_string(
    name: &str,
    service: &[ServiceAstElement],
    attributes: &[Attribute],
) -> String {
    let service = raw_service_to_service(service, attributes);
    let mut result = Vec::new();

    if !attributes.is_empty() {
        result.push(attributes_to_string(&attributes));
    }

    result.push(format!("service {name} {{"));

    for Dependency { name, attributes } in &service.dependencies {
        if !attributes.is_empty() {
            result.push(format!("\t{}", attributes_to_string(&attributes)));
        }
        result.push(format!(
            "\tdepends on {};",
            namespace_to_string(name.clone())
        ))
    }

    result.push("".to_string());

    for (
        name,
        Function {
            arguments,
            return_type,
            attributes,
        },
    ) in &service.functions
    {
        if !attributes.is_empty() {
            result.push(format!("\t{}", attributes_to_string(&attributes)));
        }
        let arg_str = arguments
            .iter()
            .map(|(name, arg)| argument_to_string(name, arg))
            .collect::<Vec<_>>()
            .join(", ");
        if let Some(ret) = return_type {
            result.push(format!(
                "\tfn {name}({arg_str}) -> {};",
                namespace_to_string(ret.clone())
            ))
        } else {
            result.push(format!("\tfn {name}({arg_str});"))
        }
    }

    result.push("".to_string());

    for (
        name,
        Event {
            arguments,
            attributes,
        },
    ) in &service.events
    {
        if !attributes.is_empty() {
            result.push(format!("\t{}", attributes_to_string(&attributes)));
        }
        let arg_str = arguments
            .iter()
            .map(|(name, arg)| argument_to_string(name, arg))
            .collect::<Vec<_>>()
            .join(", ");
        result.push(format!("\tevent {name}({arg_str});"))
    }

    result.push(format!("}};"));
    result.join("\n")
}

pub fn pretty(raw: &[AstElement]) -> String {
    let mut first_element = true;
    let mut last_element_import = false;
    let mut result = Vec::new();
    for element in raw {
        match element {
            AstElement::Import(import) => {
                if !last_element_import && !first_element {
                    result.push("\n".to_owned());
                }
                if !import.attributes.is_empty() {
                    result.push(attributes_to_string(&import.attributes));
                }
                result.push(format!(
                    "import {};",
                    namespace_to_string(import.path.clone())
                ));
                last_element_import = true;
            }
            AstElement::DataType((name, dt)) => {
                if !first_element {
                    result.push("\n".to_owned());
                }
                result.push(datatype_to_string(name, &dt));
                last_element_import = false;
            }
            AstElement::Enum((name, en)) => {
                if !first_element {
                    result.push("\n".to_owned());
                }
                result.push(enum_to_string(name, &en));
                last_element_import = false;
            }
            AstElement::Service((name, svc, attributes)) => {
                if !first_element {
                    result.push("\n".to_owned());
                }
                result.push(service_to_string(name, svc, attributes));
                last_element_import = false;
            }
        }
        first_element = false;
    }
    return result.join("\n");
}
