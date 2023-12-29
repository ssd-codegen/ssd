use crate::options::{BaseInputData, BaseOutputData};
use crate::parse_raw_data;
use clap::Parser;
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

use ssd::parse_file;
use ssd_data::{RawModel, SsdModel};

use crate::{print_or_write, update_types};

use tera::{Context, Tera};

#[derive(Debug, Parser)]
pub struct Parameters {
    /// The template to use to generate the file.
    pub template: PathBuf,
    #[clap(flatten)]
    pub input: BaseInputData,
    #[clap(flatten)]
    pub out: BaseOutputData,
}

pub fn generate(
    base: &PathBuf,
    defines: HashMap<String, String>,
    Parameters {
        template,
        input,
        out,
    }: Parameters,
) -> Result<(), Box<dyn Error>> {
    let mut tera = Tera::default();
    tera.add_template_file(&template, None)?;
    let result = if input.raw {
        let raw = parse_raw_data(input.file)?;
        tera.render(
            &template.to_string_lossy(),
            &Context::from_serialize(RawModel { raw, defines })?,
        )?
    } else {
        let module = parse_file(base, &input.file)?;
        let module = update_types(module, input.no_map, input.typemap, None)?;
        tera.render(
            &template.to_string_lossy(),
            &Context::from_serialize(SsdModel { module, defines })?,
        )?
    };
    print_or_write(out.out, &result)?;

    Ok(())
}