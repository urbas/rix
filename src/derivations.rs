use crate::parsers::derivations::parse_derivation;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;

#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Derivation {
    pub args: Vec<String>,
    pub builder: String,
    pub env: HashMap<String, String>,
    pub input_drvs: HashMap<String, HashSet<String>>,
    pub input_srcs: HashSet<String>,
    pub outputs: HashMap<String, DerivationOutput>,
    pub platform: String,
}

#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DerivationOutput {
    pub hash: String,
    pub hash_algo: String,
    pub path: String,
}

pub fn load_derivation(drv_path: &str) -> Result<Derivation, String> {
    let content = fs::read_to_string(drv_path)
        .map_err(|err| format!("Failed to read '{}': {}", drv_path, err))?;
    parse_derivation(&content)
        .map(|(_, derivation)| derivation)
        .map_err(|err| format!("Failed to parse '{}': {}", drv_path, err))
}
