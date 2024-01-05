use std::path::PathBuf;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use ssd_data::{Namespace, TypeName, SsdModule};

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(untagged)]
enum StringOrVec {
    String(String),
    Vec(Vec<String>),
}

pub fn print_or_write(out: Option<PathBuf>, result: &str) -> anyhow::Result<()> {
    if let Some(out) = out {
        std::fs::write(out, result)?;
    } else {
        println!("{result}");
    }
    Ok(())
}

pub fn parse_raw_data(file: PathBuf) -> anyhow::Result<serde_value::Value> {
    let content = std::fs::read_to_string(file)?;
    let result = serde_json::from_str(&content)
        .or_else(|_| toml::from_str(&content))
        .or_else(|_| serde_yaml::from_str(&content))
        .or_else(|_| rsn::from_str(&content));
    #[cfg(feature = "ron")]
    let result = result.or_else(|_| ron::from_str(&content));
    Ok(result?)
}

pub fn update_types(mut module: SsdModule, typemap: &str) -> anyhow::Result<SsdModule> {
    let mappings: HashMap<StringOrVec, StringOrVec> =
        toml::from_str(typemap)?;
    let mappings: HashMap<String, String> = mappings
        .iter()
        .map(|(k, v)| match (k, v) {
            (StringOrVec::Vec(k), StringOrVec::Vec(v)) => (k.join("::"), v.join("::")),
            (StringOrVec::Vec(k), StringOrVec::String(v)) => (k.join("::"), v.clone()),
            (StringOrVec::String(k), StringOrVec::Vec(v)) => (k.clone(), v.join("::")),
            (StringOrVec::String(k), StringOrVec::String(v)) => (k.clone(), v.clone()),
        })
        .collect();
    for (_dt_name, dt) in &mut module.data_types {
        for (_name, prop) in &mut dt.properties {
            let name = prop.typ.to_string();
            if let Some(v) = mappings.get(&name) {
                prop.typ = Namespace::new(v);
            }
        }
    }

    for (_service_name, service) in &mut module.services {
        for (_handler_name, h) in &mut service.functions {
            if let Some(TypeName {
                typ,
                is_list,
                count,
                attributes,
                comments,
            }) = &h.return_type
            {
                let name = typ.to_string();
                let mut comments = comments.clone();
                if let Some(v) = mappings.get(&name) {
                    h.return_type = Some(
                        TypeName::new(Namespace::new(v), *is_list, *count, attributes.clone())
                            .with_comments(&mut comments),
                    );
                }
            }
            for (_arg_name, arg) in &mut h.arguments {
                let name = arg.typ.to_string();
                if let Some(v) = mappings.get(&name) {
                    arg.typ = Namespace::new(v);
                }
            }
        }
        for (_event_name, h) in &mut service.events {
            for (_arg_name, arg) in &mut h.arguments {
                let name = arg.typ.to_string();
                if let Some(v) = mappings.get(&name) {
                    arg.typ = Namespace::new(v);
                }
            }
        }
    }

    Ok(module)
}

pub fn update_types_from_file(
    mut module: SsdModule,
    no_map: bool,
    typemap: Option<PathBuf>,
    script: Option<&PathBuf>,
) -> anyhow::Result<SsdModule> {
    if let (false, Some(map_file)) = (
        no_map,
        typemap.or_else(|| {
            script.and_then(|script| {
                let mut typemap = script.clone();
                typemap.set_extension("tym");
                typemap.exists().then_some(typemap)
            })
        }),
    ) {
        let mappings: HashMap<StringOrVec, StringOrVec> =
            toml::from_str(&std::fs::read_to_string(map_file)?)?;
        let mappings: HashMap<String, String> = mappings
            .iter()
            .map(|(k, v)| match (k, v) {
                (StringOrVec::Vec(k), StringOrVec::Vec(v)) => (k.join("::"), v.join("::")),
                (StringOrVec::Vec(k), StringOrVec::String(v)) => (k.join("::"), v.clone()),
                (StringOrVec::String(k), StringOrVec::Vec(v)) => (k.clone(), v.join("::")),
                (StringOrVec::String(k), StringOrVec::String(v)) => (k.clone(), v.clone()),
            })
            .collect();
        for (_dt_name, dt) in &mut module.data_types {
            for (_name, prop) in &mut dt.properties {
                let name = prop.typ.to_string();
                if let Some(v) = mappings.get(&name) {
                    prop.typ = Namespace::new(v);
                }
            }
        }

        for (_service_name, service) in &mut module.services {
            for (_handler_name, h) in &mut service.functions {
                if let Some(TypeName {
                    typ,
                    is_list,
                    count,
                    attributes,
                    comments,
                }) = &h.return_type
                {
                    let name = typ.to_string();
                    let mut comments = comments.clone();
                    if let Some(v) = mappings.get(&name) {
                        h.return_type = Some(
                            TypeName::new(Namespace::new(v), *is_list, *count, attributes.clone())
                                .with_comments(&mut comments),
                        );
                    }
                }
                for (_arg_name, arg) in &mut h.arguments {
                    let name = arg.typ.to_string();
                    if let Some(v) = mappings.get(&name) {
                        arg.typ = Namespace::new(v);
                    }
                }
            }
            for (_event_name, h) in &mut service.events {
                for (_arg_name, arg) in &mut h.arguments {
                    let name = arg.typ.to_string();
                    if let Some(v) = mappings.get(&name) {
                        arg.typ = Namespace::new(v);
                    }
                }
            }
        }
    }

    Ok(module)
}