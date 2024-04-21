use std::collections::HashMap;

use super::{
    error::NixError,
    helpers::{is_nixrt_type, try_get_js_object_key},
};

#[derive(Debug, PartialEq)]
pub enum Value {
    AttrSet(HashMap<String, Value>),
    Bool(bool),
    Float(f64),
    Int(i64),
    Lambda,
    List(Vec<Value>),
    Path(String),
    Str(String),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NixTypeKind {
    Bool,
    Float,
    Int,
    List,
    Null,
    String,
    Path,
    Lambda,
    Set,
}

pub type EvalResult = Result<Value, NixError>;

pub fn js_value_to_nix(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> EvalResult {
    if js_value.is_function() {
        return Ok(Value::Lambda);
    }
    if let Some(value) = from_js_attrset(scope, nixrt, js_value)? {
        return Ok(value);
    }
    if let Some(value) = from_js_string(scope, nixrt, js_value)? {
        return Ok(value);
    }
    if let Some(value) = from_js_lazy(scope, nixrt, js_value)? {
        return Ok(value);
    }
    if let Some(value) = from_js_int(scope, nixrt, js_value)? {
        return Ok(value);
    }
    if let Some(value) = from_js_bool(scope, nixrt, js_value)? {
        return Ok(value);
    }
    if let Some(value) = from_js_float(scope, nixrt, js_value)? {
        return Ok(value);
    }
    if let Some(value) = from_js_list(scope, nixrt, js_value)? {
        return Ok(value);
    }
    if let Some(value) = from_js_path(scope, nixrt, js_value)? {
        return Ok(value);
    }
    if let Some(value) = from_js_lambda(scope, nixrt, js_value)? {
        return Ok(value);
    }
    todo!(
        "js_value_to_nix: {:?}",
        js_value.to_rust_string_lossy(scope),
    )
}

fn from_js_int(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> Result<Option<Value>, NixError> {
    if is_nixrt_type(scope, nixrt, js_value, "NixInt")? {
        let Some(int64_js_value) = try_get_js_object_key(scope, js_value, "int64")? else {
            return Ok(None);
        };
        let big_int_value: v8::Local<v8::BigInt> = int64_js_value.try_into().map_err(|err| {
            format!("Expected an int64 value. Internal conversion error: {err:?}")
        })?;
        return Ok(Some(Value::Int(big_int_value.i64_value().0)));
    }
    Ok(None)
}

fn from_js_string(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> Result<Option<Value>, NixError> {
    if is_nixrt_type(scope, nixrt, js_value, "NixString")? {
        let Some(value) = try_get_js_object_key(scope, js_value, "value")? else {
            return Ok(None);
        };
        let value_js_string: v8::Local<v8::String> = value.try_into().map_err(|err| {
            format!("Expected a string value. Internal conversion error: {err:?}")
        })?;
        return Ok(Some(Value::Str(
            value_js_string.to_rust_string_lossy(scope),
        )));
    }
    Ok(None)
}

fn from_js_lazy(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> Result<Option<Value>, NixError> {
    if is_nixrt_type(scope, nixrt, js_value, "Lazy")? {
        let to_strict = try_get_js_object_key(scope, js_value, "toStrict")?.ok_or_else(|| {
            "Internal error: could not find the `toStrict` method on the Lazy object.".to_string()
        })?;
        let to_strict_method: v8::Local<v8::Function> = to_strict.try_into().map_err(|err| {
            format!(
                "Expected `toStrict` to be a method on the Lazy object. Internal conversion error: {err:?}"
            )
        })?;
        let strict_value = to_strict_method
            .call(scope, *js_value, &[])
            .ok_or_else(|| "Could not convert the lazy value to strict.".to_string())?;
        return Ok(Some(js_value_to_nix(scope, nixrt, &strict_value)?));
    }
    Ok(None)
}

fn from_js_bool(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> Result<Option<Value>, NixError> {
    if is_nixrt_type(scope, nixrt, js_value, "NixBool")? {
        let value = try_get_js_object_key(scope, js_value, "value")?.ok_or_else(|| {
            "Internal error: could not find the `value` property on the NixBool object.".to_string()
        })?;
        let value_as_bool: v8::Local<v8::Boolean> = value.try_into().map_err(|err| {
            format!(
                "Expected `value` to be a boolean on the NixBool object. Internal conversion error: {err:?}"
            )
        })?;
        return Ok(Some(Value::Bool(value_as_bool.boolean_value(scope))));
    }
    Ok(None)
}

fn from_js_float(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> Result<Option<Value>, NixError> {
    if is_nixrt_type(scope, nixrt, js_value, "NixFloat")? {
        let value = try_get_js_object_key(scope, js_value, "value")?.ok_or_else(|| {
            "Internal error: could not find the `value` property on the NixFloat object."
                .to_string()
        })?;
        let value_as_number: v8::Local<v8::Number> = value.try_into().map_err(|err| {
            format!(
                "Expected `value` to be a number on the NixFloat object. Internal conversion error: {err:?}"
            )
        })?;
        return Ok(Some(Value::Float(
            value_as_number.number_value(scope).ok_or_else(|| {
                "Could not convert the JavaScript number to a floating point number.".to_string()
            })?,
        )));
    }
    Ok(None)
}

fn from_js_attrset(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> Result<Option<Value>, NixError> {
    if is_nixrt_type(scope, nixrt, js_value, "Attrset")? {
        let underlying_map_value = try_get_js_object_key(scope, js_value, "underlyingMap")?
            .ok_or_else(|| {
                "Internal error: could not find the `underlyingMap` method on the Attrset object."
                    .to_string()
            })?;
        let underlying_map_function: v8::Local<v8::Function> = underlying_map_value.try_into().map_err(|err| {
            format!(
                "Expected `underlyingMap` to be a method on the Attrset object. Internal conversion error: {err:?}"
            )
        })?;
        let underlying_map: v8::Local<v8::Map> = underlying_map_function
            .call(scope, *js_value, &[])
            .ok_or_else(|| "Could not get the underlying map of the Attrset.".to_string())?
            .try_into()
            .map_err(|err| {
                format!(
                    "Expected `underlyingMap` to return a Map. Internal conversion error: {err:?}"
                )
            })?;
        return Ok(Some(js_map_as_attrset(scope, nixrt, &underlying_map)?));
    }
    Ok(None)
}

fn from_js_list(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> Result<Option<Value>, NixError> {
    if is_nixrt_type(scope, nixrt, js_value, "NixList")? {
        let value = try_get_js_object_key(scope, js_value, "values")?.ok_or_else(|| {
            "Internal error: could not find the `values` property on the NixList object."
                .to_string()
        })?;
        let value_as_array: v8::Local<v8::Array> = value.try_into().map_err(|err| {
            format!(
                "Expected `values` to be an array in the NixList object. Internal conversion error: {err:?}"
            )
        })?;
        return Ok(Some(js_value_as_nix_array(scope, nixrt, &value_as_array)?));
    }
    Ok(None)
}

fn from_js_path(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> Result<Option<Value>, NixError> {
    if !is_nixrt_type(scope, nixrt, js_value, "Path")? {
        return Ok(None);
    }
    let Some(path) = try_get_js_object_key(scope, js_value, "path")? else {
        return Ok(None);
    };
    Ok(Some(Value::Path(path.to_rust_string_lossy(scope))))
}

fn from_js_lambda(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_value: &v8::Local<v8::Value>,
) -> Result<Option<Value>, NixError> {
    if !is_nixrt_type(scope, nixrt, js_value, "Lambda")? {
        return Ok(None);
    }
    Ok(Some(Value::Lambda))
}

fn js_value_as_nix_array(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_array: &v8::Local<v8::Array>,
) -> EvalResult {
    let length = js_array.length();
    let mut rust_array = Vec::with_capacity(length as usize);
    for idx in 0..length {
        let js_element = js_array.get_index(scope, idx).unwrap();
        match js_value_to_nix(scope, nixrt, &js_element) {
            Ok(value) => rust_array.push(value),
            err => return err,
        }
    }
    Ok(Value::List(rust_array))
}

fn js_map_as_attrset(
    scope: &mut v8::HandleScope<'_>,
    nixrt: &v8::Local<v8::Value>,
    js_map: &v8::Local<v8::Map>,
) -> EvalResult {
    let mut map: HashMap<String, Value> = HashMap::new();
    let js_map_array = js_map.as_array(scope);
    for idx in 0..js_map_array.length() / 2 {
        let key_idx = idx * 2;
        let value_idx = key_idx + 1;
        let key: v8::Local<v8::String> = js_map_array
            .get_index(scope, key_idx)
            .expect("Unexpected index out-of-bounds.")
            .try_into()
            .expect("Attr names must be strings.");
        let value = js_map_array
            .get_index(scope, value_idx)
            .expect("Unexpected index out-of-bounds.");
        map.insert(
            key.to_rust_string_lossy(scope),
            js_value_to_nix(scope, nixrt, &value)?,
        );
    }
    Ok(Value::AttrSet(map))
}
