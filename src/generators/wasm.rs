use clap::Parser;
use extism::{convert::Json, Manifest, PluginBuilder, Wasm};
use std::collections::HashMap;
use std::path::PathBuf;

use ssd_data::{RawModel, SsdModel};

use crate::helper::parse_raw_data;
use crate::helper::{print_or_write, update_types_from_file};
use crate::options::{BaseInputData, BaseOutputData};
use crate::parser::parse_file;

#[derive(Debug, Parser)]
pub struct Parameters {
    /// The wasm plugin to use to generate the file.
    pub wasm: PathBuf,
    #[clap(flatten)]
    pub input: BaseInputData,
    #[clap(flatten)]
    pub out: BaseOutputData,
}

pub fn generate(
    base: &PathBuf,
    defines: HashMap<String, String>,
    Parameters { wasm, input, out }: Parameters,
) -> anyhow::Result<()> {
    let file = Wasm::file(&wasm);
    let manifest = Manifest::new([file]);
    let mut plugin = PluginBuilder::new(&manifest).with_wasi(false).build()?;

    let result = if input.raw {
        let raw = parse_raw_data(input.file)?;
        plugin.call::<Json<RawModel>, &str>("generate", Json(RawModel { raw, defines }))?
    } else {
        let module = parse_file(base, &input.file)?;
        let module = update_types_from_file(module, input.no_map, input.typemap, Some(&wasm))?;
        plugin.call::<Json<SsdModel>, &str>("generate", Json(SsdModel { module, defines }))?
    };

    print_or_write(out.out, result)?;

    Ok(())
}
