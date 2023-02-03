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
    PointerValue(Box<Type>, PointerValue<'a>),
    Void,
}

impl<'a> Value<'a> {
    pub fn get_type(&self) -> Type {
        match self {
            Value::U8Value(_) => Type::U8,
            Value::I32Value(_) => Type::I32,
            Value::U64Value(_) => Type::U64,
            Value::U32Value(_) => Type::U32,
            Value::USizeValue(_) => Type::USize,
            Value::PointerValue(pointer_of, _) => Type::Ptr(pointer_of.clone()),
            Value::Void => Type::Void,
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
            Value::PointerValue(_, _) => panic!(),
        }
    }
    pub fn is_integer(&self) -> bool {
        match self {
            Value::U8Value(_) => true,
            Value::I32Value(_) => true,
            Value::U32Value(_) => true,
            Value::U64Value(_) => true,
            Value::USizeValue(_) => true,
            Value::PointerValue(_, _) => false,
            Value::Void => false,
        }
    }
}
