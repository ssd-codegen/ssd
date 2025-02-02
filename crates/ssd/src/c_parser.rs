use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;

use crate::ast::AstElement;
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
    pub opt_return_type: *mut c_char,
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

#[link(name = "minissd")]
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

    pub fn minissd_get_handler_name(handler: *const CHandler) -> *const c_char;
    pub fn minissd_get_handler_return_type(handler: *const CHandler) -> *const c_char;
    pub fn minissd_get_handler_arguments(handler: *const CHandler) -> *const CArgument;
    pub fn minissd_get_next_handler(handler: *const CHandler) -> *const CHandler;

    pub fn minissd_get_event_name(event: *const CEvent) -> *const c_char;
    pub fn minissd_get_event_arguments(event: *const CEvent) -> *const CArgument;
    pub fn minissd_get_next_event(event: *const CEvent) -> *const CEvent;

    pub fn minissd_get_dependency_path(dep: *const CDependency) -> *const c_char;
    pub fn minissd_get_next_dependency(dep: *const CDependency) -> *const CDependency;

    pub fn minissd_get_property_name(prop: *const CProperty) -> *const c_char;
    pub fn minissd_get_property_type(prop: *const CProperty) -> *const c_char;
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
    pub fn minissd_get_argument_type(arg: *const CArgument) -> *const c_char;
    pub fn minissd_get_argument_attributes(arg: *const CArgument) -> *const CAttribute;
    pub fn minissd_get_next_argument(arg: *const CArgument) -> *const CArgument;

    pub fn minissd_get_attribute_name(attr: *const CAttribute) -> *const c_char;
    pub fn minissd_get_attribute_parameters(attr: *const CAttribute) -> *const CAttributeParameter;
    pub fn minissd_get_next_attribute(attr: *const CAttribute) -> *const CAttribute;

    pub fn minissd_get_attribute_parameter_name(arg: *const CAttributeParameter) -> *const c_char;
    pub fn minissd_get_attribute_parameter_value(arg: *const CAttributeParameter) -> *const c_char;
    pub fn minissd_get_next_attribute_parameter(
        arg: *const CAttributeParameter,
    ) -> *const CAttributeParameter;
}

pub fn parse_raw(content: &str) -> Result<Vec<AstElement>, ParseError> {
    let c_str = std::ffi::CString::new(content).unwrap();
    let parser = unsafe { minissd_create_parser(c_str.into_raw() as *const c_char) };

    let ast = unsafe { minissd_parse(parser) };

    let mut result = Vec::new();
    let mut current = ast as *const CAstNode;

    if current.is_null() {
        unsafe {
            println!("{}", (*parser).get_error_message());
        }
    }

    // while ast is not null
    while !dbg!(current.is_null()) {
        let node_type = unsafe { minissd_get_node_type(current) };

        match unsafe { *node_type } {
            CNodeType::NODE_IMPORT => {
                let c_attributes = unsafe { minissd_get_attributes(current) };
                if (!c_attributes.is_null()) {
                    let mut current_attr = c_attributes;
                    let mut attributes = Vec::new();
                    while !current_attr.is_null() {
                        let name = unsafe { minissd_get_attribute_name(current_attr) };
                        let mut parameters =
                            unsafe { minissd_get_attribute_parameters(current_attr) };
                        while !parameters.is_null() {
                            let key = unsafe { minissd_get_attribute_parameter_name(parameters) };
                            let value =
                                unsafe { minissd_get_attribute_parameter_value(parameters) };

                            if value.is_null() {
                                println!(
                                    "ATTRIBUTE: {:?} {:?}",
                                    unsafe { CStr::from_ptr(key).to_str() },
                                    "None"
                                );
                            } else {
                                println!(
                                    "ATTRIBUTE: {:?} {:?}",
                                    unsafe { CStr::from_ptr(key).to_str() },
                                    unsafe { CStr::from_ptr(value).to_str() }
                                );
                            }
                            parameters =
                                unsafe { minissd_get_next_attribute_parameter(parameters) };
                        }

                        let attribute = Attribute::new(
                            Namespace::new(unsafe { CStr::from_ptr(name).to_str() }.unwrap()),
                            Vec::new(),
                        );
                        attributes.push(attribute);

                        current_attr = unsafe { minissd_get_next_attribute(current_attr) };
                    }
                    println!("ATTRIBUTES: {:?}", attributes);
                }
                let path = unsafe { minissd_get_import_path(current) };
                println!("NODE_IMPORT: {:?}", unsafe {
                    CStr::from_ptr(path).to_str()
                });
            }
            CNodeType::NODE_DATA => println!("NODE_DATA"),
            CNodeType::NODE_ENUM => println!("NODE_ENUM"),
            CNodeType::NODE_SERVICE => println!("NODE_SERVICE"),
        }

        current = unsafe { minissd_get_next_node(current) };
    }

    unsafe { minissd_free_ast(ast) };
    unsafe { minissd_free_parser(parser) };

    Ok(result)
}
