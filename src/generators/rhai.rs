use crate::options::{BaseInputData, BaseOutputData};
use crate::parse_raw_data;
use clap::Parser;
use ssd_data::{Namespace, SsdModule};
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

use ssd::parse_file;

use crate::{print_or_write, update_types};

use crate::ast::{
    Attribute, DataType, Dependency, Enum, EnumValue, Event, Function, Import, OrderedMap,
    Parameter, Service, TypeName,
};
use faccess::PathExt;
use glob::glob;
use rhai::packages::{CorePackage, Package};
use rhai::{Array, Dynamic, Engine, EvalAltResult, ImmutableString, Map, Scope, FLOAT, INT};
use std::path::Path;
use std::{any::TypeId, cell::RefCell, rc::Rc, time::Instant};

const INDENT: &str = "    ";

type ScriptResult<T> = Result<T, Box<EvalAltResult>>;

#[derive(Debug, Parser)]
pub struct Parameters {
    /// The script to use to generate the file.
    pub script: PathBuf,
    #[clap(long, short)]
    /// Enables debug mode (print and debug function in the script).
    pub debug: bool,
    #[clap(flatten)]
    pub input: BaseInputData,
    #[clap(flatten)]
    pub out: BaseOutputData,
}

#[allow(clippy::unnecessary_box_returns)]
fn error_to_runtime_error<E: std::error::Error>(e: E) -> Box<EvalAltResult> {
    e.to_string().into()
}

use rhai::Token;

fn script_exists(path: &str) -> bool {
    PathBuf::from(path).exists()
}

fn script_is_file(path: &str) -> bool {
    PathBuf::from(path).is_file()
}

fn script_is_dir(path: &str) -> bool {
    PathBuf::from(path).is_dir()
}

#[allow(clippy::needless_pass_by_value)]
fn script_is_some<T>(opt: Option<T>) -> bool {
    opt.is_some()
}

fn script_unwrap<T>(opt: Option<T>) -> T {
    opt.unwrap()
}

fn script_unwrap_string_or(opt: Option<Namespace>, default: String) -> String {
    opt.map_or(default, |n| n.to_string())
}

fn script_unwrap_or<T>(opt: Option<T>, default: T) -> T {
    opt.unwrap_or(default)
}

fn script_join(v: &[String], sep: &str) -> String {
    v.join(sep)
}

fn script_join_typepath(v: Namespace, sep: &str) -> String {
    v.into_iter().collect::<Vec<_>>().join(sep)
}

fn script_is_executable(path: &str) -> bool {
    Path::new(path).executable()
}

fn script_find_paths(pattern: &str) -> ScriptResult<Vec<Dynamic>> {
    glob(pattern)
        .map_err(error_to_runtime_error)?
        .filter_map(|e| match e {
            Ok(path) => {
                if let Some(s) = path.to_str() {
                    Some(Ok(s.into()))
                } else {
                    eprintln!("file path is not valid UTF-8 string: {path:?}");
                    None
                }
            }
            Err(e) => {
                eprintln!("glob error: {e}");
                None
            }
        })
        .collect()
}

fn script_split(s: &str, pattern: &str) -> Vec<Dynamic> {
    s.split(pattern)
        .map(|s| Dynamic::from(s.to_string()))
        .collect()
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn script_splitn(s: &str, n: INT, pattern: &str) -> Vec<Dynamic> {
    s.splitn(n as usize, pattern)
        .map(|s| Dynamic::from(s.to_string()))
        .collect()
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn script_rsplitn(s: &str, n: INT, pattern: &str) -> Vec<Dynamic> {
    s.rsplitn(n as usize, pattern)
        .map(|s| Dynamic::from(s.to_string()))
        .collect()
}

fn script_read_file(path: &str) -> ScriptResult<String> {
    std::fs::read_to_string(path).map_err(error_to_runtime_error)
}

fn script_string_is_empty(s: &str) -> bool {
    s.is_empty()
}

fn script_array_is_empty(s: &Array) -> bool {
    s.is_empty()
}

fn script_starts_with(s: &str, pat: &str) -> bool {
    s.starts_with(pat)
}

fn script_ends_with(s: &str, pat: &str) -> bool {
    s.ends_with(pat)
}

fn script_trim(s: &str) -> &str {
    s.trim()
}

fn script_is_no_string(_: Dynamic) -> bool {
    false
}

fn script_is_string(_: &str) -> bool {
    true
}

fn script_any(arr: &Array) -> ScriptResult<bool> {
    if arr.iter().all(rhai::Dynamic::is::<bool>) {
        Ok(arr.iter().any(|b| b.as_bool().unwrap()))
    } else {
        Err("any only takes bool values".into())
    }
}

fn script_all(arr: &Array) -> ScriptResult<bool> {
    if arr.iter().all(rhai::Dynamic::is::<bool>) {
        Ok(arr.iter().all(|b| b.as_bool().unwrap()))
    } else {
        Err("all only takes bool values".into())
    }
}

fn script_none(arr: &Array) -> ScriptResult<bool> {
    if arr.iter().all(rhai::Dynamic::is::<bool>) {
        Ok(!arr.iter().any(|b| b.as_bool().unwrap()))
    } else {
        Err("none only takes bool values".into())
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn script_require(arr: &Array, n: INT) -> ScriptResult<bool> {
    if arr.iter().all(rhai::Dynamic::is::<bool>) {
        Ok(arr.iter().filter(|b| b.as_bool().unwrap()).count() == n as usize)
    } else {
        Err("none only takes bool values".into())
    }
}

fn script_map_equals(m1: &Map, m2: &Map) -> ScriptResult<bool> {
    if m1.len() != m2.len() {
        return Ok(false);
    }
    for (key, value) in m1 {
        if let Some(value2) = m2.get(key) {
            if !script_value_equals(value.clone(), value2.clone())? {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }
    }
    Ok(true)
}

fn script_string_contains(s: &str, v: &str) -> bool {
    s.contains(v)
}

fn script_map_contains(m: &Map, name: &str) -> bool {
    m.get(name).is_some()
}

fn script_value_equals(v1: Dynamic, v2: Dynamic) -> ScriptResult<bool> {
    let t1 = v1.type_id();
    let t2 = v2.type_id();
    if t1 != t2 {
        Ok(false)
    } else if t1 == TypeId::of::<()>() {
        Ok(true)
    } else if t1 == TypeId::of::<bool>() {
        Ok(v1.as_bool() == v2.as_bool())
    } else if t1 == TypeId::of::<ImmutableString>() {
        Ok(v1.into_immutable_string() == v2.into_immutable_string())
    } else if t1 == TypeId::of::<char>() {
        Ok(v1.as_char() == v2.as_char())
    } else if t1 == TypeId::of::<INT>() {
        Ok(v1.as_int() == v2.as_int())
    } else if t1 == TypeId::of::<FLOAT>() {
        Ok(v1.as_float() == v2.as_float())
    } else if t1 == TypeId::of::<Array>() {
        Ok(script_array_equals(
            &v1.cast::<Array>(),
            &v2.cast::<Array>(),
        ))
    } else if t1 == TypeId::of::<Map>() {
        script_map_equals(&v1.cast::<Map>(), &v2.cast::<Map>())
    } else if t1 == TypeId::of::<Instant>() {
        Ok(v1.cast::<Instant>() == v2.cast::<Instant>())
    } else {
        Err("unsupported type".into())
    }
}

fn script_array_equals(arr: &Array, arr2: &Array) -> bool {
    if arr.len() != arr2.len() {
        return false;
    }
    let result = arr
        .iter()
        .zip(arr2.iter())
        .all(|(e1, e2)| script_value_equals(e1.clone(), e2.clone()).unwrap_or_default());
    result
}

fn script_array_contains(arr: Array, v: &Dynamic) -> bool {
    arr.into_iter()
        .any(|ele| script_value_equals(ele, v.clone()).unwrap_or_default())
}

#[allow(clippy::too_many_lines)]
pub fn build_engine(messages: Rc<RefCell<Vec<String>>>,  debug: bool) -> Engine {
    let mut engine = Engine::new();
    // Register a token mapper function to allow module as identifier name
    #[allow(deprecated)]
    engine.on_parse_token(|token, _pos, _state| {
        match token {
            // Change 'begin' ... 'end' to '{' ... '}'
            Token::Reserved(s) if s.as_str() == "module" => {
                Token::Identifier(Box::new(rhai::Identifier::from("module".to_string())))
            }

            // Pass through all other tokens unchanged
            _ => token,
        }
    });

    let package = CorePackage::new();

    // Register the package into the 'Engine' by converting it into a shared module.
    engine.register_global_module(package.as_shared_module());

    engine.register_iterator::<Vec<serde_value::Value>>();

    engine
        .register_iterator::<Vec<SsdModule>>()
        .register_iterator::<Vec<Import>>()
        .register_iterator::<OrderedMap<Namespace>>()
        .register_iterator::<Namespace>()
        .register_iterator::<OrderedMap<Enum>>()
        .register_iterator::<OrderedMap<EnumValue>>()
        .register_iterator::<OrderedMap<DataType>>()
        .register_iterator::<OrderedMap<Service>>()
        .register_iterator::<Vec<Attribute>>()
        .register_iterator::<OrderedMap<TypeName>>()
        .register_iterator::<Vec<Dependency>>()
        .register_iterator::<Vec<Parameter>>()
        .register_iterator::<OrderedMap<Function>>();

    engine.register_fn("to_string", |this: &mut Import| this.path.clone());
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    engine.register_fn("NL", |count: i64| "\n".repeat(count as usize));
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    engine.register_fn("IND", move |count: i64| INDENT.repeat(count as usize));

    #[allow(clippy::items_after_statements)]
    fn script_first<A: Clone, B>(tuple: &mut (A, B)) -> A {
        tuple.0.clone()
    }

    #[allow(clippy::items_after_statements)]
    fn script_second<A, B: Clone>(tuple: &mut (A, B)) -> B {
        tuple.1.clone()
    }

    macro_rules! register_pairs {
        ($(($A: ty, $B: ty)),*) => {
            $(
            engine
                .register_type::<($A, $B)>()
                .register_get("first", script_first::<$A, $B>)
                .register_get("second", script_second::<$A, $B>);
            )*
        };
    }

    macro_rules! register_string_pairs {
        ($($B: ty),*) => {
            $(
            register_pairs!((String, $B));
            )*
        };
    }

    register_string_pairs!(
        Enum,
        DataType,
        Service,
        Function,
        TypeName,
        EnumValue,
        Option<EnumValue>
    );

    // engine
    //     .register_type::<serde_value::Value>()
    //     .register_indexer_get(|&mut this, name| match this {
    //         serde_value::Value::Map(map) => map[name],
    //         _ => panic!("Cannot index into non-object"),
    //     });

    engine
        .register_type::<SsdModule>()
        .register_get("name", SsdModule::namespace)
        .register_get("imports", SsdModule::imports)
        .register_get("data_types", SsdModule::data_types)
        .register_get("enums", SsdModule::enums)
        .register_get("services", SsdModule::services);

    engine
        .register_type::<Import>()
        .register_get("path", Import::path)
        .register_get("attributes", Import::attributes);

    engine
        .register_type::<DataType>()
        .register_get("properties", DataType::properties)
        .register_get("attributes", DataType::attributes);

    engine
        .register_type::<Enum>()
        .register_get("values", Enum::values)
        .register_get("attributes", Enum::attributes);

    engine
        .register_type::<Service>()
        .register_get("dependencies", Service::dependencies)
        .register_get("functions", Service::functions)
        .register_get("handlers", Service::handlers)
        .register_get("events", Service::events)
        .register_get("attributes", Service::attributes);

    engine
        .register_type::<Dependency>()
        .register_get("name", Dependency::name)
        .register_get("attributes", Dependency::attributes);

    engine
        .register_type::<Function>()
        .register_get("arguments", Function::arguments)
        .register_get("return_type", Function::return_type)
        .register_get("attributes", Function::attributes);

    engine
        .register_type::<Event>()
        .register_get("arguments", Event::arguments)
        .register_get("attributes", Event::attributes);

    engine
        .register_type::<TypeName>()
        .register_get("typ", TypeName::typ)
        .register_get("is_list", TypeName::is_list)
        .register_get("count", TypeName::count)
        .register_get("attributes", TypeName::attributes);

    engine
        .register_type::<EnumValue>()
        .register_get("value", EnumValue::value)
        .register_get("attributes", EnumValue::attributes);

    engine
        .register_type::<Attribute>()
        .register_get("name", Attribute::name)
        .register_get("parameters", Attribute::parameters);

    engine
        .register_type::<Parameter>()
        .register_get("name", Parameter::name)
        .register_get("value", Parameter::value);

    engine
        .register_type::<Namespace>()
        .register_get("components", Namespace::components);

    macro_rules! register_options {
        ($($T: ty),*) => {
            $(
            engine
                .register_fn("is_some", script_is_some::<$T>)
                .register_fn("unwrap", script_unwrap::<$T>)
                .register_fn("unwrap_or", script_unwrap_or::<$T>);
            )*
        };
    }

    register_options!(
        String, i64, u64, i32, u32, i16, u16, i8, u8, usize, isize, i128, u128, TypeName
    );

    engine
        .register_fn("unwrap_or", script_unwrap_string_or)
        .register_fn("is_dir", script_is_dir)
        .register_fn("is_file", script_is_file)
        .register_fn("is_executable", script_is_executable)
        .register_fn("exists", script_exists)
        .register_fn("join", script_join)
        .register_fn("join", script_join_typepath)
        .register_fn("split", script_split)
        .register_fn("splitn", script_splitn)
        .register_fn("rsplitn", script_rsplitn)
        .register_fn("is_empty", script_string_is_empty)
        .register_fn("is_empty", script_array_is_empty)
        .register_fn("starts_with", script_starts_with)
        .register_fn("ends_with", script_ends_with)
        .register_fn("trim", script_trim)
        .register_fn("is_string", script_is_no_string)
        .register_fn("is_string", script_is_string)
        .register_fn("find_paths", script_find_paths)
        .register_fn("read_file", script_read_file);

    // DSL
    engine
        .register_custom_operator("and", 60)
        .unwrap()
        .register_fn("and", |a: bool, b: bool| a && b)
        .register_custom_operator("or", 30)
        .unwrap()
        .register_fn("or", |a: bool, b: bool| a || b)
        .register_custom_operator("xor", 30)
        .unwrap()
        .register_fn("xor", |a: bool, b: bool| a ^ b)
        .register_custom_operator("contains", 15)
        .unwrap()
        .register_custom_operator("equals", 15)
        .unwrap()
        .register_custom_operator("require", 15)
        .unwrap()
        .register_fn("contains", script_map_contains)
        .register_fn("contains", script_string_contains)
        .register_fn("equals", script_map_equals)
        .register_fn("equals", script_value_equals)
        .register_fn("equals", script_array_equals)
        .register_fn("contains", script_array_contains)
        .register_fn("require", script_require)
        .register_fn("any", script_any)
        .register_fn("all", script_all)
        .register_fn("none", script_none);

    {
        let messages = messages.clone();
        engine.register_fn("-", move |msg: &str| {
            messages.borrow_mut().push(msg.to_owned());
        });
    }
    {
        let messages = messages.clone();
        engine.register_fn("++", move |a: &str, b: &str| {
            messages.borrow_mut().push(a.to_owned());
            messages.borrow_mut().push(b.to_owned());
        });
    }
    {
        let messages = messages.clone();
        engine.register_fn("++", move |a: &str, b: usize| {
            messages.borrow_mut().push(a.to_owned());
            messages.borrow_mut().push(b.to_string());
        });
    }

    macro_rules! register_string_concat_void {
        ($($T: ty),*) => {$({
            let messages = messages.clone();
            engine.register_fn("++", move |a: $T, _b: ()| {
                messages.borrow_mut().push(a.to_string());
            });
        }
        {
            let messages = messages.clone();
            engine.register_fn("++", move |_a: (), b: $T| {
                messages.borrow_mut().push(b.to_string());
            });
        }
        )*};
    }

    macro_rules! register_string_concat {
        ($($T: ty),*) => {$({
            let messages = messages.clone();
            engine.register_fn("++", move |a: $T, b: &str| {
                messages.borrow_mut().push(a.to_string());
                messages.borrow_mut().push(b.to_owned());
            });
        }
        {
            let messages = messages.clone();
            engine.register_fn("++", move |a: &str, b: $T| {
                messages.borrow_mut().push(a.to_owned());
                messages.borrow_mut().push(b.to_string());
            });
        }
        {
            let messages = messages.clone();
            engine.register_fn("++", move |a: $T, b: $T| {
                messages.borrow_mut().push(a.to_string());
                messages.borrow_mut().push(b.to_string());
            });
        })*};
    }

    macro_rules! register_string_concat_vec {
        ($($T: ty),*) => {$({
            let messages = messages.clone();
            engine.register_fn("++", move |a: &Vec<$T>, b: &str| {
                messages.borrow_mut().push(format!("{:?}", a));
                messages.borrow_mut().push(b.to_owned());
            });
        }
        {
            let messages = messages.clone();
            engine.register_fn("++", move |a: &str, b: &Vec<$T>| {
                messages.borrow_mut().push(a.to_owned());
                messages.borrow_mut().push(format!("{:?}", b));
            });
        }
        {
            let messages = messages.clone();
            engine.register_fn("++", move |a: &Vec<$T>, b: &Vec<$T>| {
                messages.borrow_mut().push(format!("{:?}", a));
                messages.borrow_mut().push(format!("{:?}", b));
            });
        })*};
    }

    macro_rules! register_concat {
        ($($T: ty),*) => {{
            register_string_concat!($($T),*);
            register_string_concat_vec!($($T),*);
            register_string_concat_void!($($T),*);
        }};
    }

    register_concat!(i32, u32, i64, u64, f32, f64, bool);

    {
        let messages = messages.clone();
        engine.register_fn("++", move |(): (), b: &str| {
            messages.borrow_mut().push(b.to_owned());
        });
    }
    {
        let messages = messages.clone();
        engine.register_fn("++", move |(): (), b: usize| {
            messages.borrow_mut().push(b.to_string());
        });
    }
    engine.register_custom_operator("++", 15).unwrap();
    {
        let messages = messages.clone();
        engine.register_fn("emit", move |msg: &str| {
            messages.borrow_mut().push(msg.to_owned());
        });
    }
    engine.register_custom_operator("then_emit", 15).unwrap();
    {
        let messages = messages.clone();
        engine.register_fn("then_emit", move |a: bool, msg: &str| {
            if a {
                messages.borrow_mut().push(msg.to_owned());
            }
            a
        });
    }
    {
        let messages = messages.clone();
        engine.register_fn("then_emit", move |a: bool, m: Map| {
            if a {
                let msg = m
                    .get("msg")
                    .map(|e| e.clone().into_string().unwrap())
                    .unwrap();
                messages.borrow_mut().push(msg);
            }
            a
        });
    }
    engine.register_custom_operator("or_emit", 15).unwrap();
    {
        let messages = messages.clone();
        engine.register_fn("or_emit", move |a: bool, msg: &str| {
            if !a {
                messages.borrow_mut().push(msg.to_owned());
            }
            a
        });
    }
    {
        engine.register_fn("or_emit", move |a: bool, m: Map| {
            if !a {
                let msg = m
                    .get("msg")
                    .map(|e| e.clone().into_string().unwrap())
                    .unwrap();
                messages.borrow_mut().push(msg);
            }
            a
        });
    }
    // END DSL

    if debug {
        engine.on_print(move |x| eprintln!("INFO => {x}"));
        engine.on_debug(move |x, _, pos| eprintln!("DEBUG({pos:?}) => {x}"));
    } else {
        engine.on_print(|_| ());
        engine.on_debug(|_, _, _| ());
    }

    engine.disable_symbol("eval");

    engine
}

pub fn generate(
    base: &PathBuf,
    defines: HashMap<String, String>,
    Parameters {
        input,
        debug,
        script,
        out,
    }: Parameters,
) -> Result<(), Box<dyn Error>> {
    let messages = Rc::new(RefCell::new(Vec::new()));

    let engine = build_engine(messages.clone(), debug);

    let mut scope = Scope::new();
    if input.raw {
        let module = parse_raw_data(input.file)?;

        scope.push("module", module);
    } else {
        let module = parse_file(base, &input.file)?;
        let module = update_types(module, input.no_map, input.typemap, Some(&script))?;

        scope.push("module", module);
    };
    scope.push_constant("defines", defines);
    scope.push_constant("NL", "\n");
    scope.push_constant("IND", INDENT);
    engine.run_file_with_scope(&mut scope, script)?;
    let messages = messages.borrow();
    if !messages.is_empty() {
        let result = messages.join("");
        print_or_write(out.out, &result)?;
    }
    Ok(())
}