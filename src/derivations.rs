use crate::parsers::derivations::parse_derivation;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Derivation {
    pub args: Vec<String>,
    pub builder: String,
    pub env: BTreeMap<String, String>,
    pub input_drvs: BTreeMap<String, InputDrv>,
    pub input_srcs: BTreeSet<String>,
    pub outputs: BTreeMap<String, DerivationOutput>,
    pub system: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DerivationOutput {
    pub hash: Option<String>,
    pub hash_algo: Option<String>,
    pub path: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InputDrv {
    pub dynamic_outputs: BTreeMap<String, InputDrv>,
    pub outputs: BTreeSet<String>,
}

pub fn load_derivation(drv_path: &str) -> Result<Derivation, String> {
    let content = fs::read_to_string(drv_path)
        .map_err(|err| format!("Failed to read '{}': {}", drv_path, err))?;
    parse_derivation(&content)
        .map(|(_, derivation)| derivation)
        .map_err(|err| format!("Failed to parse '{}': {}", drv_path, err))
}

pub fn save_derivation(writer: &mut impl Write, derivation: &Derivation) -> std::io::Result<()> {
    write!(writer, "Derive(")?;
    write_outputs(writer, &derivation.outputs)?;
    write!(writer, ",")?;
    write_input_drvs(writer, &derivation.input_drvs)?;
    write!(writer, ",")?;
    write_iter(writer, &mut derivation.input_srcs.iter(), write_string)?;
    write!(writer, ",")?;
    write_string(writer, &derivation.system)?;
    write!(writer, ",")?;
    write_string(writer, &derivation.builder)?;
    write!(writer, ",")?;
    write_iter(writer, &mut derivation.args.iter(), write_string)?;
    write!(writer, ",")?;
    write_iter(
        writer,
        &mut derivation.env.iter(),
        |writer, (key, value)| {
            write!(writer, "(")?;
            write_string(writer, key)?;
            write!(writer, ",")?;
            write_string(writer, value)?;
            write!(writer, ")")
        },
    )?;
    write!(writer, ")")
}

fn write_outputs(
    writer: &mut impl Write,
    outputs: &BTreeMap<String, DerivationOutput>,
) -> std::io::Result<()> {
    write_iter(writer, &mut outputs.iter(), |writer, entry| {
        write_output(writer, entry.0, entry.1)
    })
}

fn write_input_drvs(
    writer: &mut impl Write,
    input_drvs: &BTreeMap<String, InputDrv>,
) -> std::io::Result<()> {
    write_iter(writer, &mut input_drvs.iter(), |writer, entry| {
        let (drv_path, input_drv) = entry;
        write!(writer, "(")?;
        write_string(writer, drv_path)?;
        write!(writer, ",")?;
        write_iter(writer, &mut input_drv.outputs.iter(), write_string)?;
        write!(writer, ")")
    })
}

fn write_iter<W, T, F>(
    writer: &mut W,
    iter: &mut impl Iterator<Item = T>,
    write_value: F,
) -> std::io::Result<()>
where
    W: Write,
    F: Fn(&mut W, T) -> std::io::Result<()>,
{
    write!(writer, "[")?;
    if let Some(entry) = iter.next() {
        write_value(writer, entry)?;
    }
    for entry in iter.by_ref() {
        write!(writer, ",")?;
        write_value(writer, entry)?;
    }
    write!(writer, "]")?;
    Ok(())
}

fn write_output(
    writer: &mut impl Write,
    output_name: &String,
    output: &DerivationOutput,
) -> std::io::Result<()> {
    write!(writer, "(")?;
    write_string(writer, output_name)?;
    write!(writer, ",")?;
    write_string(writer, &output.path)?;
    write!(writer, ",")?;
    write_string(writer, output.hash_algo.as_ref().unwrap_or(&String::new()))?;
    write!(writer, ",")?;
    write_string(writer, output.hash.as_ref().unwrap_or(&String::new()))?;
    write!(writer, ")")
}

fn write_string(writer: &mut impl Write, string: &String) -> std::io::Result<()> {
    let mut escaped_string = String::with_capacity(2 * string.capacity());
    for character in string.chars() {
        match character {
            '\t' => escaped_string.push_str("\\t"),
            '\n' => escaped_string.push_str("\\n"),
            '\r' => escaped_string.push_str("\\r"),
            '\\' => escaped_string.push_str("\\\\"),
            '"' => escaped_string.push_str("\\\""),
            character => escaped_string.push(character),
        }
    }
    write!(writer, "\"{}\"", escaped_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load() {
        let tmp_dir = tempdir().unwrap();
        let derivation = sample_derivation();
        let derivation_path = tmp_dir.path().join("foo.drv");
        let mut derivation_file = File::create(&derivation_path).unwrap();
        save_derivation(&mut derivation_file, &derivation).unwrap();
        let derivation_from_file = load_derivation(&derivation_path.to_str().unwrap()).unwrap();
        assert_eq!(derivation_from_file, derivation);
    }

    #[test]
    fn test_save_and_load_json() {
        let derivation = sample_derivation();
        let derivation_json_str = serde_json::to_string(&derivation).unwrap();
        let derivation_from_json: Derivation = serde_json::from_str(&derivation_json_str).unwrap();
        assert_eq!(derivation, derivation_from_json);
    }

    fn sample_derivation() -> Derivation {
        Derivation {
            args: vec!["foo".to_owned(), "bar".to_owned()],
            builder: "foo.sh".to_owned(),
            env: BTreeMap::from([("var1".to_owned(), "val1".to_owned())]),
            input_drvs: BTreeMap::from([(
                "foo.drv".to_owned(),
                InputDrv {
                    dynamic_outputs: BTreeMap::new(),
                    outputs: BTreeSet::from(["out".to_owned()]),
                },
            )]),
            input_srcs: BTreeSet::from(["/foo.txt".to_owned()]),
            outputs: BTreeMap::from([(
                "out".to_owned(),
                DerivationOutput {
                    hash: None,
                    hash_algo: Some("foo".to_owned()),
                    path: "/foo.out".to_owned(),
                },
            )]),
            system: "foo-x64".to_owned(),
        }
    }
}
