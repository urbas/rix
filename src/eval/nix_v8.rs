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

    let source_str = emit_module(nix_expr)?;
    let source_v8 = to_v8_source(scope, &source_str);
    let module = v8::script_compiler::compile_module(scope, source_v8).unwrap();

    {
        let try_catch = &mut v8::TryCatch::new(scope);
        if let None = module.instantiate_module(try_catch, resolve_module_callback) {
            let exception = try_catch.exception().unwrap();
            todo!(
                "Instantiation failure: {:?}",
                exception.to_rust_string_lossy(try_catch)
            )
        }
    }
    module.evaluate(scope).unwrap();
    let namespace_obj = module.get_module_namespace().to_object(scope).unwrap();
    nix_value_from_module(scope, &namespace_obj)
}

fn initialize_v8() {
    INIT_V8.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}

fn emit_module(nix_expr: &str) -> Result<String, ()> {
    let root = rnix::Root::parse(nix_expr).tree();
    let root_expr = root.expr().expect("Not implemented");
    let mut out_src =
        "import nixrt from 'nixrt/lib';\nexport const __nixrt = nixrt; export const __nix_value = "
            .to_owned();
    emit_expr(&root_expr, &mut out_src)?;
    out_src += ";";
    // eprintln!("JS Source:\n```\n{out_src:?}\n```");
    Ok(out_src)
}

fn emit_expr(nix_ast: &Expr, out_src: &mut String) -> Result<(), ()> {
    match nix_ast {
        Expr::BinOp(bin_op) => emit_bin_op(&bin_op, out_src),
        Expr::Ident(ident) => emit_ident(&ident, out_src),
        Expr::Literal(literal) => emit_literal(literal, out_src),
        Expr::UnaryOp(unary_op) => emit_unary_op(unary_op, out_src),
        _ => panic!("emit_expr: not implemented: {:?}", nix_ast),
    }
}

fn emit_bin_op(bin_op: &BinOp, out_src: &mut String) -> Result<(), ()> {
    let operator = bin_op.operator().expect("Not implemented");
    let lhs = &bin_op.lhs().expect("Not implemented");
    let rhs = &bin_op.rhs().expect("Not implemented");
    match operator {
        // Arithmetic
        BinOpKind::Add => emit_nixrt_bin_op(lhs, rhs, "nixrt.add", out_src)?,
        BinOpKind::Div => emit_nixrt_bin_op(lhs, rhs, "nixrt.div", out_src)?,
        BinOpKind::Mul => emit_nixrt_bin_op(lhs, rhs, "nixrt.mul", out_src)?,
        BinOpKind::Sub => emit_nixrt_bin_op(lhs, rhs, "nixrt.sub", out_src)?,
        // Boolean
        BinOpKind::And => emit_regular_bin_op(lhs, rhs, "&&", out_src)?,
        BinOpKind::Implication => emit_implication_bin_op(lhs, rhs, out_src)?,
        BinOpKind::Or => emit_regular_bin_op(lhs, rhs, "||", out_src)?,
        _ => panic!("BinOp not implemented: {:?}", operator),
    }
    Ok(())
}

fn emit_nixrt_bin_op(
    lhs: &Expr,
    rhs: &Expr,
    nixrt_function: &str,
    out_src: &mut String,
) -> Result<(), ()> {
    *out_src += nixrt_function;
    *out_src += "(";
    emit_expr(lhs, out_src)?;
    *out_src += ",";
    emit_expr(rhs, out_src)?;
    *out_src += ")";
    Ok(())
}

fn emit_regular_bin_op(
    lhs: &Expr,
    rhs: &Expr,
    operator: &str,
    out_src: &mut String,
) -> Result<(), ()> {
    emit_expr(lhs, out_src)?;
    *out_src += operator;
    emit_expr(rhs, out_src)
}

fn emit_implication_bin_op(lhs: &Expr, rhs: &Expr, out_src: &mut String) -> Result<(), ()> {
    emit_unary_op_kind(UnaryOpKind::Invert, lhs, out_src)?;
    *out_src += " || ";
    emit_expr(rhs, out_src)
}

fn emit_ident(ident: &Ident, out_src: &mut String) -> Result<(), ()> {
    let token = ident.ident_token().expect("Not implemented");
    match token.kind() {
        SyntaxKind::TOKEN_IDENT => emit_ident_token(&token, out_src),
        _ => todo!(),
    }
}

fn emit_ident_token(token: &SyntaxToken, out_src: &mut String) -> Result<(), ()> {
    *out_src += token.text();
    Ok(())
}

fn emit_literal(literal: &Literal, out_src: &mut String) -> Result<(), ()> {
    let token = literal.syntax().first_token().expect("Not implemented");
    match token.kind() {
        SyntaxKind::TOKEN_INTEGER => *out_src += &format!("new nixrt.NixInt({})", token.text()),
        SyntaxKind::TOKEN_FLOAT => *out_src += token.text(),
        _ => todo!("emit_literal: {:?}", literal),
    }
    Ok(())
}

fn emit_unary_op(unary_op: &UnaryOp, out_src: &mut String) -> Result<(), ()> {
    let operator = unary_op.operator().expect("Not implemented");
    let operand = unary_op.expr().expect("Not implemented");
    emit_unary_op_kind(operator, &operand, out_src)
}

fn emit_unary_op_kind(
    operator: UnaryOpKind,
    operand: &Expr,
    out_src: &mut String,
) -> Result<(), ()> {
    match operator {
        UnaryOpKind::Invert => {
            *out_src += "!";
            emit_expr(operand, out_src)?;
        }
        UnaryOpKind::Negate => {
            *out_src += "nixrt.neg(";
            emit_expr(operand, out_src)?;
            *out_src += ")";
        }
    }
    Ok(())
}

fn nix_value_from_module(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    namespace_obj: &v8::Local<v8::Object>,
) -> EvalResult {
    let nix_value_attr = v8::String::new(scope, "__nix_value").unwrap();
    let Some(nix_value) = namespace_obj
        .get(scope, nix_value_attr.into()) else {
            todo!("Namespace obj: {:?}", namespace_obj.to_rust_string_lossy(scope))
        };
    let nixrt_attr = v8::String::new(scope, "__nixrt").unwrap();
    let nixrt: v8::Local<v8::Value> = namespace_obj
        .get(scope, nixrt_attr.into())
        .unwrap()
        .try_into()
        .unwrap();
    js_value_to_nix(scope, &nixrt, &nix_value)
}

fn js_value_to_nix(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> EvalResult {
    if js_value.is_boolean() {
        return Ok(Value::Bool(js_value.is_true()));
    }
    if js_value.is_number() {
        let number = js_value
            .to_number(scope)
            .unwrap()
            .number_value(scope)
            .unwrap();
        return Ok(Value::Float(number));
    }
    if js_value.is_object() {
        let nix_int_class_name = v8::String::new(scope, "NixInt").unwrap();
        let nixrt_nix_int = nixrt
            .to_object(scope)
            .unwrap()
            .get(scope, nix_int_class_name.into())
            .unwrap()
            .to_object(scope)
            .unwrap();
        let js_object = js_value.to_object(scope).unwrap();
        let is_nix_int = js_object.instance_of(scope, nixrt_nix_int).unwrap();
        if is_nix_int {
            let nix_int_value_attr = v8::String::new(scope, "value").unwrap();
            let int_value = js_object.get(scope, nix_int_value_attr.into()).unwrap();
            return Ok(Value::Int(
                int_value
                    .to_number(scope)
                    .unwrap()
                    .number_value(scope)
                    .unwrap() as i64,
            ));
        }
        todo!("{nixrt_nix_int:?}, is instance: {is_nix_int:?}")
    }
    if js_value.is_big_int() {
        let (number, _) = js_value.to_big_int(scope).unwrap().i64_value();
        return Ok(Value::Int(number));
    }
    todo!(
        "js_value_to_nix: {:?}",
        js_value.to_rust_string_lossy(scope)
    )
}

fn new_script_origin<'s>(
    scope: &mut v8::HandleScope<'s>,
    resource_name: &str,
) -> v8::ScriptOrigin<'s> {
    let resource_name_v8_str = v8::String::new(scope, resource_name).unwrap();
    let resource_line_offset = 0;
    let resource_column_offset = 0;
    let resource_is_shared_cross_origin = true;
    let script_id = 123;
    let source_map_url = v8::String::new(scope, "source_map_url").unwrap();
    let resource_is_opaque = true;
    let is_wasm = false;
    let is_module = true;
    v8::ScriptOrigin::new(
        scope,
        resource_name_v8_str.into(),
        resource_line_offset,
        resource_column_offset,
        resource_is_shared_cross_origin,
        script_id,
        source_map_url.into(),
        resource_is_opaque,
        is_wasm,
        is_module,
    )
}

fn to_v8_source<'a>(scope: &mut v8::HandleScope, js_code: &str) -> v8::script_compiler::Source {
    let code = v8::String::new(scope, js_code).unwrap();
    let origin = new_script_origin(scope, "top_level.mjs");
    v8::script_compiler::Source::new(code, Some(&origin))
}

fn resolve_module_callback<'a>(
    context: v8::Local<'a, v8::Context>,
    _specifier: v8::Local<'a, v8::String>,
    _import_assertions: v8::Local<'a, v8::FixedArray>,
    _referrer: v8::Local<'a, v8::Module>,
) -> Option<v8::Local<'a, v8::Module>> {
    let scope = &mut unsafe { v8::CallbackScope::new(context) };
    if _specifier.to_rust_string_lossy(scope) != "nixrt/lib" {
        todo!(
            "resolve_module_callback: {:?}",
            _specifier.to_rust_string_lossy(scope),
        )
    }
    let module_source_str = std::fs::read_to_string(env!("RIX_NIXRT_JS_MODULE")).unwrap();
    let module_source_v8 = to_v8_source(scope, &module_source_str);
    let module = v8::script_compiler::compile_module(scope, module_source_v8).unwrap();
    Some(module)
}

#[cfg(test)]
mod tests {
    use crate::eval::types::Value;

    use super::*;

    fn eval_ok(nix_expr: &str) -> Value {
        evaluate(nix_expr).expect(&format!("eval '{}' shouldn't fail", nix_expr))
    }

    #[test]
    fn test_eval_int_arithmetic() {
        assert_eq!(eval_ok("1"), Value::Int(1));
        assert_eq!(eval_ok("-1"), Value::Int(-1));
        assert_eq!(eval_ok("1 + 2"), Value::Int(3));
        assert_eq!(eval_ok("1 - 2"), Value::Int(-1));
        assert_eq!(eval_ok("1 * 2"), Value::Int(2));
        assert_eq!(eval_ok("1 / 2"), Value::Int(0));
    }

    #[test]
    fn test_eval_float_arithmetic() {
        assert_eq!(eval_ok("1.0"), Value::Float(1.0));
        assert_eq!(eval_ok(".27e13"), Value::Float(0.27e13));
        assert_eq!(eval_ok("-1.0"), Value::Float(-1.0));
        assert_eq!(eval_ok("1.0 + 2.0"), Value::Float(3.0));
        assert_eq!(eval_ok("1.0 - 2.0"), Value::Float(-1.0));
        assert_eq!(eval_ok("1.0 * 2.0"), Value::Float(2.0));
        assert_eq!(eval_ok("1.0 / 2.0"), Value::Float(0.5));
    }

    #[test]
    fn test_eval_mixed_arithmetic() {
        assert_eq!(eval_ok("1 + 2.0"), Value::Float(3.0));
        assert_eq!(eval_ok("1 - 2.0"), Value::Float(-1.0));
        assert_eq!(eval_ok("1 * 2.0"), Value::Float(2.0));
        assert_eq!(eval_ok("1 / 2.0"), Value::Float(0.5));
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
