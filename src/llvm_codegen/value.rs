use inkwell::values::{BasicValueEnum, IntValue};

use crate::ast::Type;

#[derive(Debug, Clone, Copy)]
pub(super) enum Value<'a> {
    I32Value(IntValue<'a>),
    U64Value(IntValue<'a>),
    Void,
}

impl<'a> Value<'a> {
    pub fn get_primitive_type(&self) -> Option<Type> {
        match self {
            Value::I32Value(_) => Some(Type::I32),
            Value::U64Value(_) => Some(Type::U64),
            Value::Void => None,
        }
    }

    pub fn unwrap_int_value(self) -> IntValue<'a> {
        match self {
            Value::I32Value(v) => v,
            Value::U64Value(v) => v,
            Value::Void => panic!(),
        }
    }
}
