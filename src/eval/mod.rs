use rnix::ast::{BinOp, BinOpKind, Expr, Literal};
use rowan::ast::AstNode;

// TODO: add support for other types
type Value = i64;

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
    let lhs = bin_op.lhs().expect("Not implemented");
    let rhs = bin_op.rhs().expect("Not implemented");
    let lhs_value = eval_expr(&lhs);
    let rhs_value = eval_expr(&rhs);
    match bin_op.operator() {
        Some(BinOpKind::Add) => lhs_value + rhs_value,
        Some(BinOpKind::Sub) => lhs_value - rhs_value,
        Some(BinOpKind::Mul) => lhs_value * rhs_value,
        Some(BinOpKind::Div) => lhs_value / rhs_value,
        _ => panic!("Not implemented"),
    }
}

fn eval_literal(literal: &Literal) -> Value {
    let token = literal.syntax().first_token().expect("Not implemented");
    token.text().parse::<i64>().expect("Not implemented")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_int_arithmetic() {
        assert_eq!(eval_str("1 + 2"), 3);
        assert_eq!(eval_str("1 - 2"), -1);
        assert_eq!(eval_str("1 * 2"), 2);
        assert_eq!(eval_str("1 / 2"), 0);
    }

    #[test]
    fn test_eval_paren() {
        assert_eq!(eval_str("(1 + 2) + 3"), 6);
    }
}
