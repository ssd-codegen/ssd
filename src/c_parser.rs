use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;

use crate::ast::{AstElement, ServiceAstElement};
use crate::parser::{ParseError, ParseErrorType};

use ssd_data::*;

#[repr(C)]
pub struct CAttributeParameter {
    pub key: *mut c_char,
    pub opt_value: *mut c_char,
    pub next: *mut CAttributeParameter,
}

#[repr(C)]
pub struct CAttribute {
    pub name: *mut c_char,
    pub opt_ll_arguments: *mut CAttributeParameter,
    pub next: *mut CAttribute,
}

#[repr(C)]
pub struct CType {
    pub name: *mut c_char,
    pub is_list: bool,
    pub count: *mut c_int,
}

#[repr(C)]
pub struct CProperty {
    pub attributes: *mut CAttribute,
    pub name: *mut c_char,
    pub r#type: *mut c_char,
    pub next: *mut CProperty,
}

#[repr(C)]
pub struct CEnumVariant {
    pub attributes: *mut CAttribute,
    pub name: *mut c_char,
    pub opt_value: *mut c_int,
    pub next: *mut CEnumVariant,
}

#[repr(C)]
pub struct CArgument {
    pub attributes: *mut CAttribute,
    pub name: *mut c_char,
    pub r#type: *mut c_char,
    pub next: *mut CArgument,
}

#[repr(C)]
pub struct CHandler {
    pub opt_ll_attributes: *mut CAttribute,
    pub name: *mut c_char,
    pub opt_ll_arguments: *mut CArgument,
    pub opt_return_type: *mut CType,
    pub next: *mut CHandler,
}

#[repr(C)]
pub struct CEvent {
    pub opt_ll_attributes: *mut CAttribute,
    pub name: *mut c_char,
    pub opt_ll_arguments: *mut CArgument,
    pub next: *mut CEvent,
}

#[repr(C)]
pub struct CDependency {
    pub opt_ll_attributes: *mut CAttribute,
    pub path: *mut c_char,
    pub next: *mut CDependency,
}

#[repr(C)]
pub struct CImport {
    pub path: *mut c_char,
}

#[repr(C)]
pub struct CData {
    pub name: *mut c_char,
    pub ll_properties: *mut CProperty,
}

#[repr(C)]
pub struct CEnum {
    pub name: *mut c_char,
    pub ll_variants: *mut CEnumVariant,
}

#[repr(C)]
pub struct CService {
    pub name: *mut c_char,
    pub opt_ll_dependencies: *mut CDependency,
    pub opt_ll_handlers: *mut CHandler,
    pub opt_ll_events: *mut CEvent,
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum CNodeType {
    NODE_IMPORT,
    NODE_DATA,
    NODE_ENUM,
    NODE_SERVICE,
}

#[repr(C)]
pub struct CAstNode {
    pub r#type: CNodeType,
    pub opt_ll_attributes: *mut CAttribute,
    pub node: CAstNodeUnion,
    pub next: *mut CAstNode,
}

#[repr(C)]
pub union CAstNodeUnion {
    pub import_node: std::mem::ManuallyDrop<CImport>,
    pub data_node: std::mem::ManuallyDrop<CData>,
    pub enum_node: std::mem::ManuallyDrop<CEnum>,
    pub service_node: std::mem::ManuallyDrop<CService>,
}

#[repr(C)]
pub struct CParser {
    pub input: *const c_char,
    pub input_length: usize,
    pub error: [c_char; 512],
    pub current: c_char,
    pub index: usize,
    pub line: c_int,
    pub column: c_int,
}

impl CParser {
    pub fn get_error_message(&self) -> String {
        // Safe because `error` is expected to be null-terminated in C
        unsafe {
            CStr::from_ptr(self.error.as_ptr())
                .to_string_lossy() // Handles invalid UTF-8 gracefully
                .into_owned() // Converts Cow<str> to String
        }
    }
}

#[link(name = "minissd", kind = "static")]
extern "C" {
    pub fn minissd_create_parser(input: *const c_char) -> *mut CParser;
    pub fn minissd_free_parser(p: *mut CParser);

    pub fn minissd_parse(p: *mut CParser) -> *mut CAstNode;
    pub fn minissd_free_ast(ast: *mut CAstNode);

    pub fn minissd_get_node_type(node: *const CAstNode) -> *const CNodeType;
    pub fn minissd_get_import_path(node: *const CAstNode) -> *const c_char;
    pub fn minissd_get_data_name(node: *const CAstNode) -> *const c_char;
    pub fn minissd_get_enum_name(node: *const CAstNode) -> *const c_char;
    pub fn minissd_get_service_name(node: *const CAstNode) -> *const c_char;
    pub fn minissd_get_properties(node: *const CAstNode) -> *const CProperty;
    pub fn minissd_get_enum_variants(node: *const CAstNode) -> *const CEnumVariant;
    pub fn minissd_get_dependencies(node: *const CAstNode) -> *const CDependency;
    pub fn minissd_get_handlers(node: *const CAstNode) -> *const CHandler;
    pub fn minissd_get_events(node: *const CAstNode) -> *const CEvent;
    pub fn minissd_get_attributes(node: *const CAstNode) -> *const CAttribute;
    pub fn minissd_get_next_node(node: *const CAstNode) -> *const CAstNode;

    pub fn minissd_get_handler_attributes(node: *const CHandler) -> *const CAttribute;
    pub fn minissd_get_handler_name(handler: *const CHandler) -> *const c_char;
    pub fn minissd_get_handler_return_type(handler: *const CHandler) -> *const CType;
    pub fn minissd_get_handler_arguments(handler: *const CHandler) -> *const CArgument;
    pub fn minissd_get_next_handler(handler: *const CHandler) -> *const CHandler;

    pub fn minissd_get_event_name(event: *const CEvent) -> *const c_char;
    pub fn minissd_get_event_arguments(event: *const CEvent) -> *const CArgument;
    pub fn minissd_get_next_event(event: *const CEvent) -> *const CEvent;

    pub fn minissd_get_dependency_path(dep: *const CDependency) -> *const c_char;
    pub fn minissd_get_next_dependency(dep: *const CDependency) -> *const CDependency;

    pub fn minissd_get_property_name(prop: *const CProperty) -> *const c_char;
    pub fn minissd_get_property_type(prop: *const CProperty) -> *const CType;
    pub fn minissd_get_property_attributes(prop: *const CProperty) -> *const CAttribute;

    pub fn minissd_get_next_property(prop: *const CProperty) -> *const CProperty;

    pub fn minissd_get_enum_variant_name(value: *const CEnumVariant) -> *const c_char;
    pub fn minissd_get_enum_variant_value(
        value: *const CEnumVariant,
        has_value: *mut bool,
    ) -> c_int;
    pub fn minissd_get_enum_variant_attributes(value: *const CEnumVariant) -> *const CAttribute;
    pub fn minissd_get_next_enum_variant(value: *const CEnumVariant) -> *const CEnumVariant;

    pub fn minissd_get_argument_name(arg: *const CArgument) -> *const c_char;
    pub fn minissd_get_argument_type(arg: *const CArgument) -> *const CType;
    pub fn minissd_get_argument_attributes(arg: *const CArgument) -> *const CAttribute;
    pub fn minissd_get_next_argument(arg: *const CArgument) -> *const CArgument;

    pub fn minissd_get_type_name(typ: *const CType) -> *const c_char;
    pub fn minissd_get_type_is_list(typ: *const CType) -> bool;
    pub fn minissd_get_type_count(typ: *const CType) -> *const c_int;

    pub fn minissd_get_attribute_name(attr: *const CAttribute) -> *const c_char;
    pub fn minissd_get_attribute_parameters(attr: *const CAttribute) -> *const CAttributeParameter;
    pub fn minissd_get_next_attribute(attr: *const CAttribute) -> *const CAttribute;

    pub fn minissd_get_attribute_parameter_name(arg: *const CAttributeParameter) -> *const c_char;
    pub fn minissd_get_attribute_parameter_value(arg: *const CAttributeParameter) -> *const c_char;
    pub fn minissd_get_next_attribute_parameter(
        arg: *const CAttributeParameter,
    ) -> *const CAttributeParameter;
}

fn get_attributes(c_attributes: *const CAttribute) -> Vec<Attribute> {
    let mut attributes = Vec::new();
    if !c_attributes.is_null() {
        let mut current_attr = c_attributes;
        while !current_attr.is_null() {
            let name = unsafe { minissd_get_attribute_name(current_attr) };
            let mut parameters = Vec::new();
            let mut c_parameters = unsafe { minissd_get_attribute_parameters(current_attr) };
            while !c_parameters.is_null() {
                let c_key = unsafe { minissd_get_attribute_parameter_name(c_parameters) };
                let c_value = unsafe { minissd_get_attribute_parameter_value(c_parameters) };

                let name = unsafe { CStr::from_ptr(c_key).to_str() }
                    .unwrap()
                    .to_owned();

                let value = if c_value.is_null() {
                    None
                } else {
                    let value = unsafe { CStr::from_ptr(c_value).to_str() }
                        .unwrap()
                        .to_owned();
                    Some(value)
                };

                parameters.push((name, value));
                c_parameters = unsafe { minissd_get_next_attribute_parameter(c_parameters) };
            }

            let attribute = Attribute::new(
                Namespace::new(unsafe { CStr::from_ptr(name).to_str() }.unwrap()),
                parameters,
            );
            attributes.push(attribute);

            current_attr = unsafe { minissd_get_next_attribute(current_attr) };
        }
    }
    return attributes;
}

fn get_span(parser: *const CParser) -> String {
    unsafe { format!("{}:{}", (*parser).line, (*parser).column) }
}

pub fn get_error(parser: *const CParser) -> Result<(), ParseError> {
    return Err(ParseError::from_c_parser(
        ParseErrorType::CParserError(unsafe { (*parser).get_error_message() }),
        &get_span(parser),
    ));
}

pub fn parse_raw(content: &str) -> Result<Vec<AstElement>, ParseError> {
    let c_str = std::ffi::CString::new(content).unwrap();
    let parser = unsafe { minissd_create_parser(c_str.into_raw() as *const c_char) };

    let c_ast = unsafe { minissd_parse(parser) };

    let mut result = Vec::new();
    let mut current = c_ast as *const CAstNode;

    if current.is_null() {
        return get_error(parser);
    }

    // while ast is not null
    while !current.is_null() {
        let node_type = unsafe { minissd_get_node_type(current) };

        match unsafe { *node_type } {
            CNodeType::NODE_IMPORT => {
                let c_attributes = unsafe { minissd_get_attributes(current) };
                let attributes = get_attributes(c_attributes);
                let path = unsafe { CStr::from_ptr(minissd_get_import_path(current)) }
                    .to_str()
                    .unwrap();
                result.push(AstElement::Import(Import::new(
                    Namespace::new(path),
                    attributes,
                )));
            }
            CNodeType::NODE_ENUM => {
                let c_attributes = unsafe { minissd_get_attributes(current) };
                let attributes = get_attributes(c_attributes);
                let name = unsafe { CStr::from_ptr(minissd_get_enum_name(current)) }
                    .to_str()
                    .unwrap()
                    .to_owned();
                let mut variants = Vec::new();
                let mut c_variants = unsafe { minissd_get_enum_variants(current) };
                while !c_variants.is_null() {
                    let name = unsafe { CStr::from_ptr(minissd_get_enum_variant_name(c_variants)) }
                        .to_str()
                        .unwrap()
                        .to_owned();

                    let attributes =
                        get_attributes(unsafe { minissd_get_enum_variant_attributes(c_variants) });

                    let mut has_value = false;
                    let value =
                        unsafe { minissd_get_enum_variant_value(c_variants, &mut has_value) };

                    variants.push((
                        name,
                        EnumValue::new(has_value.then_some(value.into()), attributes),
                    ));

                    c_variants = unsafe { minissd_get_next_enum_variant(c_variants) };
                }

                result.push(AstElement::Enum((name, Enum::new(variants, attributes))));
            }
            CNodeType::NODE_DATA => {
                let c_attributes = unsafe { minissd_get_attributes(current) };
                let attributes = get_attributes(c_attributes);
                let name = unsafe { CStr::from_ptr(minissd_get_data_name(current)) }
                    .to_str()
                    .unwrap()
                    .to_owned();

                let mut properties = OrderedMap::new();
                let mut c_properties = unsafe { minissd_get_properties(current) };
                while !c_properties.is_null() {
                    let name = unsafe { CStr::from_ptr(minissd_get_property_name(c_properties)) }
                        .to_str()
                        .unwrap()
                        .to_owned();
                    let c_type = unsafe { minissd_get_property_type(c_properties) };
                    let typ_name = unsafe { CStr::from_ptr(minissd_get_type_name(c_type)) }
                        .to_str()
                        .unwrap();

                    let attributes =
                        get_attributes(unsafe { minissd_get_property_attributes(c_properties) });

                    let is_list = unsafe { minissd_get_type_is_list(c_type) };
                    let count = if is_list {
                        let count = unsafe { minissd_get_type_count(c_type) };
                        if count.is_null() {
                            None
                        } else {
                            Some(unsafe { *count } as usize)
                        }
                    } else {
                        None
                    };

                    properties.push((
                        name,
                        TypeName::new(Namespace::new(typ_name), is_list, count, attributes),
                    ));

                    c_properties = unsafe { minissd_get_next_property(c_properties) };
                }

                result.push(AstElement::DataType((
                    name,
                    DataType::new(properties, attributes),
                )));
            }
            CNodeType::NODE_SERVICE => {
                let c_attributes = unsafe { minissd_get_attributes(current) };
                let attributes = get_attributes(c_attributes);
                let mut handlers = Vec::new();
                let mut c_handlers = unsafe { minissd_get_handlers(current) };
                while !c_handlers.is_null() {
                    let name = unsafe { CStr::from_ptr(minissd_get_handler_name(c_handlers)) }
                        .to_str()
                        .unwrap()
                        .to_owned();

                    let attributes =
                        get_attributes(unsafe { minissd_get_handler_attributes(c_handlers) });
                    let mut arguments = Vec::new();
                    let mut c_arguments = unsafe { minissd_get_handler_arguments(c_handlers) };
                    while !c_arguments.is_null() {
                        let name =
                            unsafe { CStr::from_ptr(minissd_get_argument_name(c_arguments)) }
                                .to_str()
                                .unwrap()
                                .to_owned();
                        let c_type = unsafe { minissd_get_argument_type(c_arguments) };

                        let typ_name = unsafe { CStr::from_ptr(minissd_get_type_name(c_type)) }
                            .to_str()
                            .unwrap();
                        let is_list = unsafe { minissd_get_type_is_list(c_type) };
                        let count = if is_list {
                            let count = unsafe { minissd_get_type_count(c_type) };
                            if count.is_null() {
                                None
                            } else {
                                Some(unsafe { *count } as usize)
                            }
                        } else {
                            None
                        };
                        let attributes =
                            get_attributes(unsafe { minissd_get_argument_attributes(c_arguments) });
                        arguments.push((
                            name,
                            TypeName::new(Namespace::new(&typ_name), is_list, count, attributes),
                        ));
                        c_arguments = unsafe { minissd_get_next_argument(c_arguments) };
                    }

                    let c_return_type = unsafe { minissd_get_handler_return_type(c_handlers) };
                    let return_type_name = if c_return_type.is_null() {
                        None
                    } else {
                        Some(
                            unsafe { CStr::from_ptr(minissd_get_type_name(c_return_type)) }
                                .to_str()
                                .unwrap()
                                .to_owned(),
                        )
                    };

                    let is_list = unsafe { minissd_get_type_is_list(c_return_type) };
                    let count = if is_list {
                        let count = unsafe { minissd_get_type_count(c_return_type) };
                        if count.is_null() {
                            None
                        } else {
                            Some(unsafe { *count } as usize)
                        }
                    } else {
                        None
                    };

                    handlers.push(ServiceAstElement::Function((
                        name,
                        Function::new(
                            arguments,
                            return_type_name.map(|rt| {
                                TypeName::new(Namespace::new(&rt), is_list, count, vec![])
                            }),
                            attributes,
                        ),
                    )));
                    c_handlers = unsafe { minissd_get_next_handler(c_handlers) };
                }
                result.push(AstElement::Service((
                    unsafe { CStr::from_ptr(minissd_get_service_name(current)) }
                        .to_str()
                        .unwrap()
                        .to_owned(),
                    handlers,
                    attributes,
                )));
            }
        }

        current = unsafe { minissd_get_next_node(current) };
    }

    unsafe { minissd_free_ast(c_ast) };
    unsafe { minissd_free_parser(parser) };

    Ok(result)
}
