use crate::{
    eval::{error::NixErrorKind, types::Value},
    tests::{eval_err, eval_ok},
};

#[test]
fn eval_lambda() {
    assert_eq!(eval_ok("a: 1"), Value::Lambda);
}

#[test]
fn eval_lambda_application() {
    assert_eq!(eval_ok("(a: 1) 2"), Value::Int(1));
    assert_eq!(eval_ok("(a: a + 1) 2"), Value::Int(3));
}

#[test]
fn eval_pattern_lambda() {
    assert_eq!(eval_ok("({a, b}: a + b) {a = 1; b = 2;}"), Value::Int(3));
    assert_eq!(eval_ok("({a, b ? 2}: a + b) {a = 1;}"), Value::Int(3));
    // TODO: Improve error
    assert_eq!(
        eval_err("{a, a}: a"),
        NixErrorKind::UnexpectedRustError {
            message: "duplicate formal function argument 'a'.".to_string()
        }
    );
}

#[test]
fn eval_pattern_lambda_args_binding() {
    assert_eq!(eval_ok("({a}@args: args.a) {a = 1;}"), Value::Int(1));
    // TODO: Improve error
    assert_eq!(
        eval_err("{a}@a: a"),
        NixErrorKind::UnexpectedRustError {
            message: "duplicate formal function argument 'a'.".to_string()
        }
    );
    assert_eq!(
        eval_err("({a ? 1}@args: args.a) {}"),
        NixErrorKind::MissingAttribute {
            attr_path: vec!["a".to_string()]
        }
    );
}
