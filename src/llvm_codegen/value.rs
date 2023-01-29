use inkwell::values::IntValue;

use crate::ast::Type;

use super::LLVMCodegenerator;

#[derive(Debug, Clone, Copy)]
pub(super) enum Value<'a> {
    U8Value(IntValue<'a>),
    I32Value(IntValue<'a>),
    U64Value(IntValue<'a>),
    U32Value(IntValue<'a>),
    USizeValue(IntValue<'a>),
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
        }
    }
    pub fn is_integer(self) -> bool {
        match self {
            Value::U8Value(_) => true,
            Value::I32Value(_) => true,
            Value::U32Value(_) => true,
            Value::U64Value(_) => true,
            Value::USizeValue(_) => true,
            Value::Void => false,
        }
    }
}

impl<'a> LLVMCodegenerator<'a> {
    pub(super) fn cast_primitive_value(&'a self, value: Value<'a>, ty: &Type) -> Value<'a> {
        assert!(ty.is_primitive());
        if value.is_integer() {
            let int_value = value.unwrap_int_value();
            match ty {
                Type::I32 => Value::I32Value(self.llvm_builder.build_int_cast_sign_flag(
                    int_value,
                    self.i32_type,
                    true,
                    "(i32)",
                )),
                Type::U32 => Value::U32Value(self.llvm_builder.build_int_cast_sign_flag(
                    int_value,
                    self.i32_type,
                    false,
                    "(u32)",
                )),
                Type::U64 => Value::U64Value(self.llvm_builder.build_int_cast_sign_flag(
                    int_value,
                    self.i64_type,
                    false,
                    "(u64)",
                )),
                Type::USize => Value::USizeValue(self.llvm_builder.build_int_cast_sign_flag(
                    int_value,
                    match self.pointer_size {
                        super::PointerSize::SixteenFour => self.i64_type,
                        super::PointerSize::ThirtyTwo => self.i32_type,
                    },
                    false,
                    "(u64)",
                )),
                Type::U8 => Value::U8Value(self.llvm_builder.build_int_cast_sign_flag(
                    int_value,
                    self.i8_type,
                    false,
                    "(u8)",
                )),
                Type::Ptr(_) => unreachable!(),
            }
        } else {
            unimplemented!()
        }
    }
}
