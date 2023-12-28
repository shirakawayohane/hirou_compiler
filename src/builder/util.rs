use inkwell::values::{BasicValue, BasicValueEnum};

use crate::resolved_ast::ResolvedType;

use super::LLVMCodeGenerator;

impl LLVMCodeGenerator<'_> {
    pub(super) fn get_cast_type<'a>(
        &self,
        lhs: &'a ResolvedType,
        rhs: &'a ResolvedType,
    ) -> (Option<ResolvedType>, Option<ResolvedType>) {
        let ptr_sized_int_type = self.ptr_sized_int_type;
        match lhs {
            ResolvedType::I32 => match rhs {
                ResolvedType::I32 => (None, None),
                ResolvedType::I64 => (Some(ResolvedType::I64), None),
                ResolvedType::U32 => (None, Some(ResolvedType::I32)),
                ResolvedType::U64 => (Some(ResolvedType::I64), Some(ResolvedType::I64)),
                ResolvedType::USize => (Some(ResolvedType::I64), Some(ResolvedType::I64)),
                ResolvedType::U8 => (None, Some(ResolvedType::I32)),
                _ => panic!("Invalid type for binary expression"),
            },
            ResolvedType::I64 => match rhs {
                ResolvedType::I32 => (None, Some(ResolvedType::I64)),
                ResolvedType::I64 => (None, None),
                ResolvedType::U32 => (None, Some(ResolvedType::I64)),
                ResolvedType::U64 => (None, Some(ResolvedType::I64)),
                ResolvedType::USize => (None, Some(ResolvedType::I64)),
                ResolvedType::U8 => (None, Some(ResolvedType::I64)),
                _ => panic!("Invalid type for binary expression"),
            },
            ResolvedType::U32 => match rhs {
                ResolvedType::I32 => (Some(ResolvedType::I32), None),
                ResolvedType::I64 => (Some(ResolvedType::I64), None),
                ResolvedType::U32 => (None, None),
                ResolvedType::U64 => (Some(ResolvedType::U64), None),
                ResolvedType::USize => {
                    if ptr_sized_int_type.get_bit_width() == 32 {
                        (None, None)
                    } else {
                        (Some(ResolvedType::USize), None)
                    }
                }
                ResolvedType::U8 => (Some(ResolvedType::U32), None),
                _ => panic!("Invalid type for binary expression"),
            },
            ResolvedType::U64 => match rhs {
                ResolvedType::I32 => (Some(ResolvedType::I64), None),
                ResolvedType::I64 => (Some(ResolvedType::I64), None),
                ResolvedType::U32 => (None, Some(ResolvedType::U64)),
                ResolvedType::U64 => (None, None),
                ResolvedType::USize => {
                    if ptr_sized_int_type.get_bit_width() == 32 {
                        (None, Some(ResolvedType::U64))
                    } else {
                        (None, None)
                    }
                }
                _ => panic!("Invalid type for binary expression"),
            },
            ResolvedType::USize => match rhs {
                ResolvedType::I32 => (Some(ResolvedType::I32), None),
                ResolvedType::I64 => (Some(ResolvedType::I64), None),
                ResolvedType::U32 => {
                    if ptr_sized_int_type.get_bit_width() == 32 {
                        (None, None)
                    } else {
                        (Some(ResolvedType::U64), None)
                    }
                }
                ResolvedType::U64 => {
                    if ptr_sized_int_type.get_bit_width() == 32 {
                        (Some(ResolvedType::U64), None)
                    } else {
                        (None, None)
                    }
                }
                ResolvedType::USize => (None, None),
                ResolvedType::U8 => {
                    if ptr_sized_int_type.get_bit_width() == 32 {
                        (Some(ResolvedType::U32), None)
                    } else {
                        (Some(ResolvedType::U64), None)
                    }
                }
                _ => panic!("Invalid type for binary expression"),
            },
            ResolvedType::U8 => match rhs {
                ResolvedType::I32 => (Some(ResolvedType::I32), None),
                ResolvedType::I64 => (Some(ResolvedType::I64), None),
                ResolvedType::U32 => (Some(ResolvedType::U32), None),
                ResolvedType::U64 => (Some(ResolvedType::U64), None),
                ResolvedType::USize => {
                    if ptr_sized_int_type.get_bit_width() == 32 {
                        (Some(ResolvedType::U32), None)
                    } else {
                        (Some(ResolvedType::U64), None)
                    }
                }
                ResolvedType::U8 => (None, None),
                _ => panic!("Invalid type for binary expression"),
            },
            ResolvedType::Ptr(_) => panic!("Invalid type for binary expression"),
            ResolvedType::Void => panic!("Invalid type for binary expression"),
            ResolvedType::Unknown => panic!("Invalid type for binary expression"),
            ResolvedType::Struct(_) => panic!("Invalid type for binary expression"),
        }
    }

    pub(crate) fn gen_try_cast<'ctx>(
        &'ctx self,
        value: BasicValueEnum<'ctx>,
        ty: &ResolvedType,
    ) -> BasicValueEnum<'ctx> {
        dbg!(ty, value);
        let value = value.into_int_value();
        match ty {
            ResolvedType::I32 => self
                .llvm_builder
                .build_int_cast_sign_flag(value, self.llvm_context.i32_type(), true, "(i32)")
                .unwrap()
                .as_basic_value_enum(),
            ResolvedType::U32 => self
                .llvm_builder
                .build_int_cast_sign_flag(value, self.llvm_context.i32_type(), false, "(u32)")
                .unwrap()
                .as_basic_value_enum(),
            ResolvedType::U64 => self
                .llvm_builder
                .build_int_cast_sign_flag(value, self.llvm_context.i64_type(), false, "(u64)")
                .unwrap()
                .as_basic_value_enum(),
            ResolvedType::U8 => self
                .llvm_builder
                .build_int_cast_sign_flag(value, self.llvm_context.i8_type(), false, "(u8)")
                .unwrap()
                .as_basic_value_enum(),
            ResolvedType::I64 => self
                .llvm_builder
                .build_int_cast(value, self.llvm_context.i64_type(), "(i64)")
                .unwrap()
                .as_basic_value_enum(),
            ResolvedType::Ptr(_) => unreachable!(),
            ResolvedType::Void => unreachable!(),
            ResolvedType::USize => self
                .llvm_builder
                .build_int_cast(value, self.ptr_sized_int_type, "(usize)")
                .unwrap()
                .as_basic_value_enum(),
            ResolvedType::Unknown => unreachable!(),
            ResolvedType::Struct(_) => unreachable!(),
        }
    }
}
