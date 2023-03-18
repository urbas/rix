use std::{
    path::{Component, PathBuf},
    str::FromStr,
};

pub fn create_builtins_obj<'s>(scope: &mut v8::HandleScope<'s>) -> v8::Local<'s, v8::Object> {
    let builtins_obj = v8::Object::new(scope);

    add_builtin(scope, &builtins_obj, "isAbsolutePath", is_abolute_path);
    add_builtin(scope, &builtins_obj, "joinPaths", join_paths);

    builtins_obj
}

pub fn is_abolute_path<'a, 'b>(
    scope: &mut v8::HandleScope<'a>,
    args: v8::FunctionCallbackArguments,
    mut ret_val: v8::ReturnValue<'b>,
) {
    let path_js_str: v8::Local<v8::String> = args.get(0).try_into().expect("Expected a string.");
    let path = PathBuf::from_str(&path_js_str.to_rust_string_lossy(scope))
        .expect("Given string is not a path.");
    let is_absolute_js_bool: v8::Local<v8::Value> = v8::Boolean::new(scope, path.is_absolute())
        .try_into()
        .expect("Internal error");
    ret_val.set(is_absolute_js_bool);
}

pub fn join_paths<'a, 'b>(
    scope: &mut v8::HandleScope<'a>,
    args: v8::FunctionCallbackArguments,
    mut ret_val: v8::ReturnValue<'b>,
) {
    let base_js_str: v8::Local<v8::String> = args.get(0).try_into().expect("Expected a string.");
    let mut base = PathBuf::from_str(&base_js_str.to_rust_string_lossy(scope))
        .expect("Given string is not a path.");

    let path_js_str: v8::Local<v8::String> = args.get(1).try_into().expect("Expected a string.");
    let path = PathBuf::from_str(&path_js_str.to_rust_string_lossy(scope))
        .expect("Given string is not a path.");

    // TODO: replace this logic with std::path::absolute once it's stable.
    for path_component in path.components() {
        match path_component {
            Component::RootDir | Component::Prefix(_) => {
                ret_val.set(path_js_str.into());
                return;
            }
            Component::CurDir => continue,
            Component::Normal(component_str) => base.push(component_str),
            Component::ParentDir => {
                base.pop();
            }
        }
    }

    let joined_path = v8::String::new(
        scope,
        base.to_str().expect("Failed to convert path to a string."),
    )
    .expect("Internal error.");
    ret_val.set(joined_path.into());
}

fn add_builtin<'s>(
    scope: &mut v8::HandleScope<'s>,
    builtins_obj: &v8::Local<'s, v8::Object>,
    func_name: &str,
    func: impl v8::MapFnTo<v8::FunctionCallback>,
) {
    let func_name = v8::String::new(scope, func_name).expect("Unexpected internal error.");
    let function = v8::Function::new(scope, func).expect("Unexpected internal error.");
    builtins_obj.set(scope, func_name.into(), function.into());
}
