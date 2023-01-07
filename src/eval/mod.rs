use std::cmp::Ordering;
use std::collections::HashMap;

use rnix::ast::{
    Attr, AttrSet, Attrpath, BinOp, BinOpKind, Expr, HasAttr, HasEntry, Ident, List, Literal,
    Select, Str, UnaryOp, UnaryOpKind,
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

type EvalResult = Result<Value, ()>;

pub fn eval_str(nix_expr: &str) -> EvalResult {
    let root = rnix::Root::parse(nix_expr).tree();
    let root_expr = root.expr().expect("Not implemented");
    eval_expr(&root_expr)
}

pub fn eval_expr(expr: &Expr) -> EvalResult {
    match expr {
        Expr::AttrSet(attrset) => eval_attrset(&attrset),
        Expr::BinOp(bin_op) => eval_bin_op(&bin_op),
        Expr::HasAttr(has_op) => eval_has_op(&has_op),
        Expr::Ident(ident) => eval_ident(&ident),
        Expr::List(list) => eval_list(list),
        Expr::Literal(literal) => eval_literal(literal),
        Expr::Paren(paren) => eval_expr(&paren.expr().expect("Not implemented")),
        Expr::Select(select) => eval_select_op(select),
        Expr::Str(string) => eval_string_expr(string),
        Expr::UnaryOp(unary_op) => eval_unary_op(unary_op),
        _ => panic!("Not implemented: {:?}", expr),
    }
}

fn eval_attrset(attrset: &AttrSet) -> EvalResult {
    let mut hash_map = HashMap::new();
    for attrpath_value in attrset.attrpath_values() {
        let attrpath = attrpath_value.attrpath().expect("Not implemented");
        let value = eval_expr(&attrpath_value.value().expect("Not implemented"))?;
        hash_map.insert(eval_attrpath(&attrpath)?, value);
    }
    Ok(Value::AttrSet(hash_map))
}

fn eval_attrpath(attrpath: &Attrpath) -> Result<String, ()> {
    let Some(attr) = attrpath.attrs().next() else {
        todo!()
    };
    let Ok(ident) = Ident::try_from(attr) else {
        todo!()
    };
    Ok(ident
        .ident_token()
        .expect("Not implemented")
        .text()
        .to_owned())
}

fn eval_bin_op(bin_op: &BinOp) -> EvalResult {
    let operator = bin_op.operator().expect("Not implemented");
    let lhs = &bin_op.lhs().expect("Not implemented");
    let rhs = &bin_op.rhs().expect("Not implemented");
    match operator {
        // Arithmetic
        BinOpKind::Add => eval_add_bin_op(&lhs, &rhs),
        BinOpKind::Sub => eval_arithmetic_bin_op(&lhs, &rhs, |x, y| x - y, |x, y| x - y),
        BinOpKind::Mul => eval_arithmetic_bin_op(&lhs, &rhs, |x, y| x * y, |x, y| x * y),
        BinOpKind::Div => eval_arithmetic_bin_op(&lhs, &rhs, |x, y| x / y, |x, y| x / y),
        // Boolean
        BinOpKind::Or => eval_or_bin_op(&lhs, &rhs),
        BinOpKind::And => eval_and_bin_op(&lhs, &rhs),
        BinOpKind::Implication => eval_implication_bin_op(&lhs, &rhs),
        // Comparison
        BinOpKind::More => eval_gt_bin_op(&lhs, &rhs),
        BinOpKind::MoreOrEq => eval_gt_or_eq_bin_op(&lhs, &rhs),
        BinOpKind::Less => eval_lt_bin_op(&lhs, &rhs),
        BinOpKind::LessOrEq => eval_lt_or_eq_bin_op(&lhs, &rhs),
        BinOpKind::Equal => eval_eq_bin_op(&lhs, &rhs),
        BinOpKind::NotEqual => eval_neq_bin_op(&lhs, &rhs),
        // List
        BinOpKind::Concat => eval_concat_bin_op(&lhs, &rhs),
        // Attrset
        BinOpKind::Update => eval_update_bin_op(&lhs, &rhs),
    }
}

fn eval_add_bin_op(lhs: &Expr, rhs: &Expr) -> EvalResult {
    match eval_expr(lhs)? {
        Value::Int(lhs_int) => match eval_expr(rhs)? {
            Value::Int(rhs_int) => Ok(Value::Int(lhs_int + rhs_int)),
            Value::Float(rhs_float) => Ok(Value::Float(lhs_int as f64 + rhs_float)),
            _ => todo!(),
        },
        Value::Float(lhs_float) => match eval_expr(rhs)? {
            Value::Int(rhs_int) => Ok(Value::Float(lhs_float + rhs_int as f64)),
            Value::Float(rhs_float) => Ok(Value::Float(lhs_float + rhs_float)),
            _ => todo!(),
        },
        Value::Str(mut lhs_str) => {
            let Value::Str(rhs_str) = eval_expr(rhs)? else {
                todo!()
            };
            lhs_str.push_str(&rhs_str);
            Ok(Value::Str(lhs_str))
        }
        _ => todo!(),
    }
}

fn eval_arithmetic_bin_op(
    lhs: &Expr,
    rhs: &Expr,
    float_operator: fn(f64, f64) -> f64,
    int_operator: fn(i64, i64) -> i64,
) -> EvalResult {
    match (eval_expr(lhs)?, eval_expr(rhs)?) {
        (Value::Int(lhs_int), Value::Int(rhs_int)) => {
            Ok(Value::Int(int_operator(lhs_int, rhs_int)))
        }
        (Value::Int(lhs_int), Value::Float(rhs_float)) => {
            Ok(Value::Float(float_operator(lhs_int as f64, rhs_float)))
        }
        (Value::Float(lhs_float), Value::Int(rhs_int)) => {
            Ok(Value::Float(float_operator(lhs_float, rhs_int as f64)))
        }
        (Value::Float(lhs_float), Value::Float(rhs_float)) => {
            Ok(Value::Float(float_operator(lhs_float, rhs_float)))
        }
        _ => Err(()), // TODO: add a better error message that explains nicely what the error is
    }
}

fn eval_or_bin_op(lhs: &Expr, rhs: &Expr) -> EvalResult {
    Ok(Value::Bool(eval_bool(lhs)? || eval_bool(rhs)?))
}

fn eval_and_bin_op(lhs: &Expr, rhs: &Expr) -> EvalResult {
    Ok(Value::Bool(eval_bool(lhs)? && eval_bool(rhs)?))
}

fn eval_implication_bin_op(lhs: &Expr, rhs: &Expr) -> EvalResult {
    Ok(Value::Bool(!eval_bool(lhs)? || eval_bool(rhs)?))
}

fn eval_bool(expr: &Expr) -> Result<bool, ()> {
    match eval_expr(expr)? {
        Value::Bool(value) => Ok(value),
        _ => todo!(),
    }
}

fn eval_gt_bin_op(lhs: &Expr, rhs: &Expr) -> EvalResult {
    match partial_cmp(&eval_expr(lhs)?, &eval_expr(rhs)?)? {
        Ordering::Greater => Ok(Value::Bool(true)),
        _ => Ok(Value::Bool(false)),
    }
}

fn eval_gt_or_eq_bin_op(lhs: &Expr, rhs: &Expr) -> EvalResult {
    match partial_cmp(&eval_expr(lhs)?, &eval_expr(rhs)?)? {
        Ordering::Greater | Ordering::Equal => Ok(Value::Bool(true)),
        _ => Ok(Value::Bool(false)),
    }
}

fn eval_lt_bin_op(lhs: &Expr, rhs: &Expr) -> EvalResult {
    match partial_cmp(&eval_expr(lhs)?, &eval_expr(rhs)?)? {
        Ordering::Less => Ok(Value::Bool(true)),
        _ => Ok(Value::Bool(false)),
    }
}

fn eval_lt_or_eq_bin_op(lhs: &Expr, rhs: &Expr) -> EvalResult {
    match partial_cmp(&eval_expr(lhs)?, &eval_expr(rhs)?)? {
        Ordering::Less | Ordering::Equal => Ok(Value::Bool(true)),
        _ => Ok(Value::Bool(false)),
    }
}

fn eval_eq_bin_op(lhs: &Expr, rhs: &Expr) -> EvalResult {
    match partial_cmp(&eval_expr(lhs)?, &eval_expr(rhs)?)? {
        Ordering::Equal => Ok(Value::Bool(true)),
        _ => Ok(Value::Bool(false)),
    }
}

fn eval_neq_bin_op(lhs: &Expr, rhs: &Expr) -> EvalResult {
    match partial_cmp(&eval_expr(lhs)?, &eval_expr(rhs)?)? {
        Ordering::Equal => Ok(Value::Bool(false)),
        _ => Ok(Value::Bool(true)),
    }
}

fn partial_cmp(lhs: &Value, rhs: &Value) -> Result<Ordering, ()> {
    match (lhs, rhs) {
        (Value::Int(lhs_num), Value::Int(rhs_num)) => lhs_num.partial_cmp(rhs_num).ok_or(()),
        (Value::Float(lhs_num), Value::Int(rhs_num)) => {
            lhs_num.partial_cmp(&(*rhs_num as f64)).ok_or(())
        }
        (Value::Int(lhs_num), Value::Float(rhs_num)) => {
            (*lhs_num as f64).partial_cmp(rhs_num).ok_or(())
        }
        (Value::Float(lhs_num), Value::Float(rhs_num)) => lhs_num.partial_cmp(rhs_num).ok_or(()),
        (Value::Str(lhs_str), Value::Str(rhs_str)) => lhs_str.partial_cmp(rhs_str).ok_or(()),
        (Value::List(lhs_list), Value::List(rhs_list)) => partial_cmp_list(lhs_list, rhs_list),
        (Value::Bool(lhs_bool), Value::Bool(rhs_bool)) => lhs_bool.partial_cmp(rhs_bool).ok_or(()),
        _ => todo!(),
    }
}

fn partial_cmp_list(lhs_list: &Vec<Value>, rhs_list: &Vec<Value>) -> Result<Ordering, ()> {
    if lhs_list.is_empty() {
        if rhs_list.is_empty() {
            return Ok(Ordering::Equal);
        }
        return Ok(Ordering::Less);
    }
    if rhs_list.is_empty() {
        return Ok(Ordering::Greater);
    }
    for (lhs_value, rhs_value) in lhs_list.iter().zip(rhs_list.iter()) {
        match partial_cmp(lhs_value, rhs_value)? {
            Ordering::Equal => continue,
            not_equal => return Ok(not_equal),
        }
    }
    lhs_list.len().partial_cmp(&rhs_list.len()).ok_or(())
}

fn eval_concat_bin_op(lhs: &Expr, rhs: &Expr) -> EvalResult {
    let Value::List(mut lhs_vector) = eval_expr(lhs)? else {
        todo!()
    };
    let Value::List(rhs_vector) = eval_expr(rhs)? else {
        todo!()
    };
    lhs_vector.extend(rhs_vector);
    Ok(Value::List(lhs_vector))
}

fn eval_update_bin_op(lhs: &Expr, rhs: &Expr) -> EvalResult {
    let Value::AttrSet(mut lhs_hash_map) = eval_expr(lhs)? else {
        todo!()
    };
    let Value::AttrSet(rhs_hash_map) = eval_expr(rhs)? else {
        todo!()
    };
    lhs_hash_map.extend(rhs_hash_map);
    Ok(Value::AttrSet(lhs_hash_map))
}

fn eval_has_op(has_op: &HasAttr) -> EvalResult {
    let mut lhs_value = &eval_expr(&has_op.expr().expect("Unreachable"))?;
    let attr_path = has_op.attrpath().expect("Unreachable");
    for attr in attr_path.attrs() {
        let attr_str = attr_to_str(&attr)?;
        let Value::AttrSet(hash_map) = lhs_value else {
            return Ok(Value::Bool(false));
        };
        let Some(attr_value) = hash_map.get(&attr_str) else {
            return Ok(Value::Bool(false));
        };
        lhs_value = attr_value;
    }
    Ok(Value::Bool(true))
}

fn attr_to_str(attr: &Attr) -> Result<String, ()> {
    Ok(match attr {
        Attr::Ident(ident) => ident.ident_token().expect("Unreachable").text().to_owned(),
        Attr::Str(str_expr) => {
            let Value::Str(attr_str) = eval_string_expr(str_expr)? else {
                todo!()
            };
            attr_str
        }
        _ => todo!(),
    })
}

fn eval_ident(ident: &Ident) -> EvalResult {
    let token = ident.ident_token().expect("Not implemented");
    match token.kind() {
        TOKEN_IDENT => eval_ident_token(&token),
        _ => todo!(),
    }
}

fn eval_ident_token(token: &SyntaxToken) -> EvalResult {
    Ok(match token.text() {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ => todo!(),
    })
}

fn eval_list(list: &List) -> EvalResult {
    let values_list: Result<Vec<Value>, ()> = list.items().map(|item| eval_expr(&item)).collect();
    Ok(Value::List(values_list?))
}

fn eval_literal(literal: &Literal) -> EvalResult {
    let token = literal.syntax().first_token().expect("Not implemented");
    Ok(match token.kind() {
        TOKEN_INTEGER => Value::Int(token.text().parse::<i64>().expect("Not implemented")),
        TOKEN_FLOAT => Value::Float(token.text().parse::<f64>().expect("Not implemented")),
        _ => todo!(),
    })
}

fn eval_unary_op(unary_op: &UnaryOp) -> EvalResult {
    let operator = unary_op.operator().expect("Not implemented");
    let operand = eval_expr(&unary_op.expr().expect("Not implemented"))?;
    match operator {
        UnaryOpKind::Invert => eval_invert_unary_op(&operand),
        UnaryOpKind::Negate => eval_negate_unary_op(&operand),
    }
}

fn eval_invert_unary_op(operand: &Value) -> EvalResult {
    let Value::Bool(operand_value) = operand else {
        todo!()
    };
    Ok(Value::Bool(!operand_value))
}

fn eval_negate_unary_op(operand: &Value) -> EvalResult {
    Ok(match operand {
        Value::Int(operand_int) => Value::Int(-operand_int),
        Value::Float(operand_float) => Value::Float(-operand_float),
        _ => todo!(),
    })
}

fn eval_select_op(select: &Select) -> EvalResult {
    let mut lhs_value = eval_expr(&select.expr().expect("Unreachable"))?;
    let attr_path = select.attrpath().expect("Unreachable");
    for attr in attr_path.attrs() {
        let attr_str = attr_to_str(&attr)?;
        let Value::AttrSet(mut hash_map) = lhs_value else {
            return eval_select_default(select);
        };
        let Some(attr_value) = hash_map.remove(&attr_str) else {
            return eval_select_default(select);
        };
        lhs_value = attr_value;
    }
    Ok(lhs_value)
}

fn eval_select_default(select: &Select) -> EvalResult {
    select
        .default_expr()
        .map(|expr| eval_expr(&expr))
        .transpose()?
        .ok_or(())
}

fn eval_string_expr(string: &Str) -> EvalResult {
    let mut tokens = string.syntax().children_with_tokens();
    if let None = tokens.next() {
        todo!()
    };
    let Some(NodeOrToken:: Token(string_content)) = tokens.next() else {
        todo!()
    };
    Ok(Value::Str(string_content.text().to_owned()))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn eval_ok(nix_expr: &str) -> Value {
        eval_str(nix_expr).expect("Shouldn't fail")
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
    fn test_eval_float_arithmetic() {
        assert_eq!(eval_ok("-1.0"), Value::Float(-1.0));
        assert_eq!(eval_ok("1 / 2.0"), Value::Float(0.5));
        assert_eq!(eval_ok("1.0 / 2"), Value::Float(0.5));
        assert_eq!(eval_ok("1.0 / 2.0"), Value::Float(0.5));
    }

    #[test]
    fn test_eval_bool_expr() {
        assert_eq!(eval_ok("true"), Value::Bool(true));
        assert_eq!(eval_ok("false"), Value::Bool(false));
        assert_eq!(eval_ok("!false"), Value::Bool(true));
        assert_eq!(eval_ok("false || true"), Value::Bool(true));
        assert_eq!(eval_ok("true && true"), Value::Bool(true));
        assert_eq!(eval_ok("false || true && false"), Value::Bool(false));
        assert_eq!(eval_ok("false && true || false"), Value::Bool(false));
        assert_eq!(eval_ok("true -> false"), Value::Bool(false));
    }

    #[test]
    fn test_eval_comparison_gt_expr() {
        assert_eq!(eval_ok("2 > 1"), Value::Bool(true));
        assert_eq!(eval_ok("1.1 > 1"), Value::Bool(true));
        assert_eq!(eval_ok("2 > 1.9"), Value::Bool(true));
        assert_eq!(eval_ok("1.8 > 1.9"), Value::Bool(false));

        assert_eq!(eval_ok(r#""b" > "a""#), Value::Bool(true));
        assert_eq!(eval_ok(r#""ab" > "aa""#), Value::Bool(true));
        assert_eq!(eval_ok(r#""ab" > "ac""#), Value::Bool(false));

        assert_eq!(eval_ok("[] > []"), Value::Bool(false));
        assert_eq!(eval_ok("[] > [1]"), Value::Bool(false));
        assert_eq!(eval_ok("[1] > []"), Value::Bool(true));
        assert_eq!(eval_ok("[1] > [1]"), Value::Bool(false));
        assert_eq!(eval_ok("[2] > [1]"), Value::Bool(true));
        assert_eq!(eval_ok("[1 1] > [1]"), Value::Bool(true));

        assert_eq!(eval_ok("[true] > []"), Value::Bool(true));
        assert_eq!(eval_ok("[true] > [true]"), Value::Bool(false));

        // TODO: `nix` throws an error here because booleans can only be compared for equality.
        // We have to make this fail.
        // assert_eq!(eval_str("[true] > [false]"), Err(()));
    }

    #[test]
    fn test_eval_comparison_gt_or_eq_expr() {
        assert_eq!(eval_ok("0 >= 1"), Value::Bool(false));
        assert_eq!(eval_ok("1 >= 1"), Value::Bool(true));
        assert_eq!(eval_ok("2 >= 1"), Value::Bool(true));

        assert_eq!(eval_ok(r#""a" >= "b""#), Value::Bool(false));
        assert_eq!(eval_ok(r#""b" >= "b""#), Value::Bool(true));
        assert_eq!(eval_ok(r#""c" >= "b""#), Value::Bool(true));

        assert_eq!(eval_ok("[] >= []"), Value::Bool(true));
        assert_eq!(eval_ok("[] >= [1]"), Value::Bool(false));
        assert_eq!(eval_ok("[1] >= []"), Value::Bool(true));
        assert_eq!(eval_ok("[1] >= [1]"), Value::Bool(true));
        assert_eq!(eval_ok("[2] >= [1]"), Value::Bool(true));
        assert_eq!(eval_ok("[1 1] >= [1]"), Value::Bool(true));

        assert_eq!(eval_ok("[true] >= []"), Value::Bool(true));
        assert_eq!(eval_ok("[true] >= [true]"), Value::Bool(true));

        // TODO: `nix` throws an error here because booleans can only be compared for equality.
        // We have to make this fail.
        // assert_eq!(eval_str("[true] >= [false]"), Err(()));
    }

    #[test]
    fn test_eval_comparison_lt_expr() {
        assert_eq!(eval_ok("1 < 2"), Value::Bool(true));
        assert_eq!(eval_ok("1 < 1.1"), Value::Bool(true));
        assert_eq!(eval_ok("1.9 < 2"), Value::Bool(true));
        assert_eq!(eval_ok("1.9 < 1.8"), Value::Bool(false));

        assert_eq!(eval_ok(r#""a" < "b""#), Value::Bool(true));
        assert_eq!(eval_ok(r#""aa" < "ab""#), Value::Bool(true));
        assert_eq!(eval_ok(r#""ac" < "ab""#), Value::Bool(false));

        assert_eq!(eval_ok("[] < []"), Value::Bool(false));
        assert_eq!(eval_ok("[1] < []"), Value::Bool(false));
        assert_eq!(eval_ok("[] < [1]"), Value::Bool(true));
        assert_eq!(eval_ok("[1] < [1]"), Value::Bool(false));
        assert_eq!(eval_ok("[1] < [2]"), Value::Bool(true));
        assert_eq!(eval_ok("[1] < [1 1]"), Value::Bool(true));

        assert_eq!(eval_ok("[] < [true]"), Value::Bool(true));
        assert_eq!(eval_ok("[true] < [true]"), Value::Bool(false));

        // TODO: `nix` throws an error here because booleans can only be compared for equality.
        // We have to make this fail.
        // assert_eq!(eval_str("[false] < [true]"), Err(()));
    }

    #[test]
    fn test_eval_comparison_lt_or_eq_expr() {
        assert_eq!(eval_ok("0 <= 1"), Value::Bool(true));
        assert_eq!(eval_ok("1 <= 1"), Value::Bool(true));
        assert_eq!(eval_ok("2 <= 1"), Value::Bool(false));

        assert_eq!(eval_ok(r#""a" <= "b""#), Value::Bool(true));
        assert_eq!(eval_ok(r#""b" <= "b""#), Value::Bool(true));
        assert_eq!(eval_ok(r#""c" <= "b""#), Value::Bool(false));

        assert_eq!(eval_ok("[] <= []"), Value::Bool(true));
        assert_eq!(eval_ok("[] <= [1]"), Value::Bool(true));
        assert_eq!(eval_ok("[1] <= []"), Value::Bool(false));
        assert_eq!(eval_ok("[1] <= [1]"), Value::Bool(true));
        assert_eq!(eval_ok("[2] <= [1]"), Value::Bool(false));
        assert_eq!(eval_ok("[1 1] <= [1]"), Value::Bool(false));
        assert_eq!(eval_ok("[1] <= [1 1]"), Value::Bool(true));

        assert_eq!(eval_ok("[true] <= []"), Value::Bool(false));
        assert_eq!(eval_ok("[true] <= [true]"), Value::Bool(true));

        // TODO: `nix` throws an error here because booleans can only be compared for equality.
        // We have to make this fail.
        // assert_eq!(eval_str("[true] <= [false]"), Err(()));
    }

    #[test]
    fn test_eval_comparison_eq_expr() {
        assert_eq!(eval_ok("1 == 1"), Value::Bool(true));
        assert_eq!(eval_ok("1 == 1.1"), Value::Bool(false));
        assert_eq!(eval_ok("1 == 1.0"), Value::Bool(true));

        assert_eq!(eval_ok(r#""a" == "a""#), Value::Bool(true));
        assert_eq!(eval_ok(r#""aa" == "ab""#), Value::Bool(false));

        assert_eq!(eval_ok("[] == []"), Value::Bool(true));
        assert_eq!(eval_ok("[1] == [1]"), Value::Bool(true));
        assert_eq!(eval_ok("[1] == [1 1]"), Value::Bool(false));

        assert_eq!(eval_ok("[true] == [true]"), Value::Bool(true));
        assert_eq!(eval_ok("[true] == [false]"), Value::Bool(false));
    }

    #[test]
    fn test_eval_comparison_not_eq_expr() {
        assert_eq!(eval_ok("1 != 1"), Value::Bool(false));
        assert_eq!(eval_ok("1 != 1.1"), Value::Bool(true));
        assert_eq!(eval_ok("1 != 1.0"), Value::Bool(false));

        assert_eq!(eval_ok(r#""a" != "a""#), Value::Bool(false));
        assert_eq!(eval_ok(r#""aa" != "ab""#), Value::Bool(true));

        assert_eq!(eval_ok("[] != []"), Value::Bool(false));
        assert_eq!(eval_ok("[1] != [1]"), Value::Bool(false));
        assert_eq!(eval_ok("[1] != [1 1]"), Value::Bool(true));

        assert_eq!(eval_ok("[true] != [true]"), Value::Bool(false));
        assert_eq!(eval_ok("[true] != [false]"), Value::Bool(true));
    }

    #[test]
    fn test_eval_paren() {
        assert_eq!(eval_ok("(1 + 2) + 3"), Value::Int(6));
    }

    #[test]
    fn test_eval_string_expr() {
        assert_eq!(eval_ok("\"Hello!\""), Value::Str("Hello!".to_owned()));
    }

    #[test]
    fn test_eval_string_concat_op() {
        assert_eq!(
            eval_ok("\"Hell\" + \"o!\""),
            Value::Str("Hello!".to_owned())
        );
    }

    #[test]
    fn test_eval_string_mul_op() {
        assert_eq!(eval_str("\"a\" * \"b\""), Err(()));
    }

    #[test]
    fn test_eval_list_expr() {
        assert_eq!(
            eval_ok("[ 42 true \"answer\" ]"),
            Value::List(vec![
                Value::Int(42),
                Value::Bool(true),
                Value::Str("answer".to_owned())
            ])
        );
    }

    #[test]
    fn test_eval_concat_bin_op() {
        assert_eq!(
            eval_ok("[1] ++ [2]"),
            Value::List(vec![Value::Int(1), Value::Int(2),])
        );
    }

    #[test]
    fn test_eval_attrset_expr() {
        assert_eq!(
            eval_ok("{a = 42;}"),
            Value::AttrSet(HashMap::from([("a".to_owned(), Value::Int(42))]))
        );
    }

    #[test]
    fn test_eval_update_bin_op() {
        assert_eq!(
            eval_ok("{a = 1; b = 2;} // {a = 3; c = 1;"),
            Value::AttrSet(HashMap::from([
                ("a".to_owned(), Value::Int(3)),
                ("b".to_owned(), Value::Int(2)),
                ("c".to_owned(), Value::Int(1)),
            ]))
        );
    }

    #[test]
    fn test_eval_has_op() {
        assert_eq!(eval_ok("{a = 1;} ? a"), Value::Bool(true));
        assert_eq!(eval_ok("{a = 1;} ? \"a\""), Value::Bool(true));
        assert_eq!(eval_ok("{a = {b = 1;};} ? a.c"), Value::Bool(false));
    }

    #[test]
    fn test_eval_select_op() {
        assert_eq!(eval_ok("{a = 1;}.a"), Value::Int(1));
        assert_eq!(eval_ok("{a = 1;}.b or 2"), Value::Int(2));
    }
}
