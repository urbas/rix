use crate::derivations::{Derivation, DerivationOutput};
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::char;
use nom::combinator::{map, opt, value, verify};
use nom::multi::{fold_many0, separated_list0};
use nom::sequence::{delimited, pair, preceded, tuple};
use nom::IResult;
use std::collections::{HashMap, HashSet};

pub fn parse_derivation(input: &str) -> IResult<&str, Derivation> {
    delimited(tag("Derive("), parse_derivation_args, char(')'))(input)
}

fn parse_derivation_args(input: &str) -> IResult<&str, Derivation> {
    let (input, (outputs, _, input_drvs, _, input_srcs, _, system, _, builder, _, args, _, env)) =
        tuple((
            parse_derivation_outputs,
            char(','),
            parse_input_derivations,
            char(','),
            parse_string_set,
            char(','),
            parse_string,
            char(','),
            parse_string,
            char(','),
            parse_strings,
            char(','),
            parse_env,
        ))(input)?;
    Ok((
        input,
        Derivation {
            args,
            builder,
            env,
            input_drvs,
            input_srcs,
            outputs,
            system,
        },
    ))
}

fn parse_derivation_outputs(input: &str) -> IResult<&str, HashMap<String, DerivationOutput>> {
    let derivation_outputs = fold_many0(
        pair(parse_derivation_output, opt(char(','))),
        HashMap::new,
        |mut drv_outputs, ((name, drv_output), _)| {
            drv_outputs.insert(name, drv_output);
            drv_outputs
        },
    );
    delimited(char('['), derivation_outputs, char(']'))(input)
}

fn parse_derivation_output(input: &str) -> IResult<&str, (String, DerivationOutput)> {
    let (input, (_, derivation_name, _, path, _, hash_algo, _, hash, _)) = tuple((
        char('('),
        parse_string,
        char(','),
        parse_string,
        char(','),
        parse_string,
        char(','),
        parse_string,
        char(')'),
    ))(input)?;
    Ok((
        input,
        (
            derivation_name,
            DerivationOutput {
                hash: if hash.is_empty() { None } else { Some(hash) },
                hash_algo: if hash_algo.is_empty() {
                    None
                } else {
                    Some(hash_algo)
                },
                path: path,
            },
        ),
    ))
}

fn parse_string(input: &str) -> IResult<&str, String> {
    delimited(char('"'), parse_string_inside_quotes, char('"'))(input)
}

fn parse_input_derivations(input: &str) -> IResult<&str, HashMap<String, HashSet<String>>> {
    let input_derivations = fold_many0(
        tuple((
            char('('),
            parse_string,
            char(','),
            parse_string_set,
            char(')'),
            opt(char(',')),
        )),
        HashMap::new,
        |mut input_drvs, (_, drv, _, input_type, _, _)| {
            input_drvs.insert(drv, input_type);
            input_drvs
        },
    );
    delimited(char('['), input_derivations, char(']'))(input)
}

fn parse_string_set(input: &str) -> IResult<&str, HashSet<String>> {
    let string_set = fold_many0(
        pair(parse_string, opt(char(','))),
        HashSet::new,
        |mut strings, (string, _)| {
            strings.insert(string);
            strings
        },
    );
    delimited(char('['), string_set, char(']'))(input)
}

fn parse_strings(input: &str) -> IResult<&str, Vec<String>> {
    delimited(
        char('['),
        separated_list0(char(','), parse_string),
        char(']'),
    )(input)
}

fn parse_env(input: &str) -> IResult<&str, HashMap<String, String>> {
    let env_vars = fold_many0(
        tuple((
            char('('),
            parse_string,
            char(','),
            parse_string,
            char(')'),
            opt(char(',')),
        )),
        HashMap::new,
        |mut env_vars, (_, name, _, value, _, _)| {
            env_vars.insert(name, value);
            env_vars
        },
    );
    delimited(char('['), env_vars, char(']'))(input)
}

enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
}

fn parse_string_inside_quotes(input: &str) -> IResult<&str, String> {
    fold_many0(
        parse_string_fragment,
        String::new,
        |mut string, fragment| {
            match fragment {
                StringFragment::Literal(str_literal) => string.push_str(str_literal),
                StringFragment::EscapedChar(escaped_char) => string.push(escaped_char),
            }
            string
        },
    )(input)
}

fn parse_string_fragment(input: &str) -> IResult<&str, StringFragment> {
    alt((parse_literal, parse_escaped_char))(input)
}

fn parse_literal(input: &str) -> IResult<&str, StringFragment> {
    let non_empty_literal = verify(is_not("\"\\"), |matched_str: &str| !matched_str.is_empty());
    map(non_empty_literal, StringFragment::Literal)(input)
}

fn parse_escaped_char(input: &str) -> IResult<&str, StringFragment> {
    let escaped_char = preceded(
        char('\\'),
        alt((
            char('"'),
            char('\\'),
            value('\n', char('n')),
            value('\r', char('r')),
            value('\t', char('t')),
        )),
    );
    map(escaped_char, StringFragment::EscapedChar)(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_derivation() {
        let expected_derivation = Derivation {
            args: to_string_vec(&["-e", "/builder.sh"]),
            builder: "/bash".to_owned(),
            env: vec![
                ("ENV1".to_owned(), "val1".to_owned()),
                ("ENV2".to_owned(), "val2".to_owned()),
            ]
            .into_iter()
            .collect(),
            input_drvs: vec![
                ("/drv1".to_owned(), to_string_set(&["out"])),
                ("/drv2".to_owned(), to_string_set(&["dev"])),
            ]
            .into_iter()
            .collect(),
            input_srcs: to_string_set(&["/builder.sh"]),
            outputs: vec![("out".to_owned(), to_drv_out("sha256", "abc", "/foo"))]
                .into_iter()
                .collect(),
            system: "x86_64-linux".to_owned(),
        };
        assert_eq!(
            parse_derivation(
                r#"Derive([("out","/foo","sha256","abc")],[("/drv1",["out"]),("/drv2",["dev"])],["/builder.sh"],"x86_64-linux","/bash",["-e","/builder.sh"],[("ENV1","val1"),("ENV2","val2")])"#
            ),
            Ok(("", expected_derivation,)),
        );
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse_string(r#""ab""#), Ok(("", "ab".to_owned())));
        assert_eq!(parse_string(r#""\"""#), Ok(("", "\"".to_owned())));
        assert_eq!(parse_string(r#""\\""#), Ok(("", "\\".to_owned())));
        assert_eq!(parse_string(r#""\n""#), Ok(("", "\n".to_owned())));
        assert_eq!(parse_string(r#""\r""#), Ok(("", "\r".to_owned())));
        assert_eq!(parse_string(r#""\t""#), Ok(("", "\t".to_owned())));

        assert_eq!(
            parse_string(r#""Foo\tbar\n\rmoo\\zar\"""#),
            Ok(("", "Foo\tbar\n\rmoo\\zar\"".to_owned()))
        );
    }

    #[test]
    fn test_parse_string_invalid() {
        assert_eq!(
            parse_string("").unwrap_err(),
            nom::Err::Error(nom::error::Error::new("", nom::error::ErrorKind::Char)),
            "Parsing an empty input as a string literal must fail",
        );
        assert_eq!(
            parse_string("a").unwrap_err(),
            nom::Err::Error(nom::error::Error::new("a", nom::error::ErrorKind::Char)),
            "Parsing a string literal that doesn't start with a double-quote must fail",
        );
        assert_eq!(
            parse_string("\"").unwrap_err(),
            nom::Err::Error(nom::error::Error::new("", nom::error::ErrorKind::Char)),
            "Parsing an unclosed string literal should fail",
        );
    }

    #[test]
    fn test_parse_derivation_output() {
        assert_eq!(
            parse_derivation_output(r#"("foo","store_path","sha256","hash")"#),
            Ok((
                "",
                ("foo".to_owned(), to_drv_out("sha256", "hash", "store_path")),
            )),
        );
    }

    #[test]
    fn test_parse_derivation_outputs() {
        let actual = parse_derivation_outputs(r#"[("a","b","c","d"),("e","f","g","h")]"#);
        let expected = vec![
            ("a".to_owned(), to_drv_out("c", "d", "b")),
            ("e".to_owned(), to_drv_out("g", "h", "f")),
        ];
        assert_eq!(actual, Ok(("", expected.into_iter().collect())));
    }

    #[test]
    fn test_parse_input_derivations() {
        let actual = parse_input_derivations(r#"[("a",["b","c"]),("e",["f","g"])]"#);
        let expected = vec![
            ("a".to_owned(), to_string_set(&["b", "c"])),
            ("e".to_owned(), to_string_set(&["f", "g"])),
        ];
        assert_eq!(actual, Ok(("", expected.into_iter().collect())));
    }

    #[test]
    fn test_parse_string_set() {
        let actual = parse_string_set(r#"["a","b","b"]"#);
        let expected = to_string_set(&["a", "b"]);
        assert_eq!(actual, Ok(("", expected)));
    }

    #[test]
    fn test_parse_strings() {
        let actual = parse_strings(r#"["a","b","a"]"#);
        let expected = to_string_vec(&["a", "b", "a"]);
        assert_eq!(actual, Ok(("", expected)));
    }

    #[test]
    fn test_parse_env() {
        let actual = parse_env(r#"[("A","a"),("B","b")]"#);
        let expected = vec![
            ("A".to_owned(), "a".to_owned()),
            ("B".to_owned(), "b".to_owned()),
        ];
        assert_eq!(actual, Ok(("", expected.into_iter().collect())));
    }

    fn to_drv_out(hash_algo: &str, hash: &str, path: &str) -> DerivationOutput {
        DerivationOutput {
            hash: Some(hash.to_owned()),
            hash_algo: Some(hash_algo.to_owned()),
            path: path.to_owned(),
        }
    }

    fn to_string_vec(strings: &[&str]) -> Vec<String> {
        strings.iter().cloned().map(String::from).collect()
    }

    fn to_string_set(strings: &[&str]) -> HashSet<String> {
        strings.iter().cloned().map(String::from).collect()
    }
}
