use super::*;
use crate::{
    ast::BinaryOp,
    common::{binary::get_cast_type, target::PointerSizedIntWidth},
};

impl LLVMCodeGenerator<'_> {
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
            ResolvedType::StructLike(_) => unreachable!(),
            ResolvedType::Bool => unreachable!(),
        }
    }
    pub(super) fn eval_binary_expr(
        &self,
        binary_expr: &BinaryExpr,
    ) -> Result<BasicValueEnum, BuilderError> {
        let mut left = self.gen_expression(&binary_expr.lhs)?.unwrap();
        let mut right = self.gen_expression(&binary_expr.rhs)?.unwrap();

        let (lhs_cast_type, rhs_cast_type) = get_cast_type(
            if self.ptr_sized_int_type.get_bit_width() == 32 {
                PointerSizedIntWidth::ThirtyTwo
            } else {
                PointerSizedIntWidth::SixtyFour
            },
            &binary_expr.lhs.ty,
            &binary_expr.rhs.ty,
        );

        let mut result_type = ResolvedType::I32;
        if let Some(lhs_cast_type) = lhs_cast_type {
            left = self.gen_try_cast(left, &lhs_cast_type);
            result_type = lhs_cast_type;
        }
        if let Some(rhs_cast_type) = rhs_cast_type {
            right = self.gen_try_cast(right, &rhs_cast_type);
            result_type = rhs_cast_type;
        };

        let value = match binary_expr.op {
            BinaryOp::Add => {
                if result_type.is_integer_type() {
                    self.llvm_builder.build_int_add(
                        left.into_int_value(),
                        right.into_int_value(),
                        "",
                    )?
                } else {
                    unimplemented!()
                }
            }
            BinaryOp::Sub => {
                if result_type.is_integer_type() {
                    self.llvm_builder.build_int_sub(
                        left.into_int_value(),
                        right.into_int_value(),
                        "",
                    )?
                } else {
                    unimplemented!()
                }
            }
            BinaryOp::Mul => {
                if result_type.is_integer_type() {
                    self.llvm_builder.build_int_mul(
                        left.into_int_value(),
                        right.into_int_value(),
                        "",
                    )?
                } else {
                    unimplemented!()
                }
            }
            BinaryOp::Div => {
                if result_type.is_integer_type() {
                    if result_type.is_signed_integer_type() {
                        self.llvm_builder.build_int_signed_div(
                            left.into_int_value(),
                            right.into_int_value(),
                            "",
                        )?
                    } else {
                        self.llvm_builder.build_int_unsigned_div(
                            left.into_int_value(),
                            right.into_int_value(),
                            "",
                        )?
                    }
                } else {
                    unimplemented!()
                }
            }
            BinaryOp::Equals
            | BinaryOp::NotEquals
            | BinaryOp::LessThan
            | BinaryOp::LessThanOrEquals
            | BinaryOp::GreaterThan
            | BinaryOp::GreaterThanOrEquals => {
                if result_type.is_integer_type() {
                    let predicate = if result_type.is_signed_integer_type() {
                        match binary_expr.op {
                            BinaryOp::Equals => inkwell::IntPredicate::EQ,
                            BinaryOp::NotEquals => inkwell::IntPredicate::NE,
                            BinaryOp::LessThan => inkwell::IntPredicate::SLT,
                            BinaryOp::LessThanOrEquals => inkwell::IntPredicate::SLE,
                            BinaryOp::GreaterThan => inkwell::IntPredicate::SGT,
                            BinaryOp::GreaterThanOrEquals => inkwell::IntPredicate::SGE,
                            _ => unreachable!(),
                        }
                    } else {
                        match binary_expr.op {
                            BinaryOp::Equals => inkwell::IntPredicate::EQ,
                            BinaryOp::NotEquals => inkwell::IntPredicate::NE,
                            BinaryOp::LessThan => inkwell::IntPredicate::ULT,
                            BinaryOp::LessThanOrEquals => inkwell::IntPredicate::ULE,
                            BinaryOp::GreaterThan => inkwell::IntPredicate::UGT,
                            BinaryOp::GreaterThanOrEquals => inkwell::IntPredicate::UGE,
                            _ => unreachable!(),
                        }
                    };
                    self.llvm_builder.build_int_compare(
                        predicate,
                        left.into_int_value(),
                        right.into_int_value(),
                        "",
                    )?
                } else {
                    unimplemented!()
                }
            }
        };

        Ok(value.as_basic_value_enum())
    }
}
