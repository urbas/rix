use super::error::{js_error_to_rust, NixError};

pub fn is_nixrt_type<'s, T>(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<'s, T>,
    js_value: &v8::Local<v8::Value>,
    type_name: &str,
) -> Result<bool, String>
where
    v8::Local<'s, T>: Into<v8::Local<'s, v8::Value>>,
{
    let nixrt_type = get_nixrt_type(scope, &(*nixrt).into(), type_name)?;
    js_value.instance_of(scope, nixrt_type).ok_or_else(|| {
        format!(
            "Failed to check whether value '{}' is '{type_name}'.",
            js_value.to_rust_string_lossy(scope)
        )
    })
}

pub fn get_nixrt_type<'s>(
    scope: &mut v8::HandleScope<'s>,
    nixrt: &v8::Local<v8::Value>,
    type_name: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let nix_int_class_name = v8::String::new(scope, type_name).unwrap();
    nixrt
        .to_object(scope)
        .unwrap()
        .get(scope, nix_int_class_name.into())
        .unwrap()
        .to_object(scope)
        .ok_or_else(|| format!("Could not find the type {type_name}."))
}

pub fn try_get_js_object_key<'s>(
    scope: &mut v8::HandleScope<'s>,
    js_value: &v8::Local<v8::Value>,
    key: &str,
) -> Result<Option<v8::Local<'s, v8::Value>>, String> {
    let js_object = js_value
        .to_object(scope)
        .ok_or_else(|| "Not an object.".to_owned())?;
    let key_js_str = v8::String::new(scope, key).unwrap();
    Ok(js_object.get(scope, key_js_str.into()))
}

pub fn get_js_value_key<'s: 'v, 'v, T>(
    scope: &mut v8::HandleScope<'s>,
    js_value: &v8::Local<'v, T>,
    key: &str,
) -> Result<v8::Local<'v, v8::Value>, String>
where
    v8::Local<'v, T>: TryInto<v8::Local<'v, v8::Object>>,
{
    let js_object: v8::Local<'v, v8::Object> = (*js_value)
        .try_into()
        .map_err(|_| "Not an object.".to_owned())?;
    let key_js_str = v8::String::new(scope, key).unwrap();

    if let Some(value) = js_object.get(scope, key_js_str.into()) {
        Ok(value)
    } else {
        Err(format!("Expected key '{key}' not found."))
    }
}

pub fn call_js_function<'s>(
    scope: &mut v8::HandleScope<'s>,
    js_function: &v8::Local<v8::Function>,
    nixrt: v8::Local<v8::Object>,
    args: &[v8::Local<v8::Value>],
) -> Result<v8::Local<'s, v8::Value>, NixError> {
    let try_scope = &mut v8::TryCatch::new(scope);
    let this = v8::undefined(try_scope).into();
    let Some(strict_nix_value) = js_function.call(try_scope, this, args) else {
        let exception = try_scope.exception();
        return Err(map_js_exception_value_to_rust(try_scope, nixrt, exception));
    };
    Ok(strict_nix_value)
}

pub fn call_js_instance_mehod<'s>(
    scope: &mut v8::HandleScope<'s>,
    js_function: &v8::Local<v8::Function>,
    this: v8::Local<v8::Value>,
    nixrt: v8::Local<v8::Object>,
    args: &[v8::Local<v8::Value>],
) -> Result<v8::Local<'s, v8::Value>, NixError> {
    let try_scope = &mut v8::TryCatch::new(scope);
    let Some(strict_nix_value) = js_function.call(try_scope, this, args) else {
        let exception = try_scope.exception();
        return Err(map_js_exception_value_to_rust(try_scope, nixrt, exception));
    };
    Ok(strict_nix_value)
}

pub fn map_js_exception_value_to_rust<'s>(
    scope: &mut v8::HandleScope<'s>,
    nixrt: v8::Local<v8::Object>,
    exception: Option<v8::Local<'s, v8::Value>>,
) -> NixError {
    // TODO: Again, the stack trace needs to be source-mapped.
    if let Some(error) = exception {
        js_error_to_rust(scope, nixrt, error)
    } else {
        "Unknown evaluation error.".into()
    }
}
