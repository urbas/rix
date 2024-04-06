use std::env::current_dir;
use std::path::Path;
use std::sync::Once;

use v8::{HandleScope, Local, ModuleStatus, Object};

use crate::eval::types::EvalResult;

use super::emit_js::emit_module;
use super::helpers::{get_nixrt_type, try_get_js_object_key};
use super::types::js_value_to_nix;

static INIT_V8: Once = Once::new();

pub fn evaluate(nix_expr: &str) -> EvalResult {
    initialize_v8();
    // Declare the V8 execution context
    let isolate = &mut v8::Isolate::new(Default::default());
    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);
    let global = context.global(scope);

    let import_module_attr = v8::String::new(scope, "importNixModule").unwrap();
    let import_module_fn = v8::Function::new(scope, import_nix_module).unwrap();
    global
        .set(scope, import_module_attr.into(), import_module_fn.into())
        .unwrap();

    // Execute the Nix runtime JS module, get its exports
    let nixjs_rt_str = include_str!("../../nixjs-rt/dist/lib.mjs");
    let nixjs_rt_obj = exec_module(nixjs_rt_str, scope).unwrap();

    // Set them to a global variable
    let nixrt_attr = v8::String::new(scope, "n").unwrap();
    global
        .set(scope, nixrt_attr.into(), nixjs_rt_obj.into())
        .unwrap();

    let root_nix_fn = nix_expr_to_js_function(scope, nix_expr)?;

    nix_value_from_module(scope, root_nix_fn, nixjs_rt_obj)
}

fn nix_expr_to_js_function<'s>(
    scope: &mut HandleScope<'s>,
    nix_expr: &str,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let source_str = emit_module(nix_expr)?;
    let module_source_v8 = to_v8_source(scope, &source_str, "<eval string>");
    let module = v8::script_compiler::compile_module(scope, module_source_v8).unwrap();

    if module
        .instantiate_module(scope, resolve_module_callback)
        .is_none()
    {
        todo!("Instantiation failure.")
    }

    if module.evaluate(scope).is_none() {
        todo!("evaluation failed")
    };

    if module.get_status() == ModuleStatus::Errored {
        let exception = module.get_exception();
        let string = exception.to_rust_string_lossy(scope);

        todo!("evaluation failed:\n{}", string);
    }

    let namespace_obj = module.get_module_namespace().to_object(scope).unwrap();

    let Some(nix_value) = try_get_js_object_key(scope, &namespace_obj.into(), "default")? else {
        todo!(
            "Could not find the nix value: {:?}",
            namespace_obj.to_rust_string_lossy(scope)
        )
    };
    let nix_value: v8::Local<v8::Function> =
        nix_value.try_into().expect("Nix value must be a function.");

    Ok(nix_value)
}

fn import_nix_module<'s>(
    scope: &mut HandleScope<'s>,
    args: v8::FunctionCallbackArguments<'s>,
    mut ret: v8::ReturnValue,
) {
    let module_path = args.get(0).to_rust_string_lossy(scope);
    let module_source_str = std::fs::read_to_string(module_path).unwrap();

    let nix_fn = nix_expr_to_js_function(scope, &module_source_str);

    let nix_fn = match nix_fn {
        Ok(nix_fn) => nix_fn,
        Err(err) => {
            let err_str = v8::String::new(scope, &err).unwrap();
            let err_obj = v8::Exception::error(scope, err_str);
            ret.set(err_obj);
            return;
        }
    };

    ret.set(nix_fn.into());
}

fn exec_module<'a>(
    code: &str,
    scope: &mut v8::HandleScope<'a>,
) -> Result<Local<'a, Object>, String> {
    let source = to_v8_source(scope, code, "<eval string>");
    let module = v8::script_compiler::compile_module(scope, source).unwrap();

    if module
        .instantiate_module(scope, resolve_module_callback)
        .is_none()
    {
        return Err("Instantiation failure.".to_owned());
    }

    if module.evaluate(scope).is_none() {
        return Err("Evaluation failure.".to_owned());
    }

    let obj = module.get_module_namespace().to_object(scope).unwrap();

    Ok(obj)
}

fn initialize_v8() {
    INIT_V8.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}

fn nix_value_from_module(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    nix_value: v8::Local<v8::Function>,
    nixjs_rt_obj: v8::Local<v8::Object>,
) -> EvalResult {
    let nixrt: v8::Local<v8::Value> = nixjs_rt_obj.into();

    let eval_ctx = create_eval_ctx(
        scope,
        &nixrt,
        &current_dir().map_err(|err| {
            format!("Failed to determine the current working directory. Error: {err}")
        })?,
    )?;

    let nix_value = call_js_function(scope, &nix_value, &[eval_ctx.into()])?;

    let to_strict_fn: v8::Local<v8::Function> =
        try_get_js_object_key(scope, &nixrt, "recursiveStrict")?
            .expect("Could not find the function `recursiveStrict` in `nixrt`.")
            .try_into()
            .expect("`n.recursiveStrict` is not a function.");
    let strict_nix_value = call_js_function(scope, &to_strict_fn, &[nix_value])?;

    js_value_to_nix(scope, &nixrt, &strict_nix_value)
}

fn call_js_function<'s>(
    scope: &mut v8::ContextScope<'_, v8::HandleScope<'s>>,
    js_function: &v8::Local<v8::Function>,
    args: &[v8::Local<v8::Value>],
) -> Result<v8::Local<'s, v8::Value>, String> {
    let scope = &mut v8::TryCatch::new(scope);
    let recv = v8::undefined(scope).into();
    let Some(strict_nix_value) = js_function.call(scope, recv, args) else {
        // TODO: Again, the stack trace needs to be source-mapped. See TODO above.
        let err_msg = scope
            .stack_trace()
            .map_or("Unknown evaluation error.".to_owned(), |stack| {
                stack.to_rust_string_lossy(scope)
            });
        return Err(err_msg);
    };
    Ok(strict_nix_value)
}

fn create_eval_ctx<'s>(
    scope: &mut v8::HandleScope<'s>,
    nixrt: &v8::Local<v8::Value>,
    script_path: &Path,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let eval_ctx_type = get_nixrt_type(scope, nixrt, "EvalCtx")?;
    let eval_ctx_constructor: v8::Local<v8::Function> = eval_ctx_type
        .try_into()
        .expect("Could not get the constructor of the evaluation context class.");

    let real_path = script_path
        .canonicalize()
        .map_err(|err| format!("Failed to resolve the script path. Error: {err}."))?;
    let script_dir = real_path
        .parent()
        .ok_or_else(|| format!("Failed to determine the directory of path {real_path:?}."))?;
    let script_dir_str = real_path
        .to_str()
        .ok_or_else(|| format!("Failed to converft the path {script_dir:?} to a string."))?;
    let js_script_dir_path =
        v8::String::new(scope, script_dir_str).expect("Unexpected internal error.");

    Ok(eval_ctx_constructor
        .new_instance(scope, &[js_script_dir_path.into()])
        .expect("Could not construct the global evaluation context."))
}

fn new_script_origin<'s>(
    scope: &mut v8::HandleScope<'s>,
    resource_name: &str,
    source_map_url: &str,
) -> v8::ScriptOrigin<'s> {
    let resource_name_v8_str = v8::String::new(scope, resource_name).unwrap();
    let resource_line_offset = 0;
    let resource_column_offset = 0;
    let resource_is_shared_cross_origin = true;
    let script_id = 123;
    let source_map_url = v8::String::new(scope, source_map_url).unwrap();
    let resource_is_opaque = false;
    let is_wasm = false;
    let is_module = true;
    v8::ScriptOrigin::new(
        scope,
        resource_name_v8_str.into(),
        resource_line_offset,
        resource_column_offset,
        resource_is_shared_cross_origin,
        script_id,
        source_map_url.into(),
        resource_is_opaque,
        is_wasm,
        is_module,
    )
}

fn to_v8_source(
    scope: &mut v8::HandleScope,
    js_code: &str,
    source_path: &str,
) -> v8::script_compiler::Source {
    let code = v8::String::new(scope, js_code).unwrap();
    let origin = new_script_origin(scope, source_path, &format!("file://{source_path}.map"));
    v8::script_compiler::Source::new(code, Some(&origin))
}

fn resolve_module_callback<'a>(
    _: v8::Local<'a, v8::Context>,
    _: v8::Local<'a, v8::String>,
    _: v8::Local<'a, v8::FixedArray>,
    _: v8::Local<'a, v8::Module>,
) -> Option<v8::Local<'a, v8::Module>> {
    panic!("Module resolution not supported.")
}
