use extism_pdk::*;
use ssd_data::SsdModel;

#[plugin_fn]
pub fn generate(Json(model): Json<SsdModel>) -> FnResult<String> {
    Ok(format!("{:#?}", model))
}