use inkwell::{
    builder::BuilderError,
    types::BasicType,
    values::{BasicMetadataValueEnum, BasicValue, BasicValueEnum},
};

use super::*;
use crate::{ast::BinaryOp, resolved_ast::*};

impl LLVMCodeGenerator<'_> {
    fn eval_u8(&self, value_str: &str) -> BasicValueEnum {
        let n = value_str.parse::<u8>().unwrap();
        let int_value = self.llvm_context.i8_type().const_int(n as u64, true);
        int_value.into()
    }
    fn eval_i32(&self, value_str: &str) -> BasicValueEnum {
        let n = value_str.parse::<i32>().unwrap();
        let int_value = self.llvm_context.i32_type().const_int(n as u64, true);
        int_value.into()
    }
    fn eval_i64(&self, value_str: &str) -> BasicValueEnum {
        let n = value_str.parse::<i64>().unwrap();
        let int_value = self.llvm_context.i64_type().const_int(n as u64, true);
        int_value.into()
    }
    fn eval_u32(&self, value_str: &str) -> BasicValueEnum {
        let n = value_str.parse::<u32>().unwrap();
        let int_value = self.llvm_context.i32_type().const_int(n as u64, true);
        int_value.into()
    }
    fn eval_u64(&self, value_str: &str) -> BasicValueEnum {
        let n = value_str.parse::<u64>().unwrap();
        let int_value = self.llvm_context.i64_type().const_int(n, true);
        int_value.into()
    }
    fn eval_usize(&self, value_str: &str) -> BasicValueEnum {
        let n = value_str.parse::<usize>().unwrap();
        let int_value = self.ptr_sized_int_type.const_int(n as u64, true);
        int_value.into()
    }
    fn eval_number_literal(
        &self,
        integer_literal: &NumberLiteral,
        ty: &ResolvedType,
    ) -> Result<BasicValueEnum, BuilderError> {
        let value_str = &integer_literal.value;
        Ok(match ty {
            ResolvedType::U8 => self.eval_u8(value_str),
            ResolvedType::U32 => self.eval_u32(value_str),
            ResolvedType::I32 => self.eval_i32(value_str),
            ResolvedType::I64 => self.eval_i64(value_str),
            ResolvedType::U64 => self.eval_u64(value_str),
            ResolvedType::USize => self.eval_usize(value_str),
            ResolvedType::Ptr(_) => unreachable!(),
            ResolvedType::Void => unreachable!(),
            ResolvedType::Unknown => unreachable!(),
            ResolvedType::Struct(_) => unreachable!(),
        })
    }
    fn eval_string_literal(
        &self,
        string_literal: &StringLiteral,
    ) -> Result<BasicValueEnum, BuilderError> {
        let value = string_literal.value.as_str();
        let string = self
            .llvm_builder
            .build_global_string_ptr(value, "string_literal")?;
        Ok(string.as_basic_value_enum())
    }
    fn eval_struct_literal(
        &self,
        struct_literal: &StructLiteral,
    ) -> Result<BasicValueEnum, BuilderError> {
        let mut values = Vec::new();
        for field in &struct_literal.fields {
            let value = self.gen_expression(field)?.unwrap();
            values.push(value);
        }
        let struct_type = self.llvm_context.struct_type(
            values
                .iter()
                .map(|x| x.get_type())
                .collect::<Vec<_>>()
                .as_ref(),
            false,
        );
        let struct_value = struct_type.const_named_struct(&values);
        Ok(struct_value.as_basic_value_enum())
    }
    fn eval_variable_ref(
        &self,
        variable_ref: &VariableRefExpr,
        ty: &ResolvedType,
    ) -> BasicValueEnum {
        let ptr = self.get_variable(&variable_ref.name);
        let pointee_ty = self.type_to_basic_type_enum(ty).unwrap();
        self.llvm_builder.build_load(pointee_ty, ptr, "").unwrap()
    }
    fn eval_index_access(
        &self,
        index_access: &IndexAccessExor,
        ty: &ResolvedType,
    ) -> Result<BasicValueEnum, BuilderError> {
        let ptr = self.gen_expression(&index_access.target)?.unwrap();
        let pointee_ty = self
            .type_to_basic_type_enum(ty)
            .unwrap_or(self.type_to_basic_type_enum(&ResolvedType::U8).unwrap());
        let index = self.gen_expression(&index_access.index)?.unwrap();
        let ptr = unsafe {
            self.llvm_builder.build_in_bounds_gep(
                pointee_ty,
                ptr.into_pointer_value(),
                &[index.into_int_value()],
                "",
            )?
        };
        let value = self.llvm_builder.build_load(pointee_ty, ptr, "").unwrap();
        Ok(value)
    }
    fn eval_deref(
        &self,
        deref: &DerefExpr,
        ty: &ResolvedType,
    ) -> Result<BasicValueEnum, BuilderError> {
        let ptr = self.gen_expression(&deref.target)?.unwrap();
        let pointee_ty = self
            .type_to_basic_type_enum(ty)
            .unwrap_or(self.type_to_basic_type_enum(&ResolvedType::U8).unwrap());
        let value = self
            .llvm_builder
            .build_load(pointee_ty, ptr.into_pointer_value(), "")?;
        Ok(value)
    }
    fn eval_binary_expr(&self, binary_expr: &BinaryExpr) -> Result<BasicValueEnum, BuilderError> {
        let mut left = self.gen_expression(&binary_expr.lhs)?.unwrap();
        let mut right = self.gen_expression(&binary_expr.rhs)?.unwrap();

        let (lhs_cast_type, rhs_cast_type) =
            self.get_cast_type(&binary_expr.lhs.ty, &binary_expr.rhs.ty);

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
                    self.llvm_builder
                        .build_int_add(left.into_int_value(), right.into_int_value(), "")
                        .unwrap()
                } else {
                    unimplemented!()
                }
            }
            BinaryOp::Sub => {
                if result_type.is_integer_type() {
                    self.llvm_builder
                        .build_int_sub(left.into_int_value(), right.into_int_value(), "")
                        .unwrap()
                } else {
                    unimplemented!()
                }
            }
            BinaryOp::Mul => {
                if result_type.is_integer_type() {
                    self.llvm_builder
                        .build_int_mul(left.into_int_value(), right.into_int_value(), "")
                        .unwrap()
                } else {
                    unimplemented!()
                }
            }
            BinaryOp::Div => {
                if result_type.is_integer_type() {
                    self.llvm_builder
                        .build_int_unsigned_div(left.into_int_value(), right.into_int_value(), "")
                        .unwrap()
                } else {
                    unimplemented!()
                }
            }
        };

        Ok(value.as_basic_value_enum())
    }
    fn eval_sizeof(&self, ty: &ResolvedType) -> BasicValueEnum {
        let size = self.type_to_basic_type_enum(ty).unwrap().size_of().unwrap();
        size.as_basic_value_enum()
    }
    pub(super) fn gen_call_expr(
        &self,
        call_expr: &CallExpr,
    ) -> Result<Option<BasicValueEnum<'_>>, BuilderError> {
        let args = call_expr
            .args
            .iter()
            .map(|arg| self.gen_expression(&arg).map(|x| x.unwrap().into()))
            .collect::<Result<Vec<BasicMetadataValueEnum>, _>>()?;
        let function = *self.function_by_name.get(&call_expr.callee).unwrap();
        let func = self.gen_or_get_function(function);
        let value = self.llvm_builder.build_call(func, &args, "").unwrap();
        Ok(value.try_as_basic_value().left())
    }
    pub(super) fn gen_expression(
        &self,
        expr: &ResolvedExpression,
    ) -> Result<Option<BasicValueEnum>, BuilderError> {
        match &expr.kind {
            ExpressionKind::NumberLiteral(number_literal) => {
                self.eval_number_literal(number_literal, &expr.ty).map(Some)
            }
            ExpressionKind::VariableRef(variable_ref) => {
                Ok(Some(self.eval_variable_ref(variable_ref, &expr.ty)))
            }
            ExpressionKind::IndexAccess(index_access) => {
                self.eval_index_access(index_access, &expr.ty).map(Some)
            }
            ExpressionKind::Deref(deref) => self.eval_deref(deref, &expr.ty).map(Some),
            ExpressionKind::BinaryExpr(binary_expr) => self.eval_binary_expr(binary_expr).map(Some),
            ExpressionKind::CallExpr(call_expr) => self.gen_call_expr(call_expr),
            ExpressionKind::StringLiteral(string_literal) => {
                self.eval_string_literal(string_literal).map(Some)
            }
            ExpressionKind::StructLiteral(struct_literal) => {
                self.eval_struct_literal(struct_literal).map(Some)
            }
            ExpressionKind::SizeOf(ty) => Ok(Some(self.eval_sizeof(ty))),
        }
    }
}
