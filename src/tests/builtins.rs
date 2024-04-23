use crate::{eval::types::Value, tests::eval_ok};

mod add {
    use super::*;

    #[test]
    fn test_eval() {
        assert_eq!(eval_ok("builtins.add 1 2"), Value::Int(3));
    }
}

mod head {
    use super::*;

    #[test]
    fn test_eval() {
        assert_eq!(eval_ok("builtins.head [ 1 2 ]"), Value::Int(1));
    }

    #[test]
    fn test_eval_lazy() {
        assert_eq!(eval_ok("builtins.head [ 1 (1 / 0) ]"), Value::Int(1));
    }
}

mod all {
    use super::*;

    #[test]
    fn test_eval() {
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
    fn test_eval_lazy() {
        assert_eq!(
            eval_ok("builtins.all (a: false) [ 1 (1 / 0) ]"),
            Value::Bool(false)
        );
    }
}

mod any {
    use super::*;

    #[test]
    fn test_eval() {
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
    fn test_eval_lazy() {
        assert_eq!(
            eval_ok("builtins.any (a: true) [ 1 (1 / 0) ]"),
            Value::Bool(true)
        );
    }
}

mod attr_names {
    use super::*;

    #[test]
    fn test_eval() {
        assert_eq!(
            eval_ok("builtins.attrNames { b = true; a = false; }"),
            Value::List(vec![Value::Str("a".into()), Value::Str("b".into())])
        );
    }

    #[test]
    fn test_eval_lazy() {
        assert_eq!(
            eval_ok("builtins.head (builtins.attrNames { b = 1 / 0; a = false; })"),
            Value::Str("a".into())
        );
    }
}

mod attr_values {
    use super::*;

    #[test]
    fn test_eval() {
        assert_eq!(
            eval_ok("builtins.attrValues { b = true; a = false; }"),
            Value::List(vec![Value::Bool(false), Value::Bool(true)])
        );
    }

    #[test]
    fn test_eval_lazy() {
        assert_eq!(
            eval_ok("builtins.head (builtins.attrValues { b = 1 / 0; a = false; })"),
            Value::Bool(false)
        );
    }
}

mod import {
    use super::*;

    #[test]
    fn test_eval() {
        assert_eq!(
            eval_ok("(builtins.import ./flake.nix).description"),
            Value::Str("A reimplementation or nix in Rust.".into())
        );
    }

    #[test]
    fn test_eval_lazy() {
        assert_eq!(
            eval_ok("let value = (builtins.import ./error.nix); in 1"),
            Value::Int(1)
        );
    }
}
