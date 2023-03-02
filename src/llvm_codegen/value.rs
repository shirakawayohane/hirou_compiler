use core::panic;

use inkwell::values::{IntValue, PointerValue};

use crate::ast::ResolvedType;

use super::{error::CompileError, LLVMCodegenerator};

#[derive(Debug, Clone)]
pub enum Value<'a> {
    U8Value(IntValue<'a>),
    I32Value(IntValue<'a>),
    U64Value(IntValue<'a>),
    U32Value(IntValue<'a>),
    USizeValue(IntValue<'a>),
    PointerValue(Box<ResolvedType>, PointerValue<'a>),
    Void,
}

impl<'a> Value<'a> {
    pub fn get_type(&self) -> ResolvedType {
        match self {
            Value::U8Value(_) => ResolvedType::U8,
            Value::I32Value(_) => ResolvedType::I32,
            Value::U64Value(_) => ResolvedType::U64,
            Value::U32Value(_) => ResolvedType::U32,
            Value::USizeValue(_) => ResolvedType::USize,
            Value::PointerValue(pointer_of, _) => ResolvedType::Ptr(pointer_of.clone()),
            Value::Void => ResolvedType::Void,
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
    pub fn unwrap_pointer_value(self) -> PointerValue<'a> {
        if let Value::PointerValue(_, ptr) = self {
            ptr
        } else {
            panic!()
        }
    }
}

impl LLVMCodegenerator<'_> {
    pub(crate) fn gen_try_cast<'ctx>(
        &'ctx self,
        value: Value<'ctx>,
        ty: &ResolvedType,
    ) -> Result<Value<'ctx>, CompileError> {
        Ok(match ty {
            ResolvedType::I32 => Value::I32Value(self.llvm_builder.build_int_cast_sign_flag(
                value.unwrap_int_value(),
                self.i32_type,
                true,
                "(i32)",
            )),
            ResolvedType::U32 => Value::U32Value(self.llvm_builder.build_int_cast_sign_flag(
                value.unwrap_int_value(),
                self.i32_type,
                false,
                "(u32)",
            )),
            ResolvedType::U64 => Value::U64Value(self.llvm_builder.build_int_cast_sign_flag(
                value.unwrap_int_value(),
                self.i64_type,
                false,
                "(u64)",
            )),
            ResolvedType::USize => Value::USizeValue(self.llvm_builder.build_int_cast_sign_flag(
                value.unwrap_int_value(),
                match self.pointer_size {
                    super::PointerSize::SixteenFour => self.i64_type,
                },
                false,
                "(u64)",
            )),
            ResolvedType::U8 => Value::U8Value(self.llvm_builder.build_int_cast_sign_flag(
                value.unwrap_int_value(),
                self.i8_type,
                false,
                "(u8)",
            )),
            ResolvedType::Ptr(_) => unimplemented!(),
            ResolvedType::Void => unimplemented!(),
        })
    }
}
