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

    let js_source = emit_module(nix_expr)?;
    let v8_source = to_v8_source(scope, &js_source);

    // TODO: Make this faster! Maybe we can use v8's compiled code caching to speed things up a notch?
    let module = v8::script_compiler::compile_module(scope, v8_source).unwrap();
    module
        .instantiate_module(scope, resolve_module_callback)
        .unwrap();
    module.evaluate(scope).unwrap();

    let namespace = module.get_module_namespace();
    let namespace_obj = namespace.to_object(scope).unwrap();
    let nix_value_attr = v8::String::new(scope, "nix_value").unwrap();
    let nix_value: v8::Local<v8::Value> = namespace_obj
        .get(scope, nix_value_attr.into())
        .unwrap()
        .try_into()
        .unwrap();

    js_value_to_nix(scope, &nix_value)
}

fn initialize_v8() {
    INIT_V8.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}

fn emit_module(nix_expr: &str) -> Result<String, ()> {
    // TODO: Make this faster! Don't transpile if it's already transpiled in a cache.
    let root = rnix::Root::parse(nix_expr).tree();
    let root_expr = root.expr().expect("Not implemented");
    let body = emit_expr(&root_expr)?;
    Ok(format!(
        r#"// import {{NixToJs}} from 'nixjs';
export const nix_value = {};
"#,
        body
    ))
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

fn to_v8_source(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    js_code: &str,
) -> v8::script_compiler::Source {
    let code = v8::String::new(scope, js_code).unwrap();
    let resource_name = v8::String::new(scope, "top_level").unwrap().into();
    let source_map_url = v8::String::new(scope, "").unwrap().into();
    let origin = v8::ScriptOrigin::new(
        scope,
        resource_name,
        0,
        0,
        false,
        0,
        source_map_url,
        false,
        false,
        true,
    );
    v8::script_compiler::Source::new(code, Some(&origin))
}

fn resolve_module_callback<'a>(
    context: v8::Local<'a, v8::Context>,
    _specifier: v8::Local<'a, v8::String>,
    _import_assertions: v8::Local<'a, v8::FixedArray>,
    _referrer: v8::Local<'a, v8::Module>,
) -> Option<v8::Local<'a, v8::Module>> {
    // TODO: figure out how to best ship the Nix Runtime JavaScript (NixJS) libraries.
    // 1.) NixJS should be testable in the standard JS way (without builtins that require Rust code)
    // 2.) Stack traces should refer back to the original nix code
    // 3.) Must be able to attach a profiler to a rix evaluation
    // 4.) Must be able to attach a debugger to a rix evaluation and add breakpoints straight to original nix code
    let scope = &mut unsafe { v8::CallbackScope::new(context) };
    let scope = &mut v8::HandleScope::new(scope);
    // TODO: compile the module and return it, something along these lines:
    // let source = v8::script_compiler::Source::new(specifier, Some(&origin));
    // let module = v8::script_compiler::compile_module(scope, source).unwrap();
    todo!(
        "resolve_module_callback: {:?}",
        _specifier.to_rust_string_lossy(scope)
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
