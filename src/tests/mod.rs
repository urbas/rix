#![cfg(test)]
#![allow(clippy::expect_fun_call)]
#![allow(clippy::approx_constant)]

use std::collections::HashMap;

use crate::eval::{error::NixErrorKind, execution::evaluate, types::Value};

mod attr_set;
mod builtins;
mod literals;
mod operators;
mod lambda;

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
fn eval_if_then_else() {
    assert_eq!(eval_ok("if true then 1 else 0"), Value::Int(1));
    assert_eq!(eval_ok("if false then 1 else 0"), Value::Int(0));
}

#[test]
fn eval_if_then_else_invalid_type() {
    assert!(evaluate("if 0 then 1 else 0").is_err());
}

#[test]
fn eval_let_in() {
    assert_eq!(eval_ok("let a = 1; in a"), Value::Int(1));
}

#[test]
fn eval_with() {
    assert_eq!(eval_ok("with {a = 1;}; a"), Value::Int(1));
    assert_eq!(eval_ok("let a = 2; in with {a = 1;}; a"), Value::Int(2));
    assert_eq!(eval_ok("with {a = 1;}; with {a = 2;}; a"), Value::Int(2));
}

#[test]
fn eval_recursive_let() {
    assert_eq!(eval_ok("let a = 1; b = a + 1; in b"), Value::Int(2));
}
