use crate::{eval::error::NixErrorKind, tests::eval_err};
use crate::{
    eval::types::{NixTypeKind, Value},
    tests::eval_ok,
};

mod abort {

    use super::*;

    #[test]
    fn eval() {
        assert_eq!(
            eval_err("builtins.abort \"foo\""),
            NixErrorKind::Abort {
                message: "foo".to_owned()
            }
        );
    }
}

mod add {
    use super::*;

    #[test]
    fn eval_ints() {
        assert_eq!(eval_ok("builtins.add 1 2"), Value::Int(3));
    }

    #[test]
    fn eval_floats() {
        assert_eq!(eval_ok("builtins.add 1.0 2.0"), Value::Float(3.0));
    }

    #[test]
    fn eval_mixed() {
        assert_eq!(eval_ok("builtins.add 1 2.0"), Value::Float(3.0));
        assert_eq!(eval_ok("builtins.add 1.0 2"), Value::Float(3.0));
    }
}

mod head {
    use super::*;

    #[test]
    fn eval() {
        assert_eq!(eval_ok("builtins.head [ 1 2 ]"), Value::Int(1));
    }

    #[test]
    fn eval_lazy() {
        assert_eq!(eval_ok("builtins.head [ 1 (1 / 0) ]"), Value::Int(1));
    }

    #[test]
    fn eval_empty() {
        // Would be weird to have a custom error message kind for this, imo.
        assert_eq!(
            eval_err("builtins.head []"),
            NixErrorKind::Other {
                message: "Cannot fetch the first element in an empty list.".to_string()
            }
        );
    }
}

mod all {
    use super::*;

    #[test]
    fn eval() {
        assert_eq!(
            eval_ok("builtins.all (a: a == 1) [ 1 1 ]"),
            Value::Bool(true)
        );
        assert_eq!(
            eval_ok("builtins.all (a: a == 1) [ 1 2 ]"),
            Value::Bool(false)
        );
    }

    #[test]
    fn eval_lazy() {
        assert_eq!(
            eval_ok("builtins.all (a: false) [ 1 (1 / 0) ]"),
            Value::Bool(false)
        );
    }

    #[test]
    fn eval_empty() {
        assert_eq!(eval_ok("builtins.all (a: a == 1) []"), Value::Bool(true));
    }

    #[test]
    fn eval_non_lambda() {
        assert_eq!(
            eval_err("builtins.all 1 [ 1 2 ]"),
            NixErrorKind::TypeMismatch {
                expected: vec![NixTypeKind::Lambda],
                got: NixTypeKind::Int
            }
        );
    }

    #[test]
    fn eval_non_list() {
        assert_eq!(
            eval_err("builtins.all (a: a == 1) 1"),
            NixErrorKind::TypeMismatch {
                expected: vec![NixTypeKind::List],
                got: NixTypeKind::Int
            }
        );
    }
}

mod any {
    use super::*;

    #[test]
    fn eval() {
        assert_eq!(
            eval_ok("builtins.any (a: a == 1) [ 1 2 ]"),
            Value::Bool(true)
        );
        assert_eq!(
            eval_ok("builtins.any (a: a == 1) [ 2 2 ]"),
            Value::Bool(false)
        );
    }

    #[test]
    fn eval_lazy() {
        assert_eq!(
            eval_ok("builtins.any (a: true) [ 1 (1 / 0) ]"),
            Value::Bool(true)
        );
    }

    #[test]
    fn eval_empty() {
        assert_eq!(eval_ok("builtins.any (a: a == 1) []"), Value::Bool(false));
    }

    #[test]
    fn eval_non_lambda() {
        assert_eq!(
            eval_err("builtins.any 1 [ 1 2 ]"),
            NixErrorKind::TypeMismatch {
                expected: vec![NixTypeKind::Lambda],
                got: NixTypeKind::Int
            }
        );
    }

    #[test]
    fn eval_non_list() {
        assert_eq!(
            eval_err("builtins.any (a: a == 1) 1"),
            NixErrorKind::TypeMismatch {
                expected: vec![NixTypeKind::List],
                got: NixTypeKind::Int
            }
        );
    }
}

mod attr_names {
    use super::*;

    #[test]
    fn eval() {
        assert_eq!(
            eval_ok("builtins.attrNames { b = true; a = false; }"),
            Value::List(vec![Value::Str("a".into()), Value::Str("b".into())])
        );
    }

    #[test]
    fn eval_lazy() {
        assert_eq!(
            eval_ok("builtins.head (builtins.attrNames { b = 1 / 0; a = false; })"),
            Value::Str("a".into())
        );
    }

    #[test]
    fn eval_empty() {
        assert_eq!(eval_ok("builtins.attrNames {}"), Value::List(Vec::new()));
    }

    #[test]
    fn eval_nested() {
        assert_eq!(
            eval_ok("builtins.attrNames { a = { b = 1; }; }"),
            Value::List(vec![Value::Str("a".into())])
        );
    }

    #[test]
    fn eval_non_attr_set() {
        assert_eq!(
            eval_err("builtins.attrNames 1"),
            NixErrorKind::TypeMismatch {
                expected: vec![NixTypeKind::Set],
                got: NixTypeKind::Int
            }
        );
    }
}

mod attr_values {
    use super::*;

    #[test]
    fn eval() {
        assert_eq!(
            eval_ok("builtins.attrValues { b = true; a = false; }"),
            Value::List(vec![Value::Bool(false), Value::Bool(true)])
        );
    }

    #[test]
    fn eval_lazy() {
        assert_eq!(
            eval_ok("builtins.head (builtins.attrValues { b = 1 / 0; a = false; })"),
            Value::Bool(false)
        );
    }

    #[test]
    fn eval_empty() {
        assert_eq!(eval_ok("builtins.attrValues {}"), Value::List(Vec::new()));
    }

    #[test]
    fn eval_nested() {
        assert_eq!(
            eval_ok("builtins.attrValues { a = { b = 1; }; }"),
            Value::List(vec![Value::AttrSet(
                vec![("b".into(), Value::Int(1))].into_iter().collect()
            )])
        );
    }

    #[test]
    fn eval_non_attr_set() {
        assert_eq!(
            eval_err("builtins.attrValues 1"),
            NixErrorKind::TypeMismatch {
                expected: vec![NixTypeKind::Set],
                got: NixTypeKind::Int
            }
        );
    }
}

mod import {
    use super::*;

    #[test]
    fn eval() {
        assert_eq!(
            eval_ok("(builtins.import ./flake.nix).description"),
            Value::Str("A reimplementation or nix in Rust.".into())
        );
    }

    #[test]
    fn eval_lazy() {
        assert_eq!(
            eval_ok("let value = (builtins.import ./error.nix); in 1"),
            Value::Int(1)
        );
    }

    // TODO: Make this test work.
    // fn eval_invalid_file() {
    //     assert_eq!(
    //         eval_err("builtins.import ./non_existent_file.nix"),
    //         NixErrorKind::Import {
    //             path: "./non_existent_file.nix".to_owned()
    //         }
    //     );
    // }
}
