use v8::Object;

pub fn is_nixrt_type(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
    type_name: &str,
) -> Result<bool, String> {
    let nixrt_type = get_nixrt_type(scope, nixrt, type_name)?;
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
