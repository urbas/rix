#![cfg(test)]
#![allow(clippy::expect_fun_call)]
#![allow(clippy::approx_constant)]

use std::collections::HashMap;

use crate::eval::{error::NixErrorKind, execution::evaluate, types::Value};

mod attr_set;
mod builtins;
mod literals;
mod operators;

fn eval_ok(nix_expr: &str) -> Value {
    match evaluate(nix_expr) {
        Ok(val) => val,
        Err(err) => panic!(
            "eval '{}' shouldn't fail.\nError message: {}",
            nix_expr, err
        ),
    }
}

fn eval_err(nix_expr: &str) -> NixErrorKind {
    evaluate(nix_expr)
        .expect_err(&format!("eval '{}' expected an error", nix_expr))
        .kind
}

#[test]
fn test_eval_attrset_non_string_attr() {
    assert!(evaluate(r#"{ ${1} = true; }"#).is_err());
}

#[test]
fn test_eval_attrset_update() {
    assert_eq!(eval_ok("{} // {}"), Value::AttrSet(HashMap::new()));
    assert_eq!(
        eval_ok("{a = 1; b = 2;} // {a = 3; c = 1;"),
        Value::AttrSet(HashMap::from([
            ("a".to_owned(), Value::Int(3)),
            ("b".to_owned(), Value::Int(2)),
            ("c".to_owned(), Value::Int(1)),
        ]))
    );
    assert!(evaluate("{} // 1").is_err());
    assert!(evaluate("1 // {}").is_err());
}

#[test]
fn test_eval_attrset_has() {
    assert_eq!(eval_ok("{a = 1;} ? a"), Value::Bool(true));
    assert_eq!(eval_ok("{a = 1;} ? \"a\""), Value::Bool(true));
    assert_eq!(eval_ok("{a = {b = 1;};} ? a.c"), Value::Bool(false));
}

#[test]
fn test_eval_attrset_select() {
    assert_eq!(eval_ok("{a = 1;}.a"), Value::Int(1));
    assert_eq!(eval_ok("{a = 1;}.b or 2"), Value::Int(2));
}

#[test]
fn test_eval_lambda() {
    assert_eq!(eval_ok("a: 1"), Value::Lambda);
}

#[test]
fn test_eval_lambda_application() {
    assert_eq!(eval_ok("(a: 1) 2"), Value::Int(1));
    assert_eq!(eval_ok("(a: a + 1) 2"), Value::Int(3));
}

#[test]
fn test_eval_pattern_lambda() {
    assert_eq!(eval_ok("({a, b}: a + b) {a = 1; b = 2;}"), Value::Int(3));
    assert_eq!(eval_ok("({a, b ? 2}: a + b) {a = 1;}"), Value::Int(3));
    assert!(evaluate("{a, a}: a").is_err());
}

#[test]
fn test_eval_pattern_lambda_args_binding() {
    assert_eq!(eval_ok("({a}@args: args.a) {a = 1;}"), Value::Int(1));
    assert!(evaluate("{a}@a: a").is_err());
    assert!(evaluate("({a ? 1}@args: args.a) {}").is_err());
}

#[test]
fn test_eval_path() {
    assert_eq!(eval_ok("/."), Value::Path("/".to_owned()));
    assert_eq!(eval_ok("/a"), Value::Path("/a".to_owned()));
    assert_eq!(
        eval_ok("./a"),
        Value::Path(format!("{}/a", std::env::current_dir().unwrap().display()))
    );
    assert_eq!(
        eval_ok("./a/../b"),
        Value::Path(format!("{}/b", std::env::current_dir().unwrap().display()))
    );
}

#[test]
fn test_eval_path_concat() {
    assert_eq!(eval_ok(r#"/. + "a""#), Value::Path("/a".to_owned()));
    assert_eq!(eval_ok(r#"/. + "./a/../b""#), Value::Path("/b".to_owned()));
}

#[test]
fn test_eval_if_then_else() {
    assert_eq!(eval_ok("if true then 1 else 0"), Value::Int(1));
    assert_eq!(eval_ok("if false then 1 else 0"), Value::Int(0));
}

#[test]
fn test_eval_if_then_else_invalid_type() {
    assert!(evaluate("if 0 then 1 else 0").is_err());
}

#[test]
fn test_eval_let_in() {
    assert_eq!(eval_ok("let a = 1; in a"), Value::Int(1));
}

#[test]
fn test_eval_with() {
    assert_eq!(eval_ok("with {a = 1;}; a"), Value::Int(1));
    assert_eq!(eval_ok("let a = 2; in with {a = 1;}; a"), Value::Int(2));
    assert_eq!(eval_ok("with {a = 1;}; with {a = 2;}; a"), Value::Int(2));
}

#[test]
fn test_eval_recursive_let() {
    assert_eq!(eval_ok("let a = 1; b = a + 1; in b"), Value::Int(2));
}

#[test]
fn test_eval_recursive_attrset() {
    assert_eq!(eval_ok("rec { a = 1; b = a + 1; }.b"), Value::Int(2));
    assert_eq!(eval_ok(r#"rec { a = "b"; ${a} = 1; }.b"#), Value::Int(1));
}

#[test]
fn test_eval_builtin_abort() {
    let error_msg = evaluate(r#"abort "foo""#).unwrap_err();
    let expected_msg = "foo";
    assert_eq!(
        error_msg.kind,
        NixErrorKind::Abort {
            message: expected_msg.to_owned()
        }
    );
}
