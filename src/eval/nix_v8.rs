use std::collections::HashSet;
use std::env::current_dir;
use std::path::Path;
use std::{collections::HashMap, sync::Once};

use rnix::{ast, SyntaxKind};
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

    if module
        .instantiate_module(scope, resolve_module_callback)
        .is_none()
    {
        todo!("Instantiation failure.")
    }
    if module.evaluate(scope).is_none() {
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
    let mut out_src = format!("import n from '{nixrt_js_module}';\n");
    out_src += "export const __nixrt = n;\n";
    out_src += "export const __nixValue = (ctx) => ";
    emit_expr(&root_expr, &mut out_src)?;
    out_src += ";\n";
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
        ast::Expr::LetIn(let_in) => emit_let_in(let_in, out_src),
        ast::Expr::List(list) => emit_list(list, out_src),
        ast::Expr::Literal(literal) => emit_literal(literal, out_src),
        ast::Expr::Paren(paren) => emit_paren(paren, out_src),
        ast::Expr::Path(path) => emit_path(path, out_src),
        ast::Expr::Select(select) => emit_select_expr(select, out_src),
        ast::Expr::Str(string) => emit_string_expr(string, out_src),
        ast::Expr::UnaryOp(unary_op) => emit_unary_op(unary_op, out_src),
        ast::Expr::With(with) => emit_with(with, out_src),
        _ => panic!("emit_expr: not implemented: {:?}", nix_ast),
    }
}

fn emit_apply(apply: &ast::Apply, out_src: &mut String) -> Result<(), String> {
    emit_expr(
        &apply
            .lambda()
            .expect("Unexpected lambda application without the lambda."),
        out_src,
    )?;
    out_src.push_str(".apply(");
    emit_expr(
        &apply
            .argument()
            .expect("Unexpected lambda application without arguments."),
        out_src,
    )?;
    out_src.push(')');
    Ok(())
}

fn emit_attrset(attrset: &ast::AttrSet, out_src: &mut String) -> Result<(), String> {
    emit_has_entry(attrset, attrset.rec_token().is_some(), out_src)
}

fn emit_has_entry(
    has_entry: &impl ast::HasEntry,
    is_recursive: bool,
    out_src: &mut String,
) -> Result<(), String> {
    *out_src += "n.";
    *out_src += if is_recursive {
        "recAttrset"
    } else {
        "attrset"
    };
    *out_src += "(ctx,(ctx) => [";
    for attrpath_value in has_entry.attrpath_values() {
        out_src.push('[');
        let attrpath = attrpath_value.attrpath().expect("Not implemented");
        let value = &attrpath_value.value().expect("Not implemented");
        emit_attrpath(&attrpath, out_src)?;
        *out_src += ",new n.Lazy(ctx,(ctx) => ";
        emit_expr(value, out_src)?;
        *out_src += ")],";
    }
    *out_src += "])";
    Ok(())
}

fn emit_attrpath(attrpath: &ast::Attrpath, out_src: &mut String) -> Result<(), String> {
    *out_src += "[";
    for attr in attrpath.attrs() {
        out_src.push_str("new n.Lazy(ctx,(ctx) =>");
        match attr {
            ast::Attr::Ident(ident) => {
                emit_nix_string(ident.ident_token().expect("Missing token.").text(), out_src)
            }
            ast::Attr::Str(str_expression) => emit_string_expr(&str_expression, out_src)?,
            ast::Attr::Dynamic(expr) => {
                emit_expr(&expr.expr().expect("Expected an expression."), out_src)?
            }
        }
        out_src.push_str("),");
    }
    *out_src += "]";
    Ok(())
}

fn emit_nix_string(string: &str, out_src: &mut String) {
    *out_src += "new n.NixString(\"";
    *out_src += string;
    *out_src += "\")";
}

fn emit_bin_op(bin_op: &ast::BinOp, out_src: &mut String) -> Result<(), String> {
    let operator = bin_op.operator().expect("Not implemented");
    let lhs = &bin_op.lhs().expect("Not implemented");
    let rhs = &bin_op.rhs().expect("Not implemented");
    match operator {
        // Arithmetic
        // ast::BinOpKind::Add => emit_add_bin_op(lhs, rhs, out_src)?,
        ast::BinOpKind::Add => emit_nixrt_bin_op(lhs, rhs, "add", out_src)?,
        ast::BinOpKind::Div => emit_nixrt_bin_op(lhs, rhs, "div", out_src)?,
        ast::BinOpKind::Mul => emit_nixrt_bin_op(lhs, rhs, "mul", out_src)?,
        ast::BinOpKind::Sub => emit_nixrt_bin_op(lhs, rhs, "sub", out_src)?,

        // Attrset
        ast::BinOpKind::Update => emit_nixrt_bin_op(lhs, rhs, "update", out_src)?,

        // Boolean
        ast::BinOpKind::And => emit_nixrt_bin_op(lhs, rhs, "and", out_src)?,
        ast::BinOpKind::Implication => emit_nixrt_bin_op(lhs, rhs, "implication", out_src)?,
        ast::BinOpKind::Or => emit_nixrt_bin_op(lhs, rhs, "or", out_src)?,

        // Comparison
        ast::BinOpKind::Equal => emit_nixrt_bin_op(lhs, rhs, "eq", out_src)?,
        ast::BinOpKind::Less => emit_nixrt_bin_op(lhs, rhs, "less", out_src)?,
        ast::BinOpKind::LessOrEq => emit_nixrt_bin_op(lhs, rhs, "lessEq", out_src)?,
        ast::BinOpKind::More => emit_nixrt_bin_op(lhs, rhs, "more", out_src)?,
        ast::BinOpKind::MoreOrEq => emit_nixrt_bin_op(lhs, rhs, "moreEq", out_src)?,
        ast::BinOpKind::NotEqual => emit_nixrt_bin_op(lhs, rhs, "neq", out_src)?,

        // List
        ast::BinOpKind::Concat => emit_nixrt_bin_op(lhs, rhs, "concat", out_src)?,
    }
    Ok(())
}

fn emit_nixrt_bin_op(
    lhs: &ast::Expr,
    rhs: &ast::Expr,
    nixrt_function: &str,
    out_src: &mut String,
) -> Result<(), String> {
    emit_expr(lhs, out_src)?;
    out_src.push('.');
    *out_src += nixrt_function;
    out_src.push('(');
    emit_expr(rhs, out_src)?;
    out_src.push(')');
    Ok(())
}

fn emit_ident(ident: &ast::Ident, out_src: &mut String) -> Result<(), String> {
    let token = ident.ident_token().expect("Unexpected ident without name.");
    let token_text = token.text();
    match token_text {
        "true" => out_src.push_str("n.TRUE"),
        "false" => out_src.push_str("n.FALSE"),
        "null" => out_src.push_str("n.NULL"),
        _ => {
            out_src.push_str("ctx.lookup(\"");
            js_string_escape_into(token_text, out_src);
            out_src.push_str("\")");
        }
    }
    Ok(())
}

fn emit_has_attr(has_attr: &ast::HasAttr, out_src: &mut String) -> Result<(), String> {
    emit_expr(&has_attr.expr().expect("Unreachable"), out_src)?;
    *out_src += ".has(";
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
    emit_expr(&condition, out_src)?;
    *out_src += ".asBoolean() ? (";
    emit_expr(&body, out_src)?;
    *out_src += ") : (";
    emit_expr(&else_body, out_src)?;
    *out_src += ")";
    Ok(())
}

fn emit_lambda(lambda: &ast::Lambda, out_src: &mut String) -> Result<(), String> {
    let param = lambda
        .param()
        .expect("Unexpected lambda without parameters.");
    let body = lambda
        .body()
        .expect("Unexpected lambda without parameters.");
    match param {
        ast::Param::IdentParam(ident_param) => emit_param_lambda(&ident_param, &body, out_src),
        ast::Param::Pattern(pattern) => emit_pattern_lambda(&pattern, &body, out_src),
    }
}

fn emit_param_lambda(
    ident_param: &ast::IdentParam,
    body: &ast::Expr,
    out_src: &mut String,
) -> Result<(), String> {
    *out_src += "n.paramLambda(ctx,";
    emit_ident_as_js_string(
        &ident_param
            .ident()
            .expect("Unexpected missing lambda parameter identifier."),
        out_src,
    );
    *out_src += ",(ctx) => ";
    emit_expr(body, out_src)?;
    *out_src += ")";
    Ok(())
}

fn emit_pattern_lambda(
    pattern: &ast::Pattern,
    body: &ast::Expr,
    out_src: &mut String,
) -> Result<(), String> {
    let mut formal_arg_names = HashSet::new();
    *out_src += "n.patternLambda(ctx,";
    if let Some(indent) = pattern.pat_bind().and_then(|pat_bind| pat_bind.ident()) {
        formal_arg_names.insert(indent.to_string());
        emit_ident_as_js_string(&indent, out_src);
    } else {
        *out_src += "undefined";
    }
    *out_src += ",[";
    for pattern_entry in pattern.pat_entries() {
        let ident = pattern_entry.ident().ok_or_else(|| {
            "Unsupported lambda pattern parameter without an identifier.".to_owned()
        })?;
        if !formal_arg_names.insert(ident.to_string()) {
            return Err(format!("duplicate formal function argument '{}'.", ident));
        }
        *out_src += "[";
        emit_ident_as_js_string(&ident, out_src);
        *out_src += ",";
        if let Some(default_value) = pattern_entry.default() {
            emit_expr(&default_value, out_src)?;
        }
        *out_src += "],";
    }
    *out_src += "],(ctx) => ";
    emit_expr(body, out_src)?;
    *out_src += ")";
    Ok(())
}

fn emit_ident_as_js_string(ident: &ast::Ident, out_src: &mut String) {
    out_src.push('"');
    js_string_escape_into(&ident.to_string(), out_src);
    out_src.push('"');
}

fn emit_let_in(let_in: &ast::LetIn, out_src: &mut String) -> Result<(), String> {
    *out_src += "n.letIn(ctx,";
    emit_has_entry(let_in, true, out_src)?;
    *out_src += ",(ctx) => ";
    emit_expr(
        &let_in
            .body()
            .expect("Unexpected let-in expression without a body."),
        out_src,
    )?;
    *out_src += ")";
    Ok(())
}

fn emit_list(list: &ast::List, out_src: &mut String) -> Result<(), String> {
    *out_src += "new n.NixList([";
    for element in list.items() {
        out_src.push_str("new n.Lazy(ctx,(ctx) => ");
        emit_expr(&element, out_src)?;
        out_src.push_str("),");
    }
    *out_src += "])";
    Ok(())
}

fn emit_literal(literal: &ast::Literal, out_src: &mut String) -> Result<(), String> {
    let token = literal.syntax().first_token().expect("Not implemented");
    match token.kind() {
        SyntaxKind::TOKEN_INTEGER => {
            out_src.push_str("new n.NixInt(");
            out_src.push_str(token.text());
            out_src.push_str("n)");
        }
        SyntaxKind::TOKEN_FLOAT => {
            out_src.push_str("new n.NixFloat(");
            out_src.push_str(token.text());
            out_src.push(')');
        }
        SyntaxKind::TOKEN_URI => emit_nix_string(token.text(), out_src),
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
    *out_src += "n.toPath(ctx,`";
    js_string_escape_into(&path.to_string(), out_src);
    *out_src += "`)";
    Ok(())
}

fn emit_select_expr(select: &ast::Select, out_src: &mut String) -> Result<(), String> {
    emit_expr(&select.expr().expect("Unreachable"), out_src)?;
    *out_src += ".select(";
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
    *out_src += "new n.NixString(`";
    for string_part in string.normalized_parts() {
        match string_part {
            ast::InterpolPart::Literal(literal) => {
                js_string_escape_into(&literal, out_src);
            }
            ast::InterpolPart::Interpolation(interpolation_body) => {
                *out_src += "${";
                emit_expr(
                    &interpolation_body
                        .expr()
                        .expect("String interpolation body missing."),
                    out_src,
                )?;
                *out_src += ".asString()}";
            }
        }
    }
    *out_src += "`)";
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
        ast::UnaryOpKind::Invert => emit_nixrt_unary_op(operand, "invert", out_src),
        ast::UnaryOpKind::Negate => emit_nixrt_unary_op(operand, "neg", out_src),
    }
}

fn emit_nixrt_unary_op(
    operand: &ast::Expr,
    nixrt_function: &str,
    out_src: &mut String,
) -> Result<(), String> {
    emit_expr(operand, out_src)?;
    out_src.push('.');
    *out_src += nixrt_function;
    *out_src += "()";
    Ok(())
}

fn emit_with(with: &ast::With, out_src: &mut String) -> Result<(), String> {
    *out_src += "n.withExpr(ctx,";
    emit_expr(
        &with
            .namespace()
            .ok_or_else(|| "Unexpected 'with' expression without a namespace.".to_string())?,
        out_src,
    )?;
    *out_src += ",(ctx) => ";
    emit_expr(
        &with
            .body()
            .ok_or_else(|| "Unexpected 'with' expression without a body.".to_string())?,
        out_src,
    )?;
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

    let nixrt_attr = v8::String::new(scope, "__nixrt").unwrap();
    let nixrt: v8::Local<v8::Value> = namespace_obj.get(scope, nixrt_attr.into()).unwrap();

    let eval_ctx = create_eval_ctx(
        scope,
        &nixrt,
        &current_dir().map_err(|err| {
            format!("Failed to determine the current working directory. Error: {err}")
        })?,
    )?;

    let nix_value = call_js_function(scope, &nix_value, &[eval_ctx.into()])?;

    let to_strict_fn: v8::Local<v8::Function> = try_get_js_object_key(scope, &nixrt, "toStrict")?
        .expect("Could not find the function `toStrict` in `nixrt`.")
        .try_into()
        .expect("`n.toStrict` is not a function.");
    let strict_nix_value = call_js_function(scope, &to_strict_fn, &[nix_value])?;

    js_value_to_nix(scope, &nixrt, &strict_nix_value)
}

fn call_js_function<'s>(
    scope: &mut v8::ContextScope<'_, v8::HandleScope<'s>>,
    js_function: &v8::Local<v8::Function>,
    args: &[v8::Local<v8::Value>],
) -> Result<v8::Local<'s, v8::Value>, String> {
    let scope = &mut v8::TryCatch::new(scope);
    let recv = v8::undefined(scope).into();
    let Some(strict_nix_value) = js_function.call(scope, recv, args) else {
        // TODO: Again, the stack trace needs to be source-mapped. See TODO above.
        let err_msg = scope
            .stack_trace()
            .map_or(
                "Unknown evaluation error.".to_owned(),
                |stack| stack.to_rust_string_lossy(scope),
            );
        return Err(err_msg);
    };
    Ok(strict_nix_value)
}

fn create_eval_ctx<'s>(
    scope: &mut v8::HandleScope<'s>,
    nixrt: &v8::Local<v8::Value>,
    script_path: &Path,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let eval_ctx_type = get_nixrt_type(scope, nixrt, "EvalCtx")?;
    let eval_ctx_constructor: v8::Local<v8::Function> = eval_ctx_type
        .try_into()
        .expect("Could not get the constructor of the evaluation context class.");

    let real_path = script_path
        .canonicalize()
        .map_err(|err| format!("Failed to resolve the script path. Error: {err}."))?;
    let script_dir = real_path
        .parent()
        .ok_or_else(|| format!("Failed to determine the directory of path {real_path:?}."))?;
    let script_dir_str = real_path
        .to_str()
        .ok_or_else(|| format!("Failed to converft the path {script_dir:?} to a string."))?;
    let js_script_dir_path =
        v8::String::new(scope, script_dir_str).expect("Unexpected internal error.");

    Ok(eval_ctx_constructor
        .new_instance(scope, &[js_script_dir_path.into()])
        .expect("Could not construct the global evaluation context."))
}

fn js_value_to_nix(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> EvalResult {
    if js_value.is_function() {
        return Ok(Value::Lambda);
    }
    if let Some(value) = from_js_attrset(scope, nixrt, js_value)? {
        return Ok(value);
    }
    if let Some(value) = from_js_string(scope, nixrt, js_value)? {
        return Ok(value);
    }
    if let Some(value) = from_js_lazy(scope, nixrt, js_value)? {
        return Ok(value);
    }
    if let Some(value) = from_js_int(scope, nixrt, js_value)? {
        return Ok(value);
    }
    if let Some(value) = from_js_bool(scope, nixrt, js_value)? {
        return Ok(value);
    }
    if let Some(value) = from_js_float(scope, nixrt, js_value)? {
        return Ok(value);
    }
    if let Some(value) = from_js_list(scope, nixrt, js_value)? {
        return Ok(value);
    }
    if let Some(value) = from_js_path(scope, nixrt, js_value)? {
        return Ok(value);
    }
    if let Some(value) = from_js_lambda(scope, nixrt, js_value)? {
        return Ok(value);
    }
    todo!(
        "js_value_to_nix: {:?}",
        js_value.to_rust_string_lossy(scope),
    )
}

fn from_js_int(
    scope: &mut v8::HandleScope<'_>,
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

fn from_js_string(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> Result<Option<Value>, String> {
    if is_nixrt_type(scope, nixrt, js_value, "NixString")? {
        let Some(value) = try_get_js_object_key(scope, js_value, "value")? else {
            return Ok(None);
        };
        let value_js_string: v8::Local<v8::String> = value.try_into().map_err(|err| {
            format!("Expected a string value. Internal conversion error: {err:?}")
        })?;
        return Ok(Some(Value::Str(
            value_js_string.to_rust_string_lossy(scope),
        )));
    }
    Ok(None)
}

fn from_js_lazy(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> Result<Option<Value>, String> {
    if is_nixrt_type(scope, nixrt, js_value, "Lazy")? {
        let to_strict = try_get_js_object_key(scope, js_value, "toStrict")?.ok_or_else(|| {
            "Internal error: could not find the `toStrict` method on the Lazy object.".to_string()
        })?;
        let to_strict_method: v8::Local<v8::Function> = to_strict.try_into().map_err(|err| {
            format!(
                "Expected `toStrict` to be a method on the Lazy object. Internal conversion error: {err:?}"
            )
        })?;
        let strict_value = to_strict_method
            .call(scope, *js_value, &[])
            .ok_or_else(|| "Could not convert the lazy value to strict.".to_string())?;
        return Ok(Some(js_value_to_nix(scope, nixrt, &strict_value)?));
    }
    Ok(None)
}

fn from_js_bool(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> Result<Option<Value>, String> {
    if is_nixrt_type(scope, nixrt, js_value, "NixBool")? {
        let value = try_get_js_object_key(scope, js_value, "value")?.ok_or_else(|| {
            "Internal error: could not find the `value` property on the NixBool object.".to_string()
        })?;
        let value_as_bool: v8::Local<v8::Boolean> = value.try_into().map_err(|err| {
            format!(
                "Expected `value` to be a boolean on the NixBool object. Internal conversion error: {err:?}"
            )
        })?;
        return Ok(Some(Value::Bool(value_as_bool.boolean_value(scope))));
    }
    Ok(None)
}

fn from_js_float(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> Result<Option<Value>, String> {
    if is_nixrt_type(scope, nixrt, js_value, "NixFloat")? {
        let value = try_get_js_object_key(scope, js_value, "value")?.ok_or_else(|| {
            "Internal error: could not find the `value` property on the NixFloat object."
                .to_string()
        })?;
        let value_as_number: v8::Local<v8::Number> = value.try_into().map_err(|err| {
            format!(
                "Expected `value` to be a number on the NixFloat object. Internal conversion error: {err:?}"
            )
        })?;
        return Ok(Some(Value::Float(
            value_as_number.number_value(scope).ok_or_else(|| {
                "Could not convert the JavaScript number to a floating point number.".to_string()
            })?,
        )));
    }
    Ok(None)
}

fn from_js_attrset(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> Result<Option<Value>, String> {
    if is_nixrt_type(scope, nixrt, js_value, "Attrset")? {
        let underlying_map_value = try_get_js_object_key(scope, js_value, "underlyingMap")?
            .ok_or_else(|| {
                "Internal error: could not find the `underlyingMap` method on the Attrset object."
                    .to_string()
            })?;
        let underlying_map_function: v8::Local<v8::Function> = underlying_map_value.try_into().map_err(|err| {
            format!(
                "Expected `underlyingMap` to be a method on the Attrset object. Internal conversion error: {err:?}"
            )
        })?;
        let underlying_map: v8::Local<v8::Map> = underlying_map_function
            .call(scope, *js_value, &[])
            .ok_or_else(|| "Could not get the underlying map of the Attrset.".to_string())?
            .try_into()
            .map_err(|err| {
                format!(
                    "Expected `underlyingMap` to return a Map. Internal conversion error: {err:?}"
                )
            })?;
        return Ok(Some(js_map_as_attrset(scope, nixrt, &underlying_map)?));
    }
    Ok(None)
}

fn from_js_list(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> Result<Option<Value>, String> {
    if is_nixrt_type(scope, nixrt, js_value, "NixList")? {
        let value = try_get_js_object_key(scope, js_value, "values")?.ok_or_else(|| {
            "Internal error: could not find the `values` property on the NixList object."
                .to_string()
        })?;
        let value_as_array: v8::Local<v8::Array> = value.try_into().map_err(|err| {
            format!(
                "Expected `values` to be an array in the NixList object. Internal conversion error: {err:?}"
            )
        })?;
        return Ok(Some(js_value_as_nix_array(scope, nixrt, &value_as_array)?));
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

fn is_nixrt_type(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
    type_name: &str,
) -> Result<bool, String> {
    let nixrt_type = get_nixrt_type(scope, nixrt, type_name)?;
    js_value.instance_of(scope, nixrt_type).ok_or_else(|| {
        format!(
            "Failed to check whether value '{}' is '{type_name}'.",
            js_value.to_rust_string_lossy(scope)
        )
    })
}

fn js_value_as_nix_array(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_array: &v8::Local<v8::Array>,
) -> EvalResult {
    let length = js_array.length();
    let mut rust_array = Vec::with_capacity(length as usize);
    for idx in 0..length {
        let js_element = js_array.get_index(scope, idx).unwrap();
        match js_value_to_nix(scope, nixrt, &js_element) {
            Ok(value) => rust_array.push(value),
            err => return err,
        }
    }
    Ok(Value::List(rust_array))
}

fn from_js_path(
    scope: &mut v8::HandleScope<'_>,
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

fn from_js_lambda(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> Result<Option<Value>, String> {
    if !is_nixrt_type(scope, nixrt, js_value, "Lambda")? {
        return Ok(None);
    }
    Ok(Some(Value::Lambda))
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

fn js_map_as_attrset(
    scope: &mut v8::HandleScope<'_>,
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

fn to_v8_source(
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
    }

    #[test]
    fn test_eval_attrset_literal_nesting() {
        let expected_attrset = Value::AttrSet(HashMap::from([(
            "a".to_owned(),
            Value::AttrSet(HashMap::from([("b".to_owned(), Value::Int(1))])),
        )]));
        assert_eq!(eval_ok("{a.b = 1;}"), expected_attrset);
        assert_eq!(eval_ok("{ a = {}; a.b = 1; }"), expected_attrset);
        assert!(evaluate("{ a = 1; a.b = 1; }").is_err());
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
    fn test_eval_builtin_head() {
        assert_eq!(eval_ok("builtins.head [ 1 2 ]"), Value::Int(1));
    }

    #[test]
    fn test_eval_lists_are_lazy() {
        assert_eq!(eval_ok("builtins.head [ 1 (1 / 0) ]"), Value::Int(1));
    }
}
