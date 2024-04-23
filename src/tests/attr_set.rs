use std::collections::HashMap;

use crate::{eval::{error::NixErrorKind, types::{NixTypeKind, Value}}, tests::{eval_err, eval_ok}};

#[test]
fn test_eval_attrset_literal() {
    assert_eq!(eval_ok("{}"), Value::AttrSet(HashMap::new()));
    assert_eq!(
        eval_ok("{a = 1;}"),
        Value::AttrSet(HashMap::from([("a".to_owned(), Value::Int(1))]))
    );
}

#[test]
fn test_eval_attrset_literal_nesting() {
    let expected_attrset = Value::AttrSet(HashMap::from([(
        "a".to_owned(),
        Value::AttrSet(HashMap::from([("b".to_owned(), Value::Int(1))])),
    )]));
    assert_eq!(eval_ok("{a.b = 1;}"), expected_attrset);
    assert_eq!(eval_ok("{ a = {}; a.b = 1; }"), expected_attrset);
    // TODO: Improve this error? Instead of a basic type mismatch
    assert_eq!(
        eval_err("{ a = 1; a.b = 1; }"),
        NixErrorKind::TypeMismatch {
            expected: vec![NixTypeKind::Set],
            got: NixTypeKind::Int
        }
    );
    // TODO: Replicate behaviour: nix currently throws an error while evaluating the following:
    // { a = builtins.trace "Evaluated" {}; a.b = 1; }
    // error: attribute 'a.b' already defined at
    // In similar vein, `nix` throws evaluating this expression:
    // let c = {}; in { a = c; a.b = 1; }
    // We should reproduce this here.
}

#[test]
fn test_eval_attrset_interpolated_attrs() {
    assert_eq!(eval_ok(r#"{${"a"} = 1;}.a"#), Value::Int(1));
    assert_eq!(eval_ok(r#"{${"a"}.b = 1;}.a.b"#), Value::Int(1));
    assert_eq!(eval_ok(r#"{a.${"b"} = 1;}.a.b"#), Value::Int(1));
}

#[test]
fn test_eval_attrset_null_attr() {
    assert_eq!(
        eval_ok(r#"{ ${null} = true; }"#),
        Value::AttrSet(HashMap::new()),
    );
    assert_eq!(
        eval_ok(r#"{ a.${null} = true; }"#),
        Value::AttrSet(HashMap::from([(
            "a".to_owned(),
            Value::AttrSet(HashMap::new()),
        )])),
    );
}
