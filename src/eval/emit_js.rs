use std::collections::HashSet;

use rnix::{ast, SyntaxKind};
use rowan::ast::AstNode;

pub fn emit_module(nix_expr: &str) -> Result<String, String> {
    let root = rnix::Root::parse(nix_expr).tree();
    let root_expr = root.expr().expect("Not implemented");
    let mut out_src = String::new();
    out_src += "export default (ctx) => ";
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
