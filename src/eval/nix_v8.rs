use std::sync::Once;

use rnix::{
    ast::{BinOp, BinOpKind, Expr, Ident, UnaryOp, UnaryOpKind},
    SyntaxKind, SyntaxToken,
};

use crate::eval::types::EvalResult;
use crate::eval::types::Value;

static INIT_V8: Once = Once::new();

pub fn evaluate(nix_expr: &str) -> EvalResult {
    initialize_v8();
    let isolate = &mut v8::Isolate::new(Default::default());
    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);
    let js_expr = nix_expr_to_js(nix_expr)?;
    let code = v8::String::new(scope, &js_expr).unwrap();
    // TODO: Make this faster! Maybe we can use v8's compiled code caching to speed things up a notch?
    let script = v8::Script::compile(scope, code, None).unwrap();
    let result = script.run(scope).unwrap();
    js_value_to_nix(&result)
}

fn js_value_to_nix(js_value: &v8::Local<v8::Value>) -> EvalResult {
    if js_value.is_boolean() {
        return Ok(Value::Bool(js_value.is_true()));
    }
    todo!("js_value_to_nix({:?})", js_value.is_boolean())
}

fn initialize_v8() {
    INIT_V8.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}

fn nix_expr_to_js(nix_expr: &str) -> Result<String, ()> {
    // TODO: Make this faster! Don't transpile if it's already transpiled in a cache somewhere (filesystem)?
    let root = rnix::Root::parse(nix_expr).tree();
    let root_expr = root.expr().expect("Not implemented");
    ast_to_js(&root_expr)
}

fn ast_to_js(nix_ast: &Expr) -> Result<String, ()> {
    match nix_ast {
        Expr::BinOp(bin_op) => bin_op_to_js(&bin_op),
        Expr::Ident(ident) => ident_to_js(&ident),
        Expr::UnaryOp(unary_op) => unary_op_to_js(unary_op),
        _ => panic!("Not implemented: {:?}", nix_ast),
    }
}

fn bin_op_to_js(bin_op: &BinOp) -> Result<String, ()> {
    let operator = bin_op.operator().expect("Not implemented");
    let lhs = ast_to_js(&bin_op.lhs().expect("Not implemented"))?;
    let rhs = ast_to_js(&bin_op.rhs().expect("Not implemented"))?;
    match operator {
        // Boolean
        BinOpKind::And => Ok(format!("{} && {}", lhs, rhs)),
        BinOpKind::Implication => Ok(format!("!{} || {}", lhs, rhs)),
        BinOpKind::Or => Ok(format!("{} || {}", lhs, rhs)),
        _ => panic!("BinOp not implemented: {:?}", operator),
    }
}

fn ident_to_js(ident: &Ident) -> Result<String, ()> {
    let token = ident.ident_token().expect("Not implemented");
    match token.kind() {
        SyntaxKind::TOKEN_IDENT => ident_token_to_js(&token),
        _ => todo!(),
    }
}

fn ident_token_to_js(token: &SyntaxToken) -> Result<String, ()> {
    Ok(match token.text() {
        "true" => "true".to_owned(),
        "false" => "false".to_owned(),
        _ => todo!(),
    })
}

fn unary_op_to_js(unary_op: &UnaryOp) -> Result<String, ()> {
    let operator = unary_op.operator().expect("Not implemented");
    let operand = ast_to_js(&unary_op.expr().expect("Not implemented"))?;
    match operator {
        UnaryOpKind::Invert => Ok(format!("!{}", operand)),
        _ => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::eval::types::Value;

    use super::*;

    fn eval_ok(nix_expr: &str) -> Value {
        evaluate(nix_expr).expect("Shouldn't fail")
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
