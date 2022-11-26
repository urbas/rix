use std::collections::HashMap;

use rnix::ast::{
    AttrSet, Attrpath, BinOp, BinOpKind, Expr, HasEntry, Ident, List, Literal, Str, UnaryOp,
    UnaryOpKind,
};
use rnix::{NodeOrToken, SyntaxKind::*, SyntaxToken};
use rowan::ast::AstNode;

#[derive(Debug, PartialEq)]
pub enum Value {
    AttrSet(HashMap<String, Value>),
    Bool(bool),
    Float(f64),
    Int(i64),
    List(Vec<Value>),
    Str(String),
}

pub fn eval_str(nix_expr: &str) -> Value {
    let root = rnix::Root::parse(nix_expr).tree();
    let root_expr = root.expr().expect("Not implemented");
    eval_expr(&root_expr)
}

pub fn eval_expr(expr: &Expr) -> Value {
    match expr {
        Expr::AttrSet(attrset) => eval_attrset(&attrset),
        Expr::BinOp(bin_op) => eval_bin_op(&bin_op),
        Expr::Ident(ident) => eval_ident(&ident),
        Expr::List(list) => eval_list(list),
        Expr::Literal(literal) => eval_literal(literal),
        Expr::Paren(paren) => eval_expr(&paren.expr().expect("Not implemented")),
        Expr::Str(string) => eval_string_expr(string),
        Expr::UnaryOp(unary_op) => eval_unary_op(unary_op),
        _ => panic!("Not implemented: {:?}", expr),
    }
}

fn eval_attrset(attrset: &AttrSet) -> Value {
    let mut hash_map = HashMap::new();
    for attrpath_value in attrset.attrpath_values() {
        let attrpath = attrpath_value.attrpath().expect("Not implemented");
        let value = eval_expr(&attrpath_value.value().expect("Not implemented"));
        hash_map.insert(eval_attrpath(&attrpath), value);
    }
    Value::AttrSet(hash_map)
}

fn eval_attrpath(attrpath: &Attrpath) -> String {
    let Some(attr) = attrpath.attrs().next() else {
        todo!()
    };
    let Ok(ident) = Ident::try_from(attr) else {
        todo!()
    };
    ident
        .ident_token()
        .expect("Not implemented")
        .text()
        .to_owned()
}

fn eval_bin_op(bin_op: &BinOp) -> Value {
    let operator = bin_op.operator().expect("Not implemented");
    let lhs = &bin_op.lhs().expect("Not implemented");
    let rhs = &bin_op.rhs().expect("Not implemented");
    match operator {
        BinOpKind::Add => eval_arithmetic_bin_op(&lhs, &rhs, |x, y| x + y, |x, y| x + y),
        BinOpKind::Sub => eval_arithmetic_bin_op(&lhs, &rhs, |x, y| x - y, |x, y| x - y),
        BinOpKind::Mul => eval_arithmetic_bin_op(&lhs, &rhs, |x, y| x * y, |x, y| x * y),
        BinOpKind::Div => eval_arithmetic_bin_op(&lhs, &rhs, |x, y| x / y, |x, y| x / y),
        BinOpKind::Or => eval_or_bin_op(&lhs, &rhs),
        BinOpKind::And => eval_and_bin_op(&lhs, &rhs),
        _ => panic!("Not implemented"),
    }
}

fn eval_arithmetic_bin_op(
    lhs: &Expr,
    rhs: &Expr,
    float_operator: fn(f64, f64) -> f64,
    int_operator: fn(i64, i64) -> i64,
) -> Value {
    match (eval_expr(lhs), eval_expr(rhs)) {
        (Value::Int(lhs_int), Value::Int(rhs_int)) => Value::Int(int_operator(lhs_int, rhs_int)),
        (Value::Int(lhs_int), Value::Float(rhs_float)) => {
            Value::Float(float_operator(lhs_int as f64, rhs_float))
        }
        (Value::Float(lhs_float), Value::Int(rhs_int)) => {
            Value::Float(float_operator(lhs_float, rhs_int as f64))
        }
        (Value::Float(lhs_float), Value::Float(rhs_float)) => {
            Value::Float(float_operator(lhs_float, rhs_float))
        }
        _ => panic!("Not supported"),
    }
}

fn eval_or_bin_op(lhs: &Expr, rhs: &Expr) -> Value {
    Value::Bool(eval_bool(lhs) || eval_bool(rhs))
}

fn eval_and_bin_op(lhs: &Expr, rhs: &Expr) -> Value {
    Value::Bool(eval_bool(lhs) && eval_bool(rhs))
}

fn eval_bool(expr: &Expr) -> bool {
    match eval_expr(expr) {
        Value::Bool(value) => value,
        _ => todo!(),
    }
}

fn eval_ident(ident: &Ident) -> Value {
    let token = ident.ident_token().expect("Not implemented");
    match token.kind() {
        TOKEN_IDENT => eval_ident_token(&token),
        _ => todo!(),
    }
}

fn eval_ident_token(token: &SyntaxToken) -> Value {
    match token.text() {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ => todo!(),
    }
}

fn eval_list(list: &List) -> Value {
    Value::List(list.items().map(|item| eval_expr(&item)).collect())
}

fn eval_literal(literal: &Literal) -> Value {
    let token = literal.syntax().first_token().expect("Not implemented");
    match token.kind() {
        TOKEN_INTEGER => Value::Int(token.text().parse::<i64>().expect("Not implemented")),
        TOKEN_FLOAT => Value::Float(token.text().parse::<f64>().expect("Not implemented")),
        _ => todo!(),
    }
}

fn eval_unary_op(unary_op: &UnaryOp) -> Value {
    let operator = unary_op.operator().expect("Not implemented");
    let operand = eval_expr(&unary_op.expr().expect("Not implemented"));
    match operator {
        UnaryOpKind::Invert => eval_invert_unary_op(&operand),
        _ => todo!(),
    }
}

fn eval_invert_unary_op(operand: &Value) -> Value {
    let Value::Bool(operand_value) = operand else {
        todo!()
    };
    Value::Bool(!operand_value)
}

fn eval_string_expr(string: &Str) -> Value {
    let mut tokens = string.syntax().children_with_tokens();
    if let None = tokens.next() {
        todo!()
    };
    let Some(NodeOrToken:: Token(string_content)) = tokens.next() else {
        todo!()
    };
    Value::Str(string_content.text().to_owned())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_eval_int_arithmetic() {
        assert_eq!(eval_str("1 + 2"), Value::Int(3));
        assert_eq!(eval_str("1 - 2"), Value::Int(-1));
        assert_eq!(eval_str("1 * 2"), Value::Int(2));
        assert_eq!(eval_str("1 / 2"), Value::Int(0));
    }

    #[test]
    fn test_eval_float_arithmetic() {
        assert_eq!(eval_str("1 / 2.0"), Value::Float(0.5));
        assert_eq!(eval_str("1.0 / 2"), Value::Float(0.5));
        assert_eq!(eval_str("1.0 / 2.0"), Value::Float(0.5));
    }

    #[test]
    fn test_eval_bool_expr() {
        assert_eq!(eval_str("true"), Value::Bool(true));
        assert_eq!(eval_str("false"), Value::Bool(false));
        assert_eq!(eval_str("!false"), Value::Bool(true));
        assert_eq!(eval_str("false || true"), Value::Bool(true));
        assert_eq!(eval_str("true && true"), Value::Bool(true));
        assert_eq!(eval_str("false || true && false"), Value::Bool(false));
        assert_eq!(eval_str("false && true || false"), Value::Bool(false));
    }

    #[test]
    fn test_eval_paren() {
        assert_eq!(eval_str("(1 + 2) + 3"), Value::Int(6));
    }

    #[test]
    fn test_eval_string_expr() {
        assert_eq!(eval_str("\"Hello!\""), Value::Str("Hello!".to_owned()));
    }

    #[test]
    fn test_eval_list_expr() {
        assert_eq!(
            eval_str("[ 42 true \"answer\" ]"),
            Value::List(vec![
                Value::Int(42),
                Value::Bool(true),
                Value::Str("answer".to_owned())
            ])
        );
    }

    #[test]
    fn test_eval_attrset_expr() {
        assert_eq!(
            eval_str("{a = 42;}"),
            Value::AttrSet(HashMap::from([("a".to_owned(), Value::Int(42))]))
        );
    }
}
