use super::*;
use crate::{ast::BinaryOp, common::binary::get_cast_type};

impl LLVMCodeGenerator<'_> {
    pub(crate) fn gen_try_cast<'ctx>(
        &'ctx self,
        value: BasicValueEnum<'ctx>,
        ty: &ConcreteType,
    ) -> BasicValueEnum<'ctx> {
        match ty {
            ConcreteType::I32 => self
                .llvm_builder
                .build_int_cast_sign_flag(value.into_int_value(), self.llvm_context.i32_type(), true, "(i32)")
                .unwrap()
                .as_basic_value_enum(),
            ConcreteType::U32 => self
                .llvm_builder
                .build_int_cast_sign_flag(value.into_int_value(), self.llvm_context.i32_type(), false, "(u32)")
                .unwrap()
                .as_basic_value_enum(),
            ConcreteType::U64 => self
                .llvm_builder
                .build_int_cast_sign_flag(value.into_int_value(), self.llvm_context.i64_type(), false, "(u64)")
                .unwrap()
                .as_basic_value_enum(),
            ConcreteType::U8 => self
                .llvm_builder
                .build_int_cast_sign_flag(value.into_int_value(), self.llvm_context.i8_type(), false, "(u8)")
                .unwrap()
                .as_basic_value_enum(),
            ConcreteType::I64 => self
                .llvm_builder
                .build_int_cast(value.into_int_value(), self.llvm_context.i64_type(), "(i64)")
                .unwrap()
                .as_basic_value_enum(),
            ConcreteType::F32 => self
                .llvm_builder
                .build_float_cast(value.into_float_value(), self.llvm_context.f32_type(), "(f32)")
                .unwrap()
                .as_basic_value_enum(),
            ConcreteType::F64 => self
                .llvm_builder
                .build_float_cast(value.into_float_value(), self.llvm_context.f64_type(), "(f64)")
                .unwrap()
                .as_basic_value_enum(),
            ConcreteType::Ptr(_) => unreachable!(),
            ConcreteType::Void => unreachable!(),
            ConcreteType::StructLike(_) => unreachable!(),
            ConcreteType::Bool => unreachable!(),
        }
    }
    pub(super) fn eval_binary_expr(
        &self,
        binary_expr: &BinaryExpr,
    ) -> Result<BasicValueEnum, BuilderError> {
        let mut left = self.gen_expression(&binary_expr.lhs)?.unwrap();
        let mut right = self.gen_expression(&binary_expr.rhs)?.unwrap();

        let (lhs_cast_type, rhs_cast_type) =
            get_cast_type(&binary_expr.lhs.ty, &binary_expr.rhs.ty);

        let mut result_type = ConcreteType::I32;
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
                match result_type {
                    ConcreteType::I32
                    | ConcreteType::U8
                    | ConcreteType::U32
                    | ConcreteType::I64
                    | ConcreteType::U64 => {}
                    _ => unimplemented!(),
                }
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
