use serde::{Deserialize, Serialize};
use ssd_data::{DataType, Enum, Import, Service};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AstElement {
    Import(Import),
    DataType((String, DataType)),
    Enum((String, Enum)),
    Service((String, Service)),
}
