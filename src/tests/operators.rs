use crate::{
    eval::{
        error::NixErrorKind,
        types::{NixTypeKind, Value},
    },
    tests::{eval_err, eval_ok},
};

#[test]
fn eval_int_arithmetic() {
    assert_eq!(eval_ok("1 + 2"), Value::Int(3));
    assert_eq!(eval_ok("1 - 2"), Value::Int(-1));
    assert_eq!(eval_ok("1 * 2"), Value::Int(2));
    assert_eq!(eval_ok("1 / 2"), Value::Int(0));
}

#[test]
fn eval_float_arithmetic() {
    assert_eq!(eval_ok("1.0 + 2.0"), Value::Float(3.0));
    assert_eq!(eval_ok("1.0 - 2.0"), Value::Float(-1.0));
    assert_eq!(eval_ok("1.0 * 2.0"), Value::Float(2.0));
    assert_eq!(eval_ok("1.0 / 2.0"), Value::Float(0.5));
}

#[test]
fn eval_mixed_arithmetic() {
    assert_eq!(eval_ok("1 + 2.0"), Value::Float(3.0));
    assert_eq!(eval_ok("1 - 2.0"), Value::Float(-1.0));
    assert_eq!(eval_ok("1 * 2.0"), Value::Float(2.0));
    assert_eq!(eval_ok("1 / 2.0"), Value::Float(0.5));
    assert_eq!(eval_ok("2.0 + 1"), Value::Float(3.0));
    assert_eq!(eval_ok("2.0 - 1"), Value::Float(1.0));
    assert_eq!(eval_ok("2.0 * 1"), Value::Float(2.0));
    assert_eq!(eval_ok("2.0 / 1"), Value::Float(2.0));
}

#[test]
fn eval_string_concatenation() {
    assert_eq!(
        eval_ok("\"hello\" + \"world\""),
        Value::Str("helloworld".to_string())
    );
    assert_eq!(
        eval_ok("\"hello\" + \" \" + \"world\""),
        Value::Str("hello world".to_string())
    );
}

#[test]
fn eval_int_comparison() {
    assert_eq!(eval_ok("1 == 1"), Value::Bool(true));
    assert_eq!(eval_ok("1 == 2"), Value::Bool(false));
    assert_eq!(eval_ok("1 != 2"), Value::Bool(true));
    assert_eq!(eval_ok("1 != 1"), Value::Bool(false));
    assert_eq!(eval_ok("1 < 2"), Value::Bool(true));
    assert_eq!(eval_ok("1 < 1"), Value::Bool(false));
    assert_eq!(eval_ok("1 > 2"), Value::Bool(false));
    assert_eq!(eval_ok("1 > 1"), Value::Bool(false));
    assert_eq!(eval_ok("1 <= 2"), Value::Bool(true));
    assert_eq!(eval_ok("1 <= 1"), Value::Bool(true));
    assert_eq!(eval_ok("1 >= 2"), Value::Bool(false));
    assert_eq!(eval_ok("1 >= 1"), Value::Bool(true));
}

#[test]
fn eval_float_comparison() {
    assert_eq!(eval_ok("1.0 == 1.0"), Value::Bool(true));
    assert_eq!(eval_ok("1.0 == 2.0"), Value::Bool(false));
    assert_eq!(eval_ok("1.0 != 2.0"), Value::Bool(true));
    assert_eq!(eval_ok("1.0 != 1.0"), Value::Bool(false));
    assert_eq!(eval_ok("1.0 < 2.0"), Value::Bool(true));
    assert_eq!(eval_ok("1.0 < 1.0"), Value::Bool(false));
    assert_eq!(eval_ok("1.0 > 2.0"), Value::Bool(false));
    assert_eq!(eval_ok("1.0 > 1.0"), Value::Bool(false));
    assert_eq!(eval_ok("1.0 <= 2.0"), Value::Bool(true));
    assert_eq!(eval_ok("1.0 <= 1.0"), Value::Bool(true));
    assert_eq!(eval_ok("1.0 >= 2.0"), Value::Bool(false));
    assert_eq!(eval_ok("1.0 >= 1.0"), Value::Bool(true));
}

#[test]
fn eval_float_int_comparison() {
    assert_eq!(eval_ok("1.0 == 1"), Value::Bool(true));
    assert_eq!(eval_ok("1.0 == 2"), Value::Bool(false));
    assert_eq!(eval_ok("1.0 != 2"), Value::Bool(true));
    assert_eq!(eval_ok("1.0 != 1"), Value::Bool(false));
    assert_eq!(eval_ok("1.0 < 2"), Value::Bool(true));
    assert_eq!(eval_ok("1.0 < 1"), Value::Bool(false));
    assert_eq!(eval_ok("1.0 > 2"), Value::Bool(false));
    assert_eq!(eval_ok("1.0 > 1"), Value::Bool(false));
    assert_eq!(eval_ok("1.0 <= 2"), Value::Bool(true));
    assert_eq!(eval_ok("1.0 <= 1"), Value::Bool(true));
    assert_eq!(eval_ok("1.0 >= 2"), Value::Bool(false));
    assert_eq!(eval_ok("1.0 >= 1"), Value::Bool(true));

    assert_eq!(eval_ok("1 == 1.0"), Value::Bool(true));
    assert_eq!(eval_ok("1 == 2.0"), Value::Bool(false));
    assert_eq!(eval_ok("1 != 2.0"), Value::Bool(true));
    assert_eq!(eval_ok("1 != 1.0"), Value::Bool(false));
    assert_eq!(eval_ok("1 < 2.0"), Value::Bool(true));
    assert_eq!(eval_ok("1 < 1.0"), Value::Bool(false));
    assert_eq!(eval_ok("1 > 2.0"), Value::Bool(false));
    assert_eq!(eval_ok("1 > 1.0"), Value::Bool(false));
    assert_eq!(eval_ok("1 <= 2.0"), Value::Bool(true));
    assert_eq!(eval_ok("1 <= 1.0"), Value::Bool(true));
    assert_eq!(eval_ok("1 >= 2.0"), Value::Bool(false));
    assert_eq!(eval_ok("1 >= 1.0"), Value::Bool(true));
}

#[test]
fn eval_string_comparison() {
    assert_eq!(eval_ok("\"abc\" == \"abc\""), Value::Bool(true));
    assert_eq!(eval_ok("\"abc\" == \"def\""), Value::Bool(false));
    assert_eq!(eval_ok("\"abc\" != \"def\""), Value::Bool(true));
    assert_eq!(eval_ok("\"abc\" != \"abc\""), Value::Bool(false));
    assert_eq!(eval_ok("\"abc\" < \"def\""), Value::Bool(true));
    assert_eq!(eval_ok("\"abc\" < \"abc\""), Value::Bool(false));
    assert_eq!(eval_ok("\"abc\" > \"def\""), Value::Bool(false));
    assert_eq!(eval_ok("\"abc\" > \"abc\""), Value::Bool(false));
    assert_eq!(eval_ok("\"abc\" <= \"def\""), Value::Bool(true));
    assert_eq!(eval_ok("\"abc\" <= \"abc\""), Value::Bool(true));
    assert_eq!(eval_ok("\"abc\" >= \"def\""), Value::Bool(false));
    assert_eq!(eval_ok("\"abc\" >= \"abc\""), Value::Bool(true));
}

#[test]
fn eval_path_string_concatenation() {
    let curr_dir = std::env::current_dir().unwrap();

    assert_eq!(
        eval_ok("./hello + \"world\""),
        Value::Path(format!("{}/helloworld", curr_dir.display()))
    );
    assert_eq!(
        eval_ok("\"hello\" + ./world"),
        Value::Str(format!("hello{}/world", curr_dir.display()))
    );
}

#[test]
fn eval_path_concat() {
    let curr_dir = std::env::current_dir().unwrap();

    assert_eq!(
        eval_ok("./foo + ./bar"),
        Value::Path(format!(
            "{}/foo{}/bar",
            curr_dir.display(),
            curr_dir.display()
        ))
    );

    assert_eq!(eval_ok(r#"/. + "a""#), Value::Path("/a".to_owned()));
    assert_eq!(eval_ok(r#"/. + "./a/../b""#), Value::Path("/b".to_owned()));
}

#[test]
fn eval_order_of_operations() {
    // Addition and subtraction have the same precedence, so they should be evaluated from left to right
    assert_eq!(eval_ok("1 + 2 - 3"), Value::Int(0));
    assert_eq!(eval_ok("1 - 2 + 3"), Value::Int(2));

    // Multiplication and division have higher precedence than addition and subtraction
    assert_eq!(eval_ok("1 + 2 * 3"), Value::Int(7));
    assert_eq!(eval_ok("1 * 2 + 3"), Value::Int(5));

    // Parentheses can be used to change the order of operations
    assert_eq!(eval_ok("(1 + 2) * 3"), Value::Int(9));
    assert_eq!(eval_ok("1 * (2 + 3)"), Value::Int(5));
}

#[test]
fn eval_string_operator_errors() {
    assert_eq!(
        eval_err("1 + \"hello\""),
        NixErrorKind::TypeMismatch {
            expected: vec![],
            got: NixTypeKind::Int
        }
    );
    assert_eq!(
        eval_err("\"hello\" - \"world\""),
        NixErrorKind::TypeMismatch {
            expected: vec![],
            got: NixTypeKind::String
        }
    );
    assert_eq!(
        eval_err("\"hello\" * \"world\""),
        NixErrorKind::TypeMismatch {
            expected: vec![],
            got: NixTypeKind::String
        }
    );
    assert_eq!(
        eval_err("\"hello\" / \"world\""),
        NixErrorKind::TypeMismatch {
            expected: vec![],
            got: NixTypeKind::String
        }
    );

    // FIXME: It says string instead of int, because nixjs flips the operator. Should we address?
    assert_eq!(
        eval_err("\"hello\" < 123"),
        NixErrorKind::TypeMismatch {
            expected: vec![],
            got: NixTypeKind::String
        }
    );
    assert_eq!(
        eval_err("\"hello\" > 123"),
        NixErrorKind::TypeMismatch {
            expected: vec![],
            got: NixTypeKind::Int
        }
    );
    assert_eq!(
        eval_err("\"hello\" <= 123"),
        NixErrorKind::TypeMismatch {
            expected: vec![],
            got: NixTypeKind::Int
        }
    );
    // FIXME: It says string instead of int, because nixjs flips the operator. Should we address?
    assert_eq!(
        eval_err("\"hello\" >= 123"),
        NixErrorKind::TypeMismatch {
            expected: vec![],
            got: NixTypeKind::String
        }
    );
}

#[test]
fn eval_bool_operations() {
    assert_eq!(eval_ok("!false"), Value::Bool(true));
    assert_eq!(eval_ok("false || true"), Value::Bool(true));
    assert_eq!(eval_ok("false || !false"), Value::Bool(true));
    assert_eq!(eval_ok("true && true"), Value::Bool(true));
    assert_eq!(eval_ok("false || true && false"), Value::Bool(false));
    assert_eq!(eval_ok("false && true || false"), Value::Bool(false));
    assert_eq!(eval_ok("true -> false"), Value::Bool(false));
}

#[test]
fn eval_list_operations() {
    assert_eq!(
        eval_ok("[1] ++ [2]"),
        Value::List(vec![Value::Int(1), Value::Int(2)])
    );
    assert_eq!(
        eval_ok("[1] ++ [2] ++ [3]"),
        Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
    );
    assert_eq!(
        eval_ok(r#"["a"] ++ [1] ++ [[] [] "b"]"#),
        Value::List(vec![
            Value::Str("a".to_owned()),
            Value::Int(1),
            Value::List(vec![]),
            Value::List(vec![]),
            Value::Str("b".to_owned())
        ])
    );
}
