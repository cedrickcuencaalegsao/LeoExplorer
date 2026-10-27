use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FsAttribute {
    Hidden,
    ReadOnly,
    System,
    Archive,
    Indexed,
    Compressed,
    Encrypted,
    Immutable,
    AppendOnly,
}