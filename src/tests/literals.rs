use std::collections::HashMap;

use crate::{
    eval::{
        error::NixErrorKind,
        types::{NixTypeKind, Value},
    },
    tests::{eval_err, eval_ok},
};

#[test]
fn test_eval_int_literals() {
    assert_eq!(eval_ok("1"), Value::Int(1));
    assert_eq!(eval_ok("-1"), Value::Int(-1));
    assert_eq!(eval_ok("0"), Value::Int(0));
    assert_eq!(eval_ok("1234567890"), Value::Int(1234567890));

    // Test i64 limits
    assert_eq!(
        eval_ok("9223372036854775807"),
        Value::Int(9223372036854775807)
    );
    assert_eq!(
        eval_ok("-9223372036854775808"),
        Value::Int(-9223372036854775808)
    );
}

#[test]
fn test_eval_float_literals() {
    assert_eq!(eval_ok("1.0"), Value::Float(1.0));
    assert_eq!(eval_ok("-1.0"), Value::Float(-1.0));
    assert_eq!(eval_ok("0.0"), Value::Float(0.0));
    assert_eq!(eval_ok("3.14"), Value::Float(3.14));
    assert_eq!(eval_ok("-3.14"), Value::Float(-3.14));
}

#[test]
fn test_eval_complex_float_literals() {
    // Turns out nix doesn't support scientific notation without a decimal point
    // assert_eq!(eval_ok("1e10"), Value::Float(1e10));
    // assert_eq!(eval_ok("-1e10"), Value::Float(-1e10));

    assert_eq!(eval_ok("2.5e-3"), Value::Float(2.5e-3));
    assert_eq!(eval_ok("-3.14e2"), Value::Float(-3.14e2));
    assert_eq!(eval_ok(".25e-3"), Value::Float(0.25e-3));
    assert_eq!(eval_ok("-.314e2"), Value::Float(-0.314e2));
}

#[test]
fn test_eval_bool_literals() {
    assert_eq!(eval_ok("true"), Value::Bool(true));
    assert_eq!(eval_ok("false"), Value::Bool(false));
}

#[test]
fn test_eval_string_literal() {
    assert_eq!(eval_ok(r#""Hello!""#), Value::Str("Hello!".to_owned()));
}

#[test]
fn test_eval_string_literal_escape_codes() {
    assert_eq!(
        eval_ok(r#""\"\$\n\r\t\\`""#),
        Value::Str("\"$\n\r\t\\`".to_owned())
    );
    assert_eq!(eval_ok("\"a \n b\""), Value::Str("a \n b".to_owned()));
}

#[test]
fn test_eval_string_uri() {
    assert_eq!(
        eval_ok("http://foo.bat/moo"),
        Value::Str("http://foo.bat/moo".to_owned())
    );
}

#[test]
fn test_eval_indented_string() {
    assert_eq!(
        eval_ok(
            "''
  Hello
  World!''"
        ),
        Value::Str("Hello\nWorld!".to_owned())
    );
    assert_eq!(
        eval_ok(
            "''
  a
 b
   c''"
        ),
        Value::Str(" a\nb\n  c".to_owned())
    );
    assert_eq!(
        eval_ok("''''$'''$${}''\\n''\\t''\\r''\\\\''"),
        Value::Str("$''$${}\n\t\r\\".to_owned())
    );
}

#[test]
fn test_eval_string_interpolation() {
    let path = std::env::current_dir().unwrap();

    assert_eq!(eval_ok(r#""${"A"}""#), Value::Str("A".to_owned()));
    assert_eq!(
        eval_ok(r#""${./foo}""#),
        Value::Str(format!("{}/foo", path.display()))
    );
    assert_eq!(
        eval_err(r#""${1}""#),
        NixErrorKind::TypeMismatch {
            expected: vec![NixTypeKind::String, NixTypeKind::Path],
            got: NixTypeKind::Int
        }
    );
}

#[test]
fn test_eval_list_literal() {
    assert_eq!(eval_ok("[]"), Value::List(vec![]));
    assert_eq!(
        eval_ok(r#"[42 true "answer"]"#),
        Value::List(vec![
            Value::Int(42),
            Value::Bool(true),
            Value::Str("answer".to_owned())
        ])
    );
    assert_eq!(
        eval_ok(
            r#"[
                42
                true
                "answer"
            ]"#
        ),
        Value::List(vec![
            Value::Int(42),
            Value::Bool(true),
            Value::Str("answer".to_owned())
        ])
    );
}
