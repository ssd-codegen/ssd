#[cfg(feature = "_web")]
mod generators;
#[cfg(feature = "_web")]
pub use generators::rhai::generate_web;

mod ast;
mod helper;
mod parser;
pub use parser::{parse, parse_file, parse_file_with_namespace};
pub use helper::{update_types_from_file, print_or_write, parse_raw_data};
#[cfg(not(feature = "_bin"))]
pub use helper::update_types;

#[cfg(feature = "_python")]
mod python {
    use std::path::Path;

    use pyo3::exceptions::PyException;
    use pyo3::prelude::*;
    use pyo3::Python;

    use ssd_data::Namespace;
    use ssd_data::SsdModule;

    #[pyfunction]
    pub fn parse(content: &str, namespace: &str) -> PyResult<SsdModule> {
        crate::parse(content, Namespace::new(namespace))
            .map_err(|e| PyException::new_err(e.to_string()))
    }

    #[pyfunction]
    pub fn parse_file(base: &str, path: &str) -> PyResult<SsdModule> {
        crate::parse_file(&Path::new(base), &Path::new(path))
            .map_err(|e| PyException::new_err(e.to_string()))
    }

    #[pyfunction]
    pub fn parse_file_with_namespace(path: &str, namespace: &str) -> PyResult<SsdModule> {
        crate::parse_file_with_namespace(Path::new(path), Namespace::new(namespace))
            .map_err(|e| PyException::new_err(e.to_string()))
    }

    #[pymodule]
    fn py_ssd(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(parse, m)?)?;
        m.add_function(wrap_pyfunction!(parse_file, m)?)?;
        m.add_function(wrap_pyfunction!(parse_file_with_namespace, m)?)?;
        Ok(())
    }
}
