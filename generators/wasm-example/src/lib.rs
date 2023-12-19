use extism_pdk::*;
use ssd_data::ast::SsdFile;

#[plugin_fn]
pub fn generate(Json(file): Json<SsdFile>) -> FnResult<String> {
    Ok(format!("{:#?}", file))
}