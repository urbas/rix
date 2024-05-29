#![cfg(test)]
#![allow(clippy::expect_fun_call)]
#![allow(clippy::approx_constant)]

use crate::eval::{
    error::NixErrorKind,
    execution::evaluate,
    types::{NixTypeKind, Value},
};

mod attr_set;
mod builtins;
mod lambda;
mod literals;
mod operators;

fn eval_ok(nix_expr: &str) -> Value {
    let workdir = std::env::current_dir().unwrap();
    match evaluate(nix_expr, &workdir) {
        Ok(val) => val,
        Err(err) => panic!("eval '{nix_expr}' shouldn't fail.\nError message: {err:?}",),
    }
}

fn eval_err(nix_expr: &str) -> NixErrorKind {
    let workdir = std::env::current_dir().unwrap();
    evaluate(nix_expr, &workdir)
        .expect_err(&format!("eval '{nix_expr}' expected an error"))
        .kind
}

#[test]
fn eval_if_then_else() {
    assert_eq!(eval_ok("if true then 1 else 0"), Value::Int(1));
    assert_eq!(eval_ok("if false then 1 else 0"), Value::Int(0));
}

#[test]
fn eval_if_then_else_invalid_type() {
    assert_eq!(
        eval_err("if 0 then 1 else 0"),
        NixErrorKind::TypeMismatch {
            expected: vec![NixTypeKind::Bool],
            got: NixTypeKind::Int,
        }
    );
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
