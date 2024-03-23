#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::eval::types::Value;

    use super::super::execution::*;

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
    fn test_eval_lists_are_lazy() {
        assert_eq!(eval_ok("builtins.head [ 1 (1 / 0) ]"), Value::Int(1));
    }

    #[test]
    fn test_eval_builtin_abort() {
        let error_msg = evaluate(r#"abort "foo""#).unwrap_err();
        let expected_msg = "Evaluation aborted with the following error message: 'foo'";
        assert!(
            error_msg.contains(expected_msg),
            "Error message '{error_msg}' didn't contain '{expected_msg}'."
        );
    }

    #[test]
    fn test_eval_builtin_add() {
        assert_eq!(eval_ok("builtins.add 1 2"), Value::Int(3));
    }

    #[test]
    fn test_eval_builtin_head() {
        assert_eq!(eval_ok("builtins.head [ 1 2 ]"), Value::Int(1));
    }

    #[test]
    fn test_eval_builtin_all() {
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
    fn test_eval_builtin_all_lazy() {
        assert_eq!(
            eval_ok("builtins.all (a: false) [ 1 (1 / 0) ]"),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_eval_builtin_any() {
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
    fn test_eval_builtin_any_lazy() {
        assert_eq!(
            eval_ok("builtins.any (a: true) [ 1 (1 / 0) ]"),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_eval_builtin_attr_names() {
        assert_eq!(
            eval_ok("builtins.attrNames { b = true; a = true; }"),
            Value::List(vec![Value::Str("a".to_owned()), Value::Str("b".to_owned())])
        );
    }

    #[test]
    fn test_eval_builtin_attr_values() {
        assert_eq!(
            eval_ok("builtins.attrValues { b = true; a = false; }"),
            Value::List(vec![Value::Bool(false), Value::Bool(true)])
        );
    }

    #[test]
    fn test_eval_builtin_attr_values_lazy() {
        assert_eq!(
            eval_ok("builtins.head (builtins.attrValues { b = 1 / 0; a = false; })"),
            Value::Bool(false)
        );
    }
}
