use super::helpers::{call_js_function, get_js_value_key, is_nixrt_type};

#[derive(Debug)]
pub struct NixError {
    pub message: Vec<NixErrorMessagePart>,
    pub kind: NixErrorKind,
}

impl std::fmt::Display for NixError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for part in &self.message {
            match part {
                NixErrorMessagePart::Plain(text) => write!(f, "{}", text)?,
                NixErrorMessagePart::Highlighted(text) => write!(f, "{}", text)?,
            }
        }

        Ok(())
    }
}

impl From<String> for NixError {
    fn from(message: String) -> Self {
        NixError {
            message: vec![NixErrorMessagePart::Plain(message.clone())],
            kind: NixErrorKind::UnexpectedRustError { message },
        }
    }
}

impl<'a> From<&'a str> for NixError {
    fn from(message: &'a str) -> Self {
        NixError {
            message: vec![NixErrorMessagePart::Plain(message.to_string())],
            kind: NixErrorKind::UnexpectedRustError {
                message: message.to_string(),
            },
        }
    }
}

impl From<v8::DataError> for NixError {
    fn from(error: v8::DataError) -> Self {
        NixError {
            message: vec![NixErrorMessagePart::Plain(error.to_string())],
            kind: NixErrorKind::UnexpectedJsError {
                message: error.to_string(),
            },
        }
    }
}

#[derive(Debug)]
pub enum NixErrorMessagePart {
    Plain(String),
    Highlighted(String),
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

#[derive(Debug, PartialEq, Eq)]
pub enum NixErrorKind {
    Abort {
        message: String,
    },
    CouldntFindVariable {
        var_name: String,
    },
    TypeMismatch {
        expected: Vec<NixTypeKind>,
        got: NixTypeKind,
    },
    Other {
        message: String,
    },
    MissingAttribute {
        attr_path: Vec<String>,
    },
    AttributeAlreadyDefined {
        attr_path: Vec<String>,
    },
    FunctionCallWithoutArgument {
        argument: String,
    },

    // For non-nix errors thrown in js or rust
    UnexpectedJsError {
        message: String,
    },
    UnexpectedRustError {
        message: String,
    },
}

pub fn js_error_to_rust(
    scope: &mut v8::HandleScope,
    nixrt: v8::Local<v8::Object>,
    error: v8::Local<v8::Value>,
) -> NixError {
    let result = try_js_error_to_rust(scope, nixrt, error);

    match result {
        Ok(ok) => ok,
        Err(err) => err,
    }
}

fn try_js_error_to_rust(
    scope: &mut v8::HandleScope,
    nixrt: v8::Local<v8::Object>,
    error: v8::Local<v8::Value>,
) -> Result<NixError, NixError> {
    // If the error is not a NixError instance, then it's an unexpected error.
    if !is_nixrt_type(scope, &nixrt, &error, "NixError")? {
        return Ok(NixError {
            message: vec![NixErrorMessagePart::Plain("An error occurred.".to_owned())],
            kind: NixErrorKind::UnexpectedJsError {
                message: error.to_rust_string_lossy(scope),
            },
        });
    }

    let message_js = get_js_value_key(scope, &error, "richMessage")?;
    let message = js_error_message_to_rust(scope, message_js)?;

    let kind_js = get_js_value_key(scope, &error, "kind")?;

    let kind = if is_nixrt_type(scope, &nixrt, &kind_js, "NixAbortError")? {
        let message_js = get_js_value_key(scope, &kind_js, "message")?;
        let message = message_js.to_rust_string_lossy(scope);
        NixErrorKind::Abort { message }
    } else if is_nixrt_type(scope, &nixrt, &kind_js, "NixCouldntFindVariableError")? {
        let var_name_js = get_js_value_key(scope, &kind_js, "varName")?;
        let var_name = var_name_js.to_rust_string_lossy(scope);
        NixErrorKind::CouldntFindVariable { var_name }
    } else if is_nixrt_type(scope, &nixrt, &kind_js, "NixTypeMismatchError")? {
        let expected_js = get_js_value_key(scope, &kind_js, "expected")?;
        let got_js = get_js_value_key(scope, &kind_js, "got")?;

        let mut expected = nix_type_class_array_to_enum_array(scope, nixrt, expected_js)?;
        let got = nix_type_class_to_enum(scope, nixrt, got_js)?;

        // Sort expected array, for normalization
        expected.sort_unstable();

        NixErrorKind::TypeMismatch { expected, got }
    } else if is_nixrt_type(scope, &nixrt, &kind_js, "NixOtherError")? {
        let message_js = get_js_value_key(scope, &kind_js, "message")?;
        let message = message_js.to_rust_string_lossy(scope);
        NixErrorKind::Other { message }
    } else if is_nixrt_type(scope, &nixrt, &kind_js, "NixMissingAttributeError")? {
        let attr_path_js = get_js_value_key(scope, &kind_js, "attrPath")?;
        let attr_path = js_string_array_to_rust_string_array(scope, attr_path_js)?;
        NixErrorKind::MissingAttribute { attr_path }
    } else if is_nixrt_type(scope, &nixrt, &kind_js, "NixAttributeAlreadyDefinedError")? {
        let attr_path_js = get_js_value_key(scope, &kind_js, "attrPath")?;
        let attr_path = js_string_array_to_rust_string_array(scope, attr_path_js)?;
        NixErrorKind::AttributeAlreadyDefined { attr_path }
    } else if is_nixrt_type(
        scope,
        &nixrt,
        &kind_js,
        "NixFunctionCallWithoutArgumentError",
    )? {
        let argument_js = get_js_value_key(scope, &kind_js, "argument")?;
        let argument = argument_js.to_rust_string_lossy(scope);
        NixErrorKind::FunctionCallWithoutArgument { argument }
    } else {
        return Ok(NixError {
            message: vec![NixErrorMessagePart::Plain("An error occurred.".to_owned())],
            kind: NixErrorKind::UnexpectedJsError {
                message: error.to_rust_string_lossy(scope),
            },
        });
    };

    Ok(NixError { message, kind })
}

fn nix_type_class_to_enum(
    scope: &mut v8::HandleScope,
    nixrt: v8::Local<v8::Object>,
    class: v8::Local<v8::Value>,
) -> Result<NixTypeKind, NixError> {
    let name_fn = get_js_value_key(scope, &class, "toTypeName")?.try_into()?;
    let name_js_str = call_js_function(scope, &name_fn, nixrt, &[])?;
    let name = name_js_str.to_rust_string_lossy(scope);

    match name.as_str() {
        "bool" => Ok(NixTypeKind::Bool),
        "float" => Ok(NixTypeKind::Float),
        "int" => Ok(NixTypeKind::Int),
        "list" => Ok(NixTypeKind::List),
        "null" => Ok(NixTypeKind::Null),
        "string" => Ok(NixTypeKind::String),
        "path" => Ok(NixTypeKind::Path),
        "lambda" => Ok(NixTypeKind::Lambda),
        "set" => Ok(NixTypeKind::Set),
        _ => Err(NixError {
            message: vec![NixErrorMessagePart::Plain(format!(
                "Unexpected type name: {name}"
            ))],
            kind: NixErrorKind::Other {
                message: format!("Unexpected type name: {name}"),
            },
        }),
    }
}

fn nix_type_class_array_to_enum_array(
    scope: &mut v8::HandleScope,
    nixrt: v8::Local<v8::Object>,
    class_array: v8::Local<v8::Value>,
) -> Result<Vec<NixTypeKind>, NixError> {
    let class_array: v8::Local<v8::Array> = class_array.try_into()?;

    let len_num: v8::Local<v8::Number> =
        get_js_value_key(scope, &class_array, "length")?.try_into()?;
    let len = len_num.value() as u32;

    let mut result = Vec::with_capacity(len as usize);

    for i in 0..len {
        let item_class = class_array
            .get_index(scope, i)
            .ok_or_else(|| format!("Expected index {i} not found."))?;

        let kind = nix_type_class_to_enum(scope, nixrt, item_class)?;
        result.push(kind);
    }

    Ok(result)
}

fn js_string_array_to_rust_string_array(
    scope: &mut v8::HandleScope,
    js_array: v8::Local<v8::Value>,
) -> Result<Vec<String>, NixError> {
    let js_array: v8::Local<v8::Array> = js_array.try_into()?;

    let len_num: v8::Local<v8::Number> =
        get_js_value_key(scope, &js_array, "length")?.try_into()?;
    let len = len_num.value() as u32;

    let mut result = Vec::with_capacity(len as usize);

    for i in 0..len {
        let item_js = js_array
            .get_index(scope, i)
            .ok_or_else(|| format!("Expected index {i} not found."))?;

        let item = item_js.to_rust_string_lossy(scope);
        result.push(item);
    }

    Ok(result)
}

fn js_error_message_part_to_rust(
    scope: &mut v8::HandleScope,
    error_part: v8::Local<v8::Value>,
) -> Result<NixErrorMessagePart, NixError> {
    let kind_js = get_js_value_key(scope, &error_part, "kind")?;
    let kind = kind_js.to_rust_string_lossy(scope);

    match kind.as_str() {
        "plain" => {
            let value_js = get_js_value_key(scope, &error_part, "value")?;
            let value = value_js.to_rust_string_lossy(scope);
            Ok(NixErrorMessagePart::Plain(value))
        }
        "highlighted" => {
            let value_js = get_js_value_key(scope, &error_part, "value")?;
            let value = value_js.to_rust_string_lossy(scope);
            Ok(NixErrorMessagePart::Highlighted(value))
        }
        _ => Err("Unexpected error message part kind.".into()),
    }
}

fn js_error_message_to_rust(
    scope: &mut v8::HandleScope,
    error: v8::Local<v8::Value>,
) -> Result<Vec<NixErrorMessagePart>, NixError> {
    let error: v8::Local<v8::Array> = error.try_into()?;

    let len_num: v8::Local<v8::Number> = get_js_value_key(scope, &error, "length")?.try_into()?;
    let len = len_num.value() as u32;

    let mut result = Vec::with_capacity(len as usize);

    for i in 0..len {
        let error_part = error
            .get_index(scope, i)
            .ok_or_else(|| format!("Expected index {i} not found."))?;

        let part = js_error_message_part_to_rust(scope, error_part)?;
        result.push(part);
    }

    Ok(result)
}
