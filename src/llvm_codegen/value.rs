use core::panic;

use inkwell::values::{IntValue, PointerValue};

use crate::ast::Type;

#[derive(Debug, Clone)]
pub(super) enum Value<'a> {
    U8Value(IntValue<'a>),
    I32Value(IntValue<'a>),
    U64Value(IntValue<'a>),
    U32Value(IntValue<'a>),
    USizeValue(IntValue<'a>),
    PointerValue(PointerValue<'a>),
    Void,
}

impl<'a> Value<'a> {
    pub fn get_primitive_type(&self) -> Option<Type> {
        match self {
            Value::U8Value(_) => Some(Type::U8),
            Value::I32Value(_) => Some(Type::I32),
            Value::U32Value(_) => Some(Type::U32),
            Value::U64Value(_) => Some(Type::U64),
            Value::USizeValue(_) => Some(Type::USize),
            Value::Void => None,
            Value::PointerValue(_) => None,
        }
    }

    pub fn unwrap_int_value(self) -> IntValue<'a> {
        match self {
            Value::U8Value(v) => v,
            Value::I32Value(v) => v,
            Value::U32Value(v) => v,
            Value::U64Value(v) => v,
            Value::USizeValue(v) => v,
            Value::Void => panic!(),
            Value::PointerValue(_) => panic!()
        }
    }
    pub fn is_integer(&self) -> bool {
        match self {
            Value::U8Value(_) => true,
            Value::I32Value(_) => true,
            Value::U32Value(_) => true,
            Value::U64Value(_) => true,
            Value::USizeValue(_) => true,
            Value::PointerValue(_) => false,
            Value::Void => false,
        }
    }
}
