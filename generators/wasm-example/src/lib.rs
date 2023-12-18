use extism_pdk::*;
use ssd_data::ast::SsdcFile;

#[plugin_fn]
pub fn generate(Json(file): Json<SsdcFile>) -> FnResult<String> {
    Ok(format!("{:#?}", file))
}