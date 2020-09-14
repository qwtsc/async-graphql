use crate::{GQLScalar, InputValueError, InputValueResult, ScalarType, Value};

/// The `Char` scalar type represents a unicode char.
/// The input and output values are a string, and there can only be one unicode character in this string.
#[GQLScalar(internal)]
impl ScalarType for char {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => {
                let mut chars = s.chars();
                match chars.next() {
                    Some(ch) if chars.next() == None => Ok(ch),
                    Some(_) => Err(InputValueError::Custom(
                        "There can only be one unicode character in the string.".into(),
                    )),
                    None => Err(InputValueError::Custom(
                        "A unicode character is required.".into(),
                    )),
                }
            }
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        match value {
            Value::String(_) => true,
            _ => false,
        }
    }

    fn to_value(&self) -> Value {
        Value::String((*self).into())
    }
}
