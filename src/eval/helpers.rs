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
