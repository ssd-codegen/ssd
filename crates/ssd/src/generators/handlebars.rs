use crate::options::{BaseInputData, BaseOutputData};
use clap::Parser;
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

use crate::parser::parse_file;
use ssd_data::{RawModel, SsdModel};

use crate::helper::parse_raw_data;
use crate::helper::{print_or_write, update_types_from_file};

use handlebars::Handlebars;

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
        input,
        out,
        template,
    }: Parameters,
) -> Result<(), Box<dyn Error>> {
    let reg = Handlebars::new();
    let result = if input.raw {
        let raw = parse_raw_data(input.file)?;

        reg.render_template(
            &std::fs::read_to_string(template)?,
            &RawModel { raw, defines },
        )?
    } else {
        let module = parse_file(base, &input.file)?;
        let module = update_types_from_file(module, input.no_map, input.typemap, Some(&template))?;
        reg.render_template(
            &std::fs::read_to_string(template)?,
            &SsdModel { module, defines },
        )?
    };
    print_or_write(out.out, &result)?;

    Ok(())
}
