use std::env::current_dir;
use std::path::Path;
use std::sync::Once;

use v8::ModuleStatus;

use crate::eval::types::EvalResult;

use super::emit_js::emit_expr;
use super::helpers::{get_nixrt_type, try_get_js_object_key};
use super::types::js_value_to_nix;

static INIT_V8: Once = Once::new();

pub fn evaluate(nix_expr: &str) -> EvalResult {
    initialize_v8();
    let isolate = &mut v8::Isolate::new(Default::default());
    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);

    let source_str = emit_module(nix_expr)?;
    let source_v8 = to_v8_source(scope, &source_str, "<eval string>");
    let module = v8::script_compiler::compile_module(scope, source_v8).unwrap();

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
    nix_value_from_module(scope, &namespace_obj)
}

fn initialize_v8() {
    INIT_V8.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}

pub fn emit_module(nix_expr: &str) -> Result<String, String> {
    let root = rnix::Root::parse(nix_expr).tree();
    let root_expr = root.expr().expect("Not implemented");
    let nixrt_js_module = env!("RIX_NIXRT_JS_MODULE");
    let mut out_src = format!("import n from '{nixrt_js_module}';\n");
    out_src += "export const __nixrt = n;\n";
    out_src += "export const __nixValue = (ctx) => ";
    emit_expr(&root_expr, &mut out_src)?;
    out_src += ";\n";
    Ok(out_src)
}

fn nix_value_from_module(
    scope: &mut v8::ContextScope<v8::HandleScope>,
    namespace_obj: &v8::Local<v8::Object>,
) -> EvalResult {
    let nix_value_attr = v8::String::new(scope, "__nixValue").unwrap();
    let Some(nix_value) = namespace_obj.get(scope, nix_value_attr.into()) else {
        todo!(
            "Could not find the nix value: {:?}",
            namespace_obj.to_rust_string_lossy(scope)
        )
    };
    let nix_value: v8::Local<v8::Function> =
        nix_value.try_into().expect("Nix value must be a function.");

    let nixrt_attr = v8::String::new(scope, "__nixrt").unwrap();
    let nixrt: v8::Local<v8::Value> = namespace_obj.get(scope, nixrt_attr.into()).unwrap();

    let eval_ctx = create_eval_ctx(
        scope,
        &nixrt,
        &current_dir().map_err(|err| {
            format!("Failed to determine the current working directory. Error: {err}")
        })?,
    )?;

    let nix_value = call_js_function(scope, &nix_value, &[eval_ctx.into()])?;

    let to_strict_fn: v8::Local<v8::Function> = try_get_js_object_key(scope, &nixrt, "toStrict")?
        .expect("Could not find the function `toStrict` in `nixrt`.")
        .try_into()
        .expect("`n.toStrict` is not a function.");
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
    context: v8::Local<'a, v8::Context>,
    specifier: v8::Local<'a, v8::String>,
    _import_assertions: v8::Local<'a, v8::FixedArray>,
    _referrer: v8::Local<'a, v8::Module>,
) -> Option<v8::Local<'a, v8::Module>> {
    let scope = &mut unsafe { v8::CallbackScope::new(context) };
    let module_path = specifier.to_rust_string_lossy(scope);
    let module_source_str = std::fs::read_to_string(&module_path).unwrap();
    let module_source_v8 = to_v8_source(scope, &module_source_str, &module_path);
    v8::script_compiler::compile_module(scope, module_source_v8)
}
