use serde::Serialize;
use std::collections::{HashMap, HashSet};

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
