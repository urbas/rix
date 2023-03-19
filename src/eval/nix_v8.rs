use std::env::current_dir;
use std::path::Path;
use std::{collections::HashMap, sync::Once};

use rnix::{ast, ast::HasEntry, SyntaxKind, SyntaxToken};
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
    let source_v8 = to_v8_source(scope, &source_str, "<eval string>");
    let module = v8::script_compiler::compile_module(scope, source_v8).unwrap();

    if let None = module.instantiate_module(scope, resolve_module_callback) {
        todo!("Instantiation failure.")
    }
    let Some(_) = module.evaluate(scope) else {
        todo!("evaluation failed")
    };

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

pub fn emit_module(nix_expr: &str) -> Result<String, String> {
    let root = rnix::Root::parse(nix_expr).tree();
    let root_expr = root.expr().expect("Not implemented");
    let nixrt_js_module = env!("RIX_NIXRT_JS_MODULE");
    let mut out_src = format!("import nixrt from '{nixrt_js_module}';\n");
    out_src += "export const __nixrt = nixrt;\n";
    out_src += "export const __nixValue = (evalCtx) => {return ";
    emit_expr(&root_expr, &mut out_src)?;
    out_src += ";};\n";
    Ok(out_src)
}

fn emit_expr(nix_ast: &ast::Expr, out_src: &mut String) -> Result<(), String> {
    match nix_ast {
        ast::Expr::Apply(apply) => emit_apply(apply, out_src),
        ast::Expr::AttrSet(attrset) => emit_attrset(attrset, out_src),
        ast::Expr::BinOp(bin_op) => emit_bin_op(bin_op, out_src),
        ast::Expr::HasAttr(has_attr) => emit_has_attr(has_attr, out_src),
        ast::Expr::Ident(ident) => emit_ident(ident, out_src),
        ast::Expr::IfElse(if_else) => emit_if_else(if_else, out_src),
        ast::Expr::Lambda(lambda) => emit_lambda(lambda, out_src),
        ast::Expr::List(list) => emit_list(list, out_src),
        ast::Expr::Literal(literal) => emit_literal(literal, out_src),
        ast::Expr::Paren(paren) => emit_paren(paren, out_src),
        ast::Expr::Path(path) => emit_path(path, out_src),
        ast::Expr::Select(select) => emit_select_expr(select, out_src),
        ast::Expr::Str(string) => emit_string_expr(string, out_src),
        ast::Expr::UnaryOp(unary_op) => emit_unary_op(unary_op, out_src),
        _ => panic!("emit_expr: not implemented: {:?}", nix_ast),
    }
}

fn emit_apply(apply: &ast::Apply, out_src: &mut String) -> Result<(), String> {
    emit_nixrt_bin_op(
        &apply
            .lambda()
            .expect("Unexpected lambda application without the lambda."),
        &apply
            .argument()
            .expect("Unexpected lambda application without arguments."),
        "nixrt.apply",
        out_src,
    )
}

fn emit_attrset(attrset: &ast::AttrSet, out_src: &mut String) -> Result<(), String> {
    *out_src += "nixrt.attrset(";
    for attrpath_value in attrset.attrpath_values() {
        *out_src += "[";
        let attrpath = attrpath_value.attrpath().expect("Not implemented");
        let value = &attrpath_value.value().expect("Not implemented");
        emit_attrpath(&attrpath, out_src)?;
        *out_src += ",";
        emit_expr(value, out_src)?;
        *out_src += "],";
    }
    *out_src += ")";
    Ok(())
}

fn emit_attrpath(attrpath: &ast::Attrpath, out_src: &mut String) -> Result<(), String> {
    *out_src += "nixrt.attrpath(";
    for attr in attrpath.attrs() {
        match attr {
            ast::Attr::Ident(ident) => {
                *out_src += "\"";
                *out_src += ident.ident_token().expect("Not implemented").text();
                *out_src += "\"";
            }
            ast::Attr::Str(str_expression) => emit_string_expr(&str_expression, out_src)?,
            ast::Attr::Dynamic(expr) => {
                emit_expr(&expr.expr().expect("Expected an expression."), out_src)?
            }
        }
        *out_src += ",";
    }
    *out_src += ")";
    Ok(())
}

fn emit_bin_op(bin_op: &ast::BinOp, out_src: &mut String) -> Result<(), String> {
    let operator = bin_op.operator().expect("Not implemented");
    let lhs = &bin_op.lhs().expect("Not implemented");
    let rhs = &bin_op.rhs().expect("Not implemented");
    match operator {
        // Arithmetic
        ast::BinOpKind::Add => emit_add_bin_op(lhs, rhs, out_src)?,
        ast::BinOpKind::Div => emit_nixrt_bin_op(lhs, rhs, "nixrt.div", out_src)?,
        ast::BinOpKind::Mul => emit_nixrt_bin_op(lhs, rhs, "nixrt.mul", out_src)?,
        ast::BinOpKind::Sub => emit_nixrt_bin_op(lhs, rhs, "nixrt.sub", out_src)?,

        // Attrset
        ast::BinOpKind::Update => emit_nixrt_bin_op(lhs, rhs, "nixrt.update", out_src)?,

        // Boolean
        ast::BinOpKind::And => emit_nixrt_bin_op(lhs, rhs, "nixrt.and", out_src)?,
        ast::BinOpKind::Implication => emit_nixrt_bin_op(lhs, rhs, "nixrt.implication", out_src)?,
        ast::BinOpKind::Or => emit_nixrt_bin_op(lhs, rhs, "nixrt.or", out_src)?,

        // Comparison
        ast::BinOpKind::Equal => emit_nixrt_bin_op(lhs, rhs, "nixrt.eq", out_src)?,
        ast::BinOpKind::Less => emit_nixrt_bin_op(lhs, rhs, "nixrt.less", out_src)?,
        ast::BinOpKind::LessOrEq => emit_nixrt_bin_op(lhs, rhs, "nixrt.less_eq", out_src)?,
        ast::BinOpKind::More => emit_nixrt_bin_op(lhs, rhs, "nixrt.more", out_src)?,
        ast::BinOpKind::MoreOrEq => emit_nixrt_bin_op(lhs, rhs, "nixrt.more_eq", out_src)?,
        ast::BinOpKind::NotEqual => emit_nixrt_bin_op(lhs, rhs, "nixrt.neq", out_src)?,

        // List
        ast::BinOpKind::Concat => emit_nixrt_bin_op(lhs, rhs, "nixrt.concat", out_src)?,
    }
    Ok(())
}

fn emit_add_bin_op(lhs: &ast::Expr, rhs: &ast::Expr, out_src: &mut String) -> Result<(), String> {
    *out_src += "nixrt.add(evalCtx,";
    emit_expr(lhs, out_src)?;
    *out_src += ",";
    emit_expr(rhs, out_src)?;
    *out_src += ")";
    Ok(())
}

fn emit_nixrt_bin_op(
    lhs: &ast::Expr,
    rhs: &ast::Expr,
    nixrt_function: &str,
    out_src: &mut String,
) -> Result<(), String> {
    *out_src += nixrt_function;
    *out_src += "(";
    emit_expr(lhs, out_src)?;
    *out_src += ",";
    emit_expr(rhs, out_src)?;
    *out_src += ")";
    Ok(())
}

fn emit_ident(ident: &ast::Ident, out_src: &mut String) -> Result<(), String> {
    let token = ident.ident_token().expect("Not implemented");
    match token.kind() {
        SyntaxKind::TOKEN_IDENT => emit_ident_token(&token, out_src),
        _ => todo!(),
    }
}

fn emit_ident_token(token: &SyntaxToken, out_src: &mut String) -> Result<(), String> {
    *out_src += token.text();
    Ok(())
}

fn emit_has_attr(has_attr: &ast::HasAttr, out_src: &mut String) -> Result<(), String> {
    *out_src += "nixrt.has(";
    emit_expr(&has_attr.expr().expect("Unreachable"), out_src)?;
    *out_src += ",";
    emit_attrpath(&has_attr.attrpath().expect("Unreachable"), out_src)?;
    *out_src += ")";
    Ok(())
}

fn emit_if_else(lambda: &ast::IfElse, out_src: &mut String) -> Result<(), String> {
    let condition = lambda
        .condition()
        .expect("Unexpected 'if-then-else' expression without a condition.");
    let body = lambda
        .body()
        .expect("Unexpected 'if-then-else' expression without a body.");
    let else_body = lambda
        .else_body()
        .expect("Unexpected 'if-then-else' expression without an 'else' body.");
    *out_src += "nixrt.asBoolean(";
    emit_expr(&condition, out_src)?;
    *out_src += ") ? (";
    emit_expr(&body, out_src)?;
    *out_src += ") : (";
    emit_expr(&else_body, out_src)?;
    *out_src += ")";
    Ok(())
}

fn emit_lambda(lambda: &ast::Lambda, out_src: &mut String) -> Result<(), String> {
    let _param = lambda
        .param()
        .expect("Unexpected lambda without parameters.");
    let body = lambda
        .body()
        .expect("Unexpected lambda without parameters.");
    *out_src += "new nixrt.Lambda(";
    emit_expr(&body, out_src)?;
    *out_src += ")";
    Ok(())
}

fn emit_list(list: &ast::List, out_src: &mut String) -> Result<(), String> {
    *out_src += "[";
    for element in list.items() {
        emit_expr(&element, out_src)?;
        *out_src += ",";
    }
    *out_src += "]";
    Ok(())
}

fn emit_literal(literal: &ast::Literal, out_src: &mut String) -> Result<(), String> {
    let token = literal.syntax().first_token().expect("Not implemented");
    match token.kind() {
        SyntaxKind::TOKEN_INTEGER => *out_src += &format!("new nixrt.NixInt({}n)", token.text()),
        SyntaxKind::TOKEN_FLOAT => *out_src += token.text(),
        SyntaxKind::TOKEN_URI => {
            out_src.push('`');
            js_string_escape_into(token.text(), out_src);
            out_src.push('`');
        }
        _ => todo!("emit_literal: {:?} token kind: {:?}", literal, token.kind()),
    }
    Ok(())
}

fn emit_paren(paren: &ast::Paren, out_src: &mut String) -> Result<(), String> {
    *out_src += "(";
    let body = paren
        .expr()
        .expect("Unexpected parenthesis without a body.");
    emit_expr(&body, out_src)?;
    *out_src += ")";
    Ok(())
}

fn emit_path(path: &ast::Path, out_src: &mut String) -> Result<(), String> {
    *out_src += "nixrt.toPath(evalCtx,`";
    js_string_escape_into(&path.to_string(), out_src);
    *out_src += "`)";
    Ok(())
}

fn emit_select_expr(select: &ast::Select, out_src: &mut String) -> Result<(), String> {
    *out_src += "nixrt.select(";
    emit_expr(&select.expr().expect("Unreachable"), out_src)?;
    *out_src += ",";
    emit_attrpath(&select.attrpath().expect("Unreachable"), out_src)?;
    *out_src += ",";
    match select.default_expr() {
        Some(default_expr) => emit_expr(&default_expr, out_src)?,
        None => *out_src += "undefined",
    }
    *out_src += ")";
    Ok(())
}

fn emit_string_expr(string: &ast::Str, out_src: &mut String) -> Result<(), String> {
    *out_src += "`";
    for string_part in string.normalized_parts() {
        match string_part {
            ast::InterpolPart::Literal(literal) => {
                js_string_escape_into(&literal, out_src);
            }
            ast::InterpolPart::Interpolation(interpolation_body) => {
                *out_src += "${nixrt.interpolate(";
                emit_expr(
                    &interpolation_body
                        .expr()
                        .expect("String interpolation body missing."),
                    out_src,
                )?;
                *out_src += ")}";
            }
        }
    }
    *out_src += "`";
    Ok(())
}

fn js_string_escape_into(string: &str, out_string: &mut String) {
    for character in string.chars() {
        match character {
            '`' => out_string.push_str(r#"\`"#),
            '$' => out_string.push_str(r#"\$"#),
            '\\' => out_string.push_str(r#"\\"#),
            '\r' => out_string.push_str(r#"\r"#),
            character => out_string.push(character),
        }
    }
}

fn emit_unary_op(unary_op: &ast::UnaryOp, out_src: &mut String) -> Result<(), String> {
    let operator = unary_op.operator().expect("Not implemented");
    let operand = unary_op.expr().expect("Not implemented");
    emit_unary_op_kind(operator, &operand, out_src)
}

fn emit_unary_op_kind(
    operator: ast::UnaryOpKind,
    operand: &ast::Expr,
    out_src: &mut String,
) -> Result<(), String> {
    match operator {
        ast::UnaryOpKind::Invert => emit_nixrt_unary_op(operand, "nixrt.invert", out_src),
        ast::UnaryOpKind::Negate => emit_nixrt_unary_op(operand, "nixrt.neg", out_src),
    }
}

fn emit_nixrt_unary_op(
    operand: &ast::Expr,
    nixrt_function: &str,
    out_src: &mut String,
) -> Result<(), String> {
    *out_src += nixrt_function;
    *out_src += "(";
    emit_expr(operand, out_src)?;
    *out_src += ")";
    Ok(())
}

fn nix_value_from_module(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    namespace_obj: &v8::Local<v8::Object>,
) -> EvalResult {
    let nix_value_attr = v8::String::new(scope, "__nixValue").unwrap();
    let Some(nix_value) = namespace_obj
        .get(scope, nix_value_attr.into()) else {
            todo!("Could not find the nix value: {:?}", namespace_obj.to_rust_string_lossy(scope))
        };
    let nix_value: v8::Local<v8::Function> =
        nix_value.try_into().expect("Nix value must be a function.");

    let eval_ctx = create_eval_ctx(
        scope,
        &current_dir().map_err(|err| {
            format!("Failed to determine the current working directory. Error: {err}")
        })?,
    )?;

    let scope = &mut v8::TryCatch::new(scope);
    let recv = v8::undefined(scope).into();
    let Some(nix_value) = nix_value.call(scope, recv, &[eval_ctx.into()]) else {
        // TODO: The stack trace needs to be source-mapped. Unfortunately, this doesn't
        // seem to be supported yet: https://github.com/denoland/deno/issues/4499
        let err_msg = scope
            .stack_trace()
            .map_or(
                "Unknown evaluation error.".to_owned(),
                |stack| stack.to_rust_string_lossy(scope),
            );
        return Err(err_msg);
    };

    let nixrt_attr = v8::String::new(scope, "__nixrt").unwrap();
    let nixrt: v8::Local<v8::Value> = namespace_obj
        .get(scope, nixrt_attr.into())
        .unwrap()
        .try_into()
        .unwrap();
    js_value_to_nix(scope, &nixrt, &nix_value)
}

fn create_eval_ctx<'s>(
    scope: &mut v8::HandleScope<'s>,
    script_path: &Path,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let eval_ctx = v8::Object::new(scope);

    let real_path = script_path
        .canonicalize()
        .map_err(|err| format!("Failed to resolve the script path. Error: {err}."))?;
    let script_dir = real_path
        .parent()
        .ok_or_else(|| format!("Failed to determine the directory of path {real_path:?}."))?;
    let script_dir_str = real_path
        .to_str()
        .ok_or_else(|| format!("Failed to converft the path {script_dir:?} to a string."))?;
    let js_script_dir_name =
        v8::String::new(scope, "scriptDir").expect("Unexpected internal error.");
    let js_script_dir_path =
        v8::String::new(scope, script_dir_str).expect("Unexpected internal error.");
    eval_ctx.set(scope, js_script_dir_name.into(), js_script_dir_path.into());

    Ok(eval_ctx)
}

fn js_value_to_nix<'s>(
    scope: &mut v8::HandleScope<'s>,
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
    if let Some(value) = js_value_as_nix_int(scope, nixrt, js_value)? {
        return Ok(value);
    }
    if let Some(value) = js_value_as_nix_string(scope, js_value) {
        return Ok(value);
    }
    if let Some(value) = js_value_as_nix_array(scope, nixrt, js_value) {
        return value;
    }
    if let Some(value) = js_value_as_attrset(scope, nixrt, js_value) {
        return value;
    }
    if let Some(value) = js_object_as_nix_value(scope, nixrt, js_value)? {
        return Ok(value);
    }
    todo!(
        "js_value_to_nix: {:?}",
        js_value.to_rust_string_lossy(scope)
    )
}

fn js_value_as_nix_int<'s>(
    scope: &mut v8::HandleScope<'s>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> Result<Option<Value>, String> {
    if is_nixrt_type(scope, nixrt, js_value, "NixInt")? {
        let Some(int64_js_value) = try_get_js_object_key(scope, js_value, "int64")? else {
            return Ok(None);
        };
        let big_int_value: v8::Local<v8::BigInt> = int64_js_value.try_into().map_err(|err| {
            format!("Expected an int64 value. Internal conversion error: {err:?}")
        })?;
        return Ok(Some(Value::Int(big_int_value.i64_value().0)));
    }
    Ok(None)
}

fn get_nixrt_type<'s>(
    scope: &mut v8::HandleScope<'s>,
    nixrt: &v8::Local<v8::Value>,
    type_name: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let nix_int_class_name = v8::String::new(scope, type_name).unwrap();
    nixrt
        .to_object(scope)
        .unwrap()
        .get(scope, nix_int_class_name.into())
        .unwrap()
        .to_object(scope)
        .ok_or_else(|| format!("Could not find the type {type_name}."))
}

fn is_nixrt_type<'s>(
    scope: &mut v8::HandleScope<'s>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
    type_name: &str,
) -> Result<bool, String> {
    let nixrt_type = get_nixrt_type(scope, nixrt, type_name)?;
    js_value.instance_of(scope, nixrt_type).ok_or_else(|| {
        format!(
            "Failed to check whether value '{}' is 'nixrt.{type_name}'.",
            js_value.to_rust_string_lossy(scope)
        )
    })
}

fn js_value_as_nix_string<'s>(
    scope: &mut v8::HandleScope<'s>,
    js_value: &v8::Local<v8::Value>,
) -> Option<Value> {
    if js_value.is_string() {
        let string = js_value
            .to_string(scope)
            .unwrap()
            .to_rust_string_lossy(scope);
        return Some(Value::Str(string));
    }
    None
}

fn js_value_as_nix_array<'s>(
    scope: &mut v8::HandleScope<'s>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> Option<EvalResult> {
    let js_array: Result<v8::Local<v8::Array>, _> = (*js_value).try_into();
    match js_array {
        Ok(js_array) => {
            let length = js_array.length();
            let mut rust_array = Vec::with_capacity(length as usize);
            for idx in 0..length {
                let js_element = js_array.get_index(scope, idx).unwrap();
                match js_value_to_nix(scope, nixrt, &js_element) {
                    Ok(value) => rust_array.push(value),
                    err => return Some(err),
                }
            }
            return Some(Ok(Value::List(rust_array)));
        }
        Err(_) => None,
    }
}

fn js_object_as_nix_value<'s>(
    scope: &mut v8::HandleScope<'s>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> Result<Option<Value>, String> {
    if is_nixrt_type(scope, nixrt, js_value, "Lambda")? {
        return Ok(Some(Value::Lambda));
    }
    js_value_as_nix_path(scope, nixrt, js_value)
}

fn js_value_as_nix_path<'s>(
    scope: &mut v8::HandleScope<'s>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> Result<Option<Value>, String> {
    if !is_nixrt_type(scope, nixrt, js_value, "Path")? {
        return Ok(None);
    }
    let Some(path) = try_get_js_object_key(scope, js_value, "path")? else {
        return Ok(None);
    };
    Ok(Some(Value::Path(path.to_rust_string_lossy(scope))))
}

fn try_get_js_object_key<'s>(
    scope: &mut v8::HandleScope<'s>,
    js_value: &v8::Local<v8::Value>,
    key: &str,
) -> Result<Option<v8::Local<'s, v8::Value>>, String> {
    let js_object = js_value
        .to_object(scope)
        .ok_or_else(|| "Not an object.".to_owned())?;
    let key_js_str = v8::String::new(scope, key).unwrap();
    Ok(js_object.get(scope, key_js_str.into()))
}

fn js_value_as_attrset<'s>(
    scope: &mut v8::HandleScope<'s>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> Option<EvalResult> {
    let js_map: Result<v8::Local<v8::Map>, _> = (*js_value).try_into();
    match js_map {
        Ok(js_map) => Some(js_map_as_attrset(scope, nixrt, &js_map)),
        Err(_) => None,
    }
}

fn js_map_as_attrset<'s>(
    scope: &mut v8::HandleScope<'s>,
    nixrt: &v8::Local<v8::Value>,
    js_map: &v8::Local<v8::Map>,
) -> EvalResult {
    let mut map: HashMap<String, Value> = HashMap::new();
    let js_map_array = js_map.as_array(scope);
    for idx in 0..js_map_array.length() / 2 {
        let key_idx = idx * 2;
        let value_idx = key_idx + 1;
        let key: v8::Local<v8::String> = js_map_array
            .get_index(scope, key_idx)
            .expect("Unexpected index out-of-bounds.")
            .try_into()
            .expect("Attr names must be strings.");
        let value = js_map_array
            .get_index(scope, value_idx)
            .expect("Unexpected index out-of-bounds.");
        map.insert(
            key.to_rust_string_lossy(scope),
            js_value_to_nix(scope, nixrt, &value)?,
        );
    }
    Ok(Value::AttrSet(map))
}

fn new_script_origin<'s>(
    scope: &mut v8::HandleScope<'s>,
    resource_name: &str,
    source_map_url: &str,
) -> v8::ScriptOrigin<'s> {
    let resource_name_v8_str = v8::String::new(scope, resource_name).unwrap();
    let resource_line_offset = 0;
    let resource_column_offset = 0;
    let resource_is_shared_cross_origin = true;
    let script_id = 123;
    let source_map_url = v8::String::new(scope, source_map_url).unwrap();
    let resource_is_opaque = false;
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

fn to_v8_source<'a>(
    scope: &mut v8::HandleScope,
    js_code: &str,
    source_path: &str,
) -> v8::script_compiler::Source {
    let code = v8::String::new(scope, js_code).unwrap();
    let origin = new_script_origin(scope, source_path, &format!("file://{source_path}.map"));
    v8::script_compiler::Source::new(code, Some(&origin))
}

fn resolve_module_callback<'a>(
    context: v8::Local<'a, v8::Context>,
    specifier: v8::Local<'a, v8::String>,
    _import_assertions: v8::Local<'a, v8::FixedArray>,
    _referrer: v8::Local<'a, v8::Module>,
) -> Option<v8::Local<'a, v8::Module>> {
    let scope = &mut unsafe { v8::CallbackScope::new(context) };
    let module_path = specifier.to_rust_string_lossy(scope);
    let module_source_str = std::fs::read_to_string(&module_path).unwrap();
    let module_source_v8 = to_v8_source(scope, &module_source_str, &module_path);
    v8::script_compiler::compile_module(scope, module_source_v8)
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
    fn test_eval_paren() {
        assert_eq!(eval_ok("(1 + 2) * 3"), Value::Int(9));
    }

    #[test]
    fn test_eval_string_literal() {
        assert_eq!(eval_ok(r#""Hello!""#), Value::Str("Hello!".to_owned()));
    }

    #[test]
    fn test_eval_string_uri() {
        assert_eq!(
            eval_ok("http://foo.bat/moo"),
            Value::Str("http://foo.bat/moo".to_owned())
        );
    }

    #[test]
    fn test_eval_string_escape_codes() {
        assert_eq!(
            eval_ok(r#""\"\$\n\r\t\\`""#),
            Value::Str("\"$\n\r\t\\`".to_owned())
        );
        assert_eq!(eval_ok("\"a \n b\""), Value::Str("a \n b".to_owned()));
    }

    #[test]
    fn test_eval_indented_string() {
        assert_eq!(
            eval_ok("''\n  Hello\n  World!''"),
            Value::Str("Hello\nWorld!".to_owned())
        );
        assert_eq!(
            eval_ok("''\n  a\n b\n   c''"),
            Value::Str(" a\nb\n  c".to_owned())
        );
        assert_eq!(
            eval_ok("''''$'''$${}''\\n''\\t''\\r''\\\\''"),
            Value::Str("$''$${}\n\t\r\\".to_owned())
        );
    }

    #[test]
    fn test_eval_string_concat_op() {
        assert_eq!(eval_ok(r#""Hell" + "o!""#), Value::Str("Hello!".to_owned()));
    }

    #[test]
    fn test_eval_string_multiplication_err() {
        assert!(evaluate(r#""b" * "a""#).is_err());
    }

    #[test]
    fn test_eval_string_interpolation() {
        assert_eq!(eval_ok(r#""${"A"}""#), Value::Str("A".to_owned()));
        assert!(evaluate(r#""${1}""#).is_err());
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

    #[test]
    fn test_eval_bool_and_non_bool_err() {
        assert!(evaluate("true && 1").is_err());
        assert!(evaluate("1 && true").is_err());
        assert!(evaluate("1 || true").is_err());
        assert!(evaluate("false || 1").is_err());
        assert!(evaluate("1 -> true").is_err());
        assert!(evaluate("true -> 1").is_err());
        assert!(evaluate("!1").is_err());
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
    }

    #[test]
    fn test_eval_list_concat() {
        assert_eq!(
            eval_ok("[1] ++ [2]"),
            Value::List(vec![Value::Int(1), Value::Int(2)])
        );
    }

    #[test]
    fn test_eval_comparison() {
        assert_eq!(eval_ok("1 < 2"), Value::Bool(true));
        assert_eq!(eval_ok("2 < 2"), Value::Bool(false));

        assert_eq!(eval_ok("2 <= 2"), Value::Bool(true));
        assert_eq!(eval_ok("3 <= 2"), Value::Bool(false));

        assert_eq!(eval_ok("2 > 2"), Value::Bool(false));
        assert_eq!(eval_ok("3 > 2"), Value::Bool(true));

        assert_eq!(eval_ok("1 >= 2"), Value::Bool(false));
        assert_eq!(eval_ok("2 >= 2"), Value::Bool(true));
    }

    #[test]
    fn test_eval_eq() {
        assert_eq!(eval_ok("1 == 1"), Value::Bool(true));
        assert_eq!(eval_ok("1 != 1"), Value::Bool(false));
    }

    #[test]
    fn test_eval_attrset_literal() {
        assert_eq!(eval_ok("{}"), Value::AttrSet(HashMap::new()));
        assert_eq!(
            eval_ok("{a = 1;}"),
            Value::AttrSet(HashMap::from([("a".to_owned(), Value::Int(1))]))
        );
        assert_eq!(
            eval_ok("{a.b = 1;}"),
            Value::AttrSet(HashMap::from([(
                "a".to_owned(),
                Value::AttrSet(HashMap::from([("b".to_owned(), Value::Int(1))])),
            )]))
        );
    }

    #[test]
    fn test_eval_attrset_interpolated_attrs() {
        assert_eq!(eval_ok(r#"{${"a"} = 1;}.a"#), Value::Int(1));
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
}
