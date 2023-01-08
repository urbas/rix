use std::sync::Once;

use rnix::{
    ast::{BinOp, BinOpKind, Expr, Ident, Literal, UnaryOp, UnaryOpKind},
    SyntaxKind, SyntaxToken,
};
use rowan::ast::AstNode;

use crate::eval::types::EvalResult;
use crate::eval::types::Value;

static INIT_V8: Once = Once::new();

pub fn evaluate(nix_expr: &str) -> EvalResult {
    initialize_v8();
    let isolate = &mut v8::Isolate::new(Default::default());
    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);
    let js_expr = emit_top_level(nix_expr)?;
    let code = v8::String::new(scope, &js_expr).unwrap();
    // TODO: Make this faster! Maybe we can use v8's compiled code caching to speed things up a notch?
    let script = v8::Script::compile(scope, code, None).unwrap();
    let result = script.run(scope).unwrap();
    js_value_to_nix(scope, &result)
}

fn initialize_v8() {
    INIT_V8.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}

fn emit_top_level(nix_expr: &str) -> Result<String, ()> {
    // TODO: Make this faster! Don't transpile if it's already transpiled in a cache somewhere (filesystem)?
    let root = rnix::Root::parse(nix_expr).tree();
    let root_expr = root.expr().expect("Not implemented");
    emit_expr(&root_expr)
}

fn emit_expr(nix_ast: &Expr) -> Result<String, ()> {
    match nix_ast {
        Expr::BinOp(bin_op) => emit_bin_op(&bin_op),
        Expr::Ident(ident) => emit_ident(&ident),
        Expr::Literal(literal) => emit_literal(literal),
        Expr::UnaryOp(unary_op) => emit_unary_op(unary_op),
        _ => panic!("emit_expr: not implemented: {:?}", nix_ast),
    }
}

fn emit_bin_op(bin_op: &BinOp) -> Result<String, ()> {
    let operator = bin_op.operator().expect("Not implemented");
    let lhs = emit_expr(&bin_op.lhs().expect("Not implemented"))?;
    let rhs = emit_expr(&bin_op.rhs().expect("Not implemented"))?;
    match operator {
        // Arithmetic
        BinOpKind::Add => Ok(format!("{} + {}", lhs, rhs)),
        BinOpKind::Div => Ok(format!("Math.floor({} / {})", lhs, rhs)),
        BinOpKind::Mul => Ok(format!("{} * {}", lhs, rhs)),
        BinOpKind::Sub => Ok(format!("{} - {}", lhs, rhs)),
        // Boolean
        BinOpKind::And => Ok(format!("{} && {}", lhs, rhs)),
        BinOpKind::Implication => Ok(format!("!{} || {}", lhs, rhs)),
        BinOpKind::Or => Ok(format!("{} || {}", lhs, rhs)),
        _ => panic!("BinOp not implemented: {:?}", operator),
    }
}

fn emit_ident(ident: &Ident) -> Result<String, ()> {
    let token = ident.ident_token().expect("Not implemented");
    match token.kind() {
        SyntaxKind::TOKEN_IDENT => emit_ident_token(&token),
        _ => todo!(),
    }
}

fn emit_ident_token(token: &SyntaxToken) -> Result<String, ()> {
    Ok(match token.text() {
        "true" => "true".to_owned(),
        "false" => "false".to_owned(),
        _ => todo!(),
    })
}

fn emit_literal(literal: &Literal) -> Result<String, ()> {
    let token = literal.syntax().first_token().expect("Not implemented");
    Ok(match token.kind() {
        SyntaxKind::TOKEN_INTEGER => token.text().to_owned(),
        _ => todo!("emit_literal: {:?}", literal),
    })
}

fn emit_unary_op(unary_op: &UnaryOp) -> Result<String, ()> {
    let operator = unary_op.operator().expect("Not implemented");
    let operand = emit_expr(&unary_op.expr().expect("Not implemented"))?;
    match operator {
        UnaryOpKind::Invert => Ok(format!("!{}", operand)),
        UnaryOpKind::Negate => Ok(format!("-{}", operand)),
    }
}

fn js_value_to_nix(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    js_value: &v8::Local<v8::Value>,
) -> EvalResult {
    if js_value.is_boolean() {
        return Ok(Value::Bool(js_value.is_true()));
    }
    if js_value.is_number() {
        let number = js_value.to_number(scope).unwrap().value() as i64;
        return Ok(Value::Int(number));
    }
    todo!(
        "js_value_to_nix({:?})",
        js_value.to_rust_string_lossy(scope)
    )
}

#[cfg(test)]
mod tests {
    use crate::eval::types::Value;

    use super::*;

    fn eval_ok(nix_expr: &str) -> Value {
        evaluate(nix_expr).expect(&format!("eval '{}' shouldn't fail", nix_expr))
    }

    #[test]
    fn test_eval_int_literals() {
        // TODO: JS doesn't support 64-bit integers. We have to implement this.
        // assert_eq!(
        //     eval_ok(&format!("{}", i64::MAX - 1)),
        //     Value::Int(i64::MAX - 1)
        // );
        assert_eq!(eval_ok(&format!("{}", i64::MAX)), Value::Int(i64::MAX));
        assert_eq!(eval_ok(&format!("{}", i64::MIN)), Value::Int(i64::MIN));
        // assert_eq!(
        //     eval_ok(&format!("{}", i64::MIN + 1)),
        //     Value::Int(i64::MIN + 1)
        // );
    }

    #[test]
    fn test_eval_int_arithmetic() {
        assert_eq!(eval_ok("-1"), Value::Int(-1));
        assert_eq!(eval_ok("1 + 2"), Value::Int(3));
        assert_eq!(eval_ok("1 - 2"), Value::Int(-1));
        assert_eq!(eval_ok("1 * 2"), Value::Int(2));
        assert_eq!(eval_ok("1 / 2"), Value::Int(0));
    }

    #[test]
    fn test_eval_bool_expr() {
        assert_eq!(eval_ok("true"), Value::Bool(true));
        assert_eq!(eval_ok("false"), Value::Bool(false));
        assert_eq!(eval_ok("!false"), Value::Bool(true));
        assert_eq!(eval_ok("false || true"), Value::Bool(true));
        assert_eq!(eval_ok("false || !false"), Value::Bool(true));
        assert_eq!(eval_ok("true && true"), Value::Bool(true));
        assert_eq!(eval_ok("false || true && false"), Value::Bool(false));
        assert_eq!(eval_ok("false && true || false"), Value::Bool(false));
        assert_eq!(eval_ok("true -> false"), Value::Bool(false));
    }
}
