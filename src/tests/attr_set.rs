use std::collections::HashMap;

use crate::{
    eval::{
        error::NixErrorKind,
        types::{NixTypeKind, Value},
    },
    tests::{eval_err, eval_ok},
};

#[test]
fn eval_attrset_literal() {
    assert_eq!(eval_ok("{}"), Value::AttrSet(HashMap::new()));
    assert_eq!(
        eval_ok("{a = 1;}"),
        Value::AttrSet(HashMap::from([("a".to_owned(), Value::Int(1))]))
    );
}

#[test]
fn eval_attrset_literal_nesting() {
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
fn eval_attrset_interpolated_attrs() {
    assert_eq!(eval_ok(r#"{${"a"} = 1;}.a"#), Value::Int(1));
    assert_eq!(eval_ok(r#"{${"a"}.b = 1;}.a.b"#), Value::Int(1));
    assert_eq!(eval_ok(r#"{a.${"b"} = 1;}.a.b"#), Value::Int(1));
}

#[test]
fn eval_attrset_null_attr() {
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

#[test]
fn eval_recursive_attrset() {
    assert_eq!(eval_ok("rec { a = 1; b = a + 1; }.b"), Value::Int(2));
    assert_eq!(eval_ok(r#"rec { a = "b"; ${a} = 1; }.b"#), Value::Int(1));
}

#[test]
fn eval_attrset_non_string_attr() {
    assert_eq!(
        eval_err(r#"{ ${1} = true; }"#),
        NixErrorKind::TypeMismatch {
            expected: vec![NixTypeKind::String, NixTypeKind::Path],
            got: NixTypeKind::Int
        }
    );
}

#[test]
fn eval_attrset_update() {
    assert_eq!(eval_ok("{} // {}"), Value::AttrSet(HashMap::new()));
    assert_eq!(
        eval_ok("{a = 1; b = 2;} // {a = 3; c = 1;"),
        Value::AttrSet(HashMap::from([
            ("a".to_owned(), Value::Int(3)),
            ("b".to_owned(), Value::Int(2)),
            ("c".to_owned(), Value::Int(1)),
        ]))
    );

    // TODO: Improve the two errors below?
    assert_eq!(
        eval_err("{} // 1"),
        NixErrorKind::TypeMismatch {
            expected: vec![],
            got: NixTypeKind::Set
        }
    );
    assert_eq!(
        eval_err("1 // {}"),
        NixErrorKind::TypeMismatch {
            expected: vec![],
            got: NixTypeKind::Int
        }
    );
}

#[test]
fn eval_attrset_has() {
    assert_eq!(eval_ok("{a = 1;} ? a"), Value::Bool(true));
    assert_eq!(eval_ok("{a = 1;} ? \"a\""), Value::Bool(true));
    assert_eq!(eval_ok("{a = {b = 1;};} ? a.c"), Value::Bool(false));
}

#[test]
fn eval_attrset_select() {
    assert_eq!(eval_ok("{a = 1;}.a"), Value::Int(1));
    assert_eq!(eval_ok("{a = 1;}.b or 2"), Value::Int(2));
}
