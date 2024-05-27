#![allow(unused_imports)]
#![allow(non_snake_case)]

use crate::{eval::error::NixErrorKind, tests::eval_err};
use crate::{
    eval::types::{NixTypeKind, Value},
    tests::eval_ok,
};

// Builtins are sorted by the order they appear in the Nix manual
// https://nixos.org/manual/nix/stable/language/builtins.html

mod derivation {
    use super::*;
}

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

mod addDrvOutputDependencies {
    use super::*;
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

mod attrNames {
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

mod attrValues {
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

mod baseNameOf {
    use super::*;

    // Returns everything following the final slash

    #[test]
    fn eval() {
        assert_eq!(
            eval_ok("builtins.baseNameOf \"/foo/bar/baz\""),
            Value::Str("baz".into())
        );
        assert_eq!(
            eval_ok("builtins.baseNameOf \"/foo/bar/baz/\""),
            Value::Str("baz".into())
        );
        assert_eq!(
            eval_ok("builtins.baseNameOf \"/foo/bar/baz//\""),
            Value::Str("".into())
        );
        assert_eq!(
            eval_ok("builtins.baseNameOf \"foo\""),
            Value::Str("foo".into())
        );
    }

    #[test]
    fn eval_path() {
        assert_eq!(
            eval_ok("builtins.baseNameOf /foo/bar/baz"),
            Value::Str("baz".into())
        );
        assert_eq!(
            eval_ok("builtins.baseNameOf ./foo"),
            Value::Str("foo".into())
        );
    }

    #[test]
    fn eval_invalid_types() {
        assert_eq!(
            eval_err("builtins.baseNameOf 1"),
            NixErrorKind::TypeMismatch {
                expected: vec![NixTypeKind::String, NixTypeKind::Path],
                got: NixTypeKind::Int
            }
        );
    }
}

mod bitAnd {
    use super::*;
}

mod bitOr {
    use super::*;
}

mod bitXor {
    use super::*;
}

mod break_ {
    use super::*;
}

mod catAttrs {
    use super::*;
}

mod ceil {
    use super::*;
}

mod compareVersions {
    use super::*;
}

mod concatLists {
    use super::*;
}

mod concatMap {
    use super::*;
}

mod concatStringsSep {
    use super::*;
}

mod convertHash {
    use super::*;
}

mod deepSeq {
    use super::*;
}

mod dirOf {
    use super::*;
}

mod div {
    use super::*;
}

mod elem {
    use super::*;
}

mod elemAt {
    use super::*;
}

mod fetchClosure {
    use super::*;
}

mod fetchGit {
    use super::*;
}

mod fetchTarball {
    use super::*;
}

mod fetchTree {
    use super::*;
}

mod fetchurl {
    use super::*;
}

mod filter {
    use super::*;
}

mod filterSource {
    use super::*;
}

mod findFile {
    use super::*;
}

mod flakeRefToString {
    use super::*;
}

mod floor {
    use super::*;
}

mod foldl {
    use super::*;
}

mod fromJSON {
    use super::*;
}

mod fromTOML {
    use super::*;
}

mod functionArgs {
    use super::*;
}

mod genList {
    use super::*;
}

mod genericClosure {
    use super::*;
}

mod getAttr {
    use super::*;
}

mod getContext {
    use super::*;
}

mod getEnv {
    use super::*;
}

mod getFlake {
    use super::*;
}

mod groupBy {
    use super::*;
}

mod hasAttr {
    use super::*;
}

mod hasContext {
    use super::*;
}

mod hashFile {
    use super::*;
}

mod hashString {
    use super::*;
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
                codename: "builtins-head-on-empty-list".to_string()
            }
        );
    }
}

mod import {
    use super::*;

    #[test]
    fn eval() {
        assert_eq!(
            eval_ok("(builtins.import ./src/tests/import_tests/basic.nix).data"),
            Value::Str("imported!".into())
        );
    }

    #[test]
    fn eval_same_folder_import() {
        assert_eq!(
            eval_ok("(builtins.import ./src/tests/import_tests/same-folder-import.nix).dataPath"),
            Value::Str("imported!".into())
        );
        assert_eq!(
            eval_ok("(builtins.import ./src/tests/import_tests/same-folder-import.nix).dataString"),
            Value::Str("imported!".into())
        );
    }

    #[test]
    fn eval_child_folder_import() {
        assert_eq!(
            eval_ok("(builtins.import ./src/tests/import_tests/child-folder-import.nix).dataPath"),
            Value::Str("imported!".into())
        );
        assert_eq!(
            eval_ok(
                "(builtins.import ./src/tests/import_tests/child-folder-import.nix).dataString"
            ),
            Value::Str("imported!".into())
        );
    }

    #[test]
    fn eval_parent_folder_import() {
        assert_eq!(
            eval_ok("(builtins.import ./src/tests/import_tests/nested/parent-folder-import.nix).dataPath"),
            Value::Str("imported!".into())
        );
        assert_eq!(
            eval_ok("(builtins.import ./src/tests/import_tests/nested/parent-folder-import.nix).dataString"),
            Value::Str("imported!".into())
        );
    }

    #[test]
    fn eval_relative_string() {
        assert_eq!(
            eval_err(r#"builtins.import "./foo.nix""#),
            NixErrorKind::Other {
                codename: "builtins-import-non-absolute-path".to_owned()
            }
        )
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

mod intersectAttrs {
    use super::*;
}

mod isAttrs {
    use super::*;
}

mod isBool {
    use super::*;
}

mod isFloat {
    use super::*;
}

mod isFunction {
    use super::*;
}

mod isInt {
    use super::*;
}

mod isList {
    use super::*;
}

mod isNull {
    use super::*;
}

mod isPath {
    use super::*;
}

mod isString {
    use super::*;
}

mod length {
    use super::*;
}

mod lessThan {
    use super::*;
}

mod listToAttrs {
    use super::*;
}

mod map {
    use super::*;
}

mod mapAttrs {
    use super::*;
}

mod match_ {
    use super::*;
}

mod mul {
    use super::*;
}

mod outputOf {
    use super::*;
}

mod parseDrvName {
    use super::*;
}

mod parseFlakeRef {
    use super::*;
}

mod partition {
    use super::*;
}

mod path {
    use super::*;
}

mod pathExists {
    use super::*;
}

mod placeholder {
    use super::*;
}

mod readDir {
    use super::*;
}

mod readFile {
    use super::*;
}

mod readFileType {
    use super::*;
}

mod removeAttrs {
    use super::*;
}

mod replaceStrings {
    use super::*;
}

mod seq {
    use super::*;
}

mod sort {
    use super::*;
}

mod split {
    use super::*;
}

mod splitVersion {
    use super::*;
}

mod storePath {
    use super::*;
}

mod stringLength {
    use super::*;
}

mod sub {
    use super::*;
}

mod substring {
    use super::*;
}

mod tail {
    use super::*;
}

mod throw {
    use super::*;
}

mod toFile {
    use super::*;
}

mod toJSON {
    use super::*;
}

mod toPath {
    use super::*;
}

mod toString {
    use super::*;
}

mod toXML {
    use super::*;
}

mod trace {
    use super::*;
}

mod traceVerbose {
    use super::*;
}

mod tryEval {
    use super::*;
}

mod typeOf {
    use super::*;
}

mod unsafeDiscardOutputDependency {
    use super::*;
}

mod zipAttrsWith {
    use super::*;
}
