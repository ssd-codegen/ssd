mod ast;
mod map_vec;
mod options;
mod parser;

use std::{any::TypeId, cell::RefCell, path::PathBuf, rc::Rc, time::Instant};

use crate::ast::{Namespace, Parameter, SsdcFile};
use crate::options::{Command, Options};
use ast::{Attribute, DataType, Dependency, Handler, Import, NameTypePair, Service};
use glob::glob;
use rhai::packages::{CorePackage, Package};
use rhai::{Array, Dynamic, Engine, EvalAltResult, ImmutableString, Map, Scope, FLOAT, INT};

use structopt::StructOpt;

type ScriptResult<T> = Result<T, Box<EvalAltResult>>;

fn error_to_runtime_error<E: std::error::Error>(e: E) -> Box<EvalAltResult> {
    e.to_string().into()
}

fn build_engine(messages: Rc<RefCell<Vec<String>>>, indent: String, debug: bool) -> Engine {
    fn script_exists(path: &str) -> bool {
        PathBuf::from(path).exists()
    }

    fn script_is_file(path: &str) -> bool {
        PathBuf::from(path).is_file()
    }

    fn script_is_dir(path: &str) -> bool {
        PathBuf::from(path).is_dir()
    }

    fn script_is_some(opt: Option<String>) -> bool {
        opt.is_some()
    }

    fn script_unwrap(opt: Option<String>) -> String {
        opt.unwrap()
    }

    fn script_unwrap_or(opt: Option<String>, default: String) -> String {
        opt.unwrap_or(default)
    }

    fn script_join(v: Vec<String>, sep: &str) -> String {
        v.join(sep)
    }

    fn script_join_typepath(v: Namespace, sep: &str) -> String {
        v.into_iter().collect::<Vec<_>>().join(sep)
    }

    fn script_is_executable(path: &str) -> bool {
        permissions::is_executable(path).unwrap_or(false)
    }

    fn script_find_paths(pattern: &str) -> ScriptResult<Vec<Dynamic>> {
        glob(pattern)
            .map_err(error_to_runtime_error)?
            .filter_map(|e| match e {
                Ok(path) => {
                    if let Some(s) = path.to_str() {
                        Some(Ok(s.into()))
                    } else {
                        eprintln!("file path is not valid UTF-8 string: {:?}", path);
                        None
                    }
                }
                Err(e) => {
                    eprintln!("glob error: {}", e);
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

    fn script_splitn(s: &str, n: INT, pattern: &str) -> Vec<Dynamic> {
        s.splitn(n as usize, pattern)
            .map(|s| Dynamic::from(s.to_string()))
            .collect()
    }

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

    fn script_array_is_empty(s: Array) -> bool {
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

    fn script_any(arr: Array) -> ScriptResult<bool> {
        if !arr.iter().all(|b| b.is::<bool>()) {
            Err("any only takes bool values".into())
        } else {
            Ok(arr.iter().any(|b| b.as_bool().unwrap()))
        }
    }

    fn script_all(arr: Array) -> ScriptResult<bool> {
        if !arr.iter().all(|b| b.is::<bool>()) {
            Err("all only takes bool values".into())
        } else {
            Ok(arr.iter().all(|b| b.as_bool().unwrap()))
        }
    }

    fn script_none(arr: Array) -> ScriptResult<bool> {
        if !arr.iter().all(|b| b.is::<bool>()) {
            Err("none only takes bool values".into())
        } else {
            Ok(!arr.iter().any(|b| b.as_bool().unwrap()))
        }
    }

    fn script_require(arr: Array, n: INT) -> ScriptResult<bool> {
        if !arr.iter().all(|b| b.is::<bool>()) {
            Err("none only takes bool values".into())
        } else {
            Ok(arr.iter().filter(|b| b.as_bool().unwrap()).count() == n as usize)
        }
    }

    fn script_map_equals(m1: Map, m2: Map) -> ScriptResult<bool> {
        if m1.len() != m2.len() {
            return Ok(false);
        }
        for (key, value) in m1.iter() {
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
            script_array_equals(v1.cast::<Array>(), v2.cast::<Array>())
        } else if t1 == TypeId::of::<Map>() {
            script_map_equals(v1.cast::<Map>(), v2.cast::<Map>())
        } else if t1 == TypeId::of::<Instant>() {
            Ok(v1.cast::<Instant>() == v2.cast::<Instant>())
        } else {
            Err("unsupported type".into())
        }
    }

    fn script_array_equals(arr: Array, arr2: Array) -> ScriptResult<bool> {
        if arr.len() != arr2.len() {
            return Ok(false);
        }
        let result = arr
            .iter()
            .zip(arr2.iter())
            .all(|(e1, e2)| script_value_equals(e1.clone(), e2.clone()).unwrap_or_default());
        Ok(result)
    }

    fn script_array_contains(arr: Array, v: Dynamic) -> ScriptResult<bool> {
        Ok(arr
            .into_iter()
            .any(|ele| script_value_equals(ele, v.clone()).unwrap_or_default()))
    }

    let mut engine = Engine::new();

    let package = CorePackage::new();

    // Register the package into the 'Engine' by converting it into a shared module.
    engine.register_global_module(package.as_shared_module());

    engine
        .register_iterator::<Vec<SsdcFile>>()
        .register_iterator::<Vec<Import>>()
        .register_iterator::<Vec<Namespace>>()
        .register_iterator::<Namespace>()
        .register_iterator::<Vec<DataType>>()
        .register_iterator::<Vec<Service>>()
        .register_iterator::<Vec<Attribute>>()
        .register_iterator::<Vec<NameTypePair>>()
        .register_iterator::<Vec<Dependency>>()
        .register_iterator::<Vec<Parameter>>()
        .register_iterator::<Vec<Handler>>();

    engine.register_fn("to_string", |this: &mut Import| this.path.clone());
    engine.register_fn("NL", |count: i64| "\n".repeat(count as usize));
    engine.register_fn("IND", move |count: i64| indent.repeat(count as usize));

    engine
        .register_type::<SsdcFile>()
        .register_get("name", SsdcFile::namespace)
        .register_get("imports", SsdcFile::imports)
        .register_get("data_types", SsdcFile::data_types)
        .register_get("services", SsdcFile::services);

    engine
        .register_type::<Import>()
        .register_get("path", Import::path)
        .register_get("attributes", Import::attributes);

    engine
        .register_type::<DataType>()
        .register_get("name", DataType::name)
        .register_get("properties", DataType::properties)
        .register_get("attributes", DataType::attributes);

    engine
        .register_type::<Service>()
        .register_get("name", Service::name)
        .register_get("dependencies", Service::dependencies)
        .register_get("handlers", Service::handlers)
        .register_get("attributes", Service::attributes);

    engine
        .register_type::<Dependency>()
        .register_get("name", Dependency::name)
        .register_get("attributes", Dependency::attributes);

    engine
        .register_type::<Handler>()
        .register_get("name", Handler::name)
        .register_get("arguments", Handler::arguments)
        .register_get("return_type", Handler::return_type)
        .register_get("attributes", Handler::attributes);

    engine
        .register_type::<NameTypePair>()
        .register_get("name", NameTypePair::name)
        .register_get("typ", NameTypePair::typ)
        .register_get("attributes", NameTypePair::attributes);

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

    engine
        .register_fn("is_some", script_is_some)
        .register_fn("unwrap", script_unwrap)
        .register_fn("unwrap_or", script_unwrap_or)
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
        .register_result_fn("find_paths", script_find_paths)
        .register_result_fn("read_file", script_read_file);

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
        .register_result_fn("equals", script_map_equals)
        .register_result_fn("equals", script_value_equals)
        .register_result_fn("equals", script_array_equals)
        .register_result_fn("contains", script_array_contains)
        .register_result_fn("require", script_require)
        .register_result_fn("any", script_any)
        .register_result_fn("all", script_all)
        .register_result_fn("none", script_none);

    {
        let messages = messages.clone();
        engine.register_fn("-", move |msg: &str| {
            messages.borrow_mut().push(msg.to_owned())
        });
    }
    {
        let messages = messages.clone();
        engine.register_fn("++", move |a: &str, b: &str| {
            messages.borrow_mut().push(a.to_owned());
            messages.borrow_mut().push(b.to_owned());
            ()
        });
    }
    {
        let messages = messages.clone();
        engine.register_fn("++", move |_: (), b: &str| {
            messages.borrow_mut().push(b.to_owned());
            ()
        });
    }
    engine.register_custom_operator("++", 15).unwrap();
    {
        let messages = messages.clone();
        engine.register_fn("emit", move |msg: &str| {
            messages.borrow_mut().push(msg.to_owned())
        });
    }
    engine.register_custom_operator("then_emit", 15).unwrap();
    {
        let messages = messages.clone();
        engine.register_fn("then_emit", move |a: bool, msg: &str| {
            if a {
                messages.borrow_mut().push(msg.to_owned())
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
                messages.borrow_mut().push(msg.to_owned())
            }
            a
        });
    }
    engine.register_custom_operator("or_emit", 15).unwrap();
    {
        let messages = messages.clone();
        engine.register_fn("or_emit", move |a: bool, msg: &str| {
            if !a {
                messages.borrow_mut().push(msg.to_owned())
            }
            a
        });
    }
    {
        let messages = messages.clone();
        engine.register_fn("or_emit", move |a: bool, m: Map| {
            if !a {
                let msg = m
                    .get("msg")
                    .map(|e| e.clone().into_string().unwrap())
                    .unwrap();
                messages.borrow_mut().push(msg.to_owned())
            }
            a
        });
    }
    // END DSL

    if debug {
        engine.on_print(move |x| eprintln!("INFO => {}", x));
        engine.on_debug(move |x, _, pos| eprintln!("DEBUG({:?}) => {}", pos, x));
    } else {
        engine.on_print(|_| ());
        engine.on_debug(|_, _, _| ());
    }

    engine.disable_symbol("eval");

    engine
}

fn parse_file(base: PathBuf, file: PathBuf) -> anyhow::Result<SsdcFile> {
    parser::parse_file(base, file)
}

fn execute<S: Fn(SsdcFile)>(ns: anyhow::Result<SsdcFile>, s: S) {
    match ns {
        Ok(ns) => s(ns),
        Err(e) => eprintln!("{}", e),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Options { command } = Options::from_args();

    let base = std::fs::canonicalize(
        shellexpand::full(std::env::current_dir()?.to_str().unwrap())?.to_string(),
    )?;
    match command {
        Command::Debug(data) => {
            let path =
                std::fs::canonicalize(shellexpand::full(data.file.to_str().unwrap())?.to_string())?;
            execute(parse_file(base, path), |ns| println!("{:#?}", ns))
        }
        Command::Pretty(data) => execute(parse_file(base, data.file), |ns| {
            println!("{}", ns.to_string())
        }),
        Command::Generate(options) => {
            let model = parse_file(base, options.base.file)?;
            let messages = Rc::new(RefCell::new(Vec::new()));

            let indent = "    ";

            let engine = build_engine(messages.clone(), indent.to_owned(), options.debug);

            let mut scope = Scope::new();
            scope.push("model", model);
            scope.push_constant("NL", "\n");
            scope.push_constant("IND", indent);
            engine.run_file_with_scope(&mut scope, options.script)?;
            let messages = messages.borrow();
            if !messages.is_empty() {
                let result = messages.join("");
                if let Some(out) = options.out {
                    std::fs::write(out, result)?;
                } else {
                    println!("{}", result);
                }
            }
        }
    };

    Ok(())
}
