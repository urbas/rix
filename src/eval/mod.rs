use rnix::ast::{BinOp, BinOpKind, Expr, Literal};
use rnix::SyntaxKind::*;
use rowan::ast::AstNode;

#[derive(Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
}

pub fn eval_str(nix_expr: &str) -> Value {
    let root = rnix::Root::parse(nix_expr).tree();
    let root_expr = root.expr().expect("Not implemented");
    eval_expr(&root_expr)
}

pub fn eval_expr(expr: &Expr) -> Value {
    match expr {
        Expr::BinOp(bin_op) => eval_bin_op(&bin_op),
        Expr::Literal(literal) => eval_literal(literal),
        Expr::Paren(paren) => eval_expr(&paren.expr().expect("Not implemented")),
        _ => panic!("Not implemented: {:?}", expr),
    }
}

fn eval_bin_op(bin_op: &BinOp) -> Value {
    let operator = bin_op.operator().expect("Not implemented");
    let lhs = bin_op.lhs().expect("Not implemented");
    let rhs = bin_op.rhs().expect("Not implemented");
    match (eval_expr(&lhs), eval_expr(&rhs)) {
        (Value::Int(lhs_int), Value::Int(rhs_int)) => eval_bin_op_int(operator, lhs_int, rhs_int),
        (Value::Int(lhs_int), Value::Float(rhs_float)) => {
            eval_bin_op_float(operator, lhs_int as f64, rhs_float)
        }
        (Value::Float(lhs_float), Value::Int(rhs_int)) => {
            eval_bin_op_float(operator, lhs_float, rhs_int as f64)
        }
        (Value::Float(lhs_float), Value::Float(rhs_float)) => {
            eval_bin_op_float(operator, lhs_float, rhs_float)
        }
    }
}

fn eval_bin_op_int(operator: BinOpKind, lhs: i64, rhs: i64) -> Value {
    match operator {
        BinOpKind::Add => Value::Int(lhs + rhs),
        BinOpKind::Sub => Value::Int(lhs - rhs),
        BinOpKind::Mul => Value::Int(lhs * rhs),
        BinOpKind::Div => Value::Int(lhs / rhs),
        _ => panic!("Not implemented"),
    }
}

fn eval_bin_op_float(operator: BinOpKind, lhs: f64, rhs: f64) -> Value {
    match operator {
        BinOpKind::Add => Value::Float(lhs + rhs),
        BinOpKind::Sub => Value::Float(lhs - rhs),
        BinOpKind::Mul => Value::Float(lhs * rhs),
        BinOpKind::Div => Value::Float(lhs / rhs),
        _ => panic!("Not implemented"),
    }
}

fn eval_literal(literal: &Literal) -> Value {
    let token = literal.syntax().first_token().expect("Not implemented");
    match token.kind() {
        TOKEN_INTEGER => Value::Int(token.text().parse::<i64>().expect("Not implemented")),
        TOKEN_FLOAT => Value::Float(token.text().parse::<f64>().expect("Not implemented")),
        _ => todo!("Not implemented"),
    }
}

#[cfg(test)]
mod tests {
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
    fn test_eval_paren() {
        assert_eq!(eval_str("(1 + 2) + 3"), Value::Int(6));
    }
}
