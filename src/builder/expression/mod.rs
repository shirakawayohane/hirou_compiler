mod binary;
mod multi;
mod unary;

use super::*;
use crate::concrete_ast::*;
use inkwell::{
    builder::BuilderError,
    types::BasicType,
    values::{BasicMetadataValueEnum, BasicValue, BasicValueEnum},
    AddressSpace,
};

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
    fn eval_number_literal(
        &self,
        integer_literal: &NumberLiteral,
        ty: &ConcreteType,
    ) -> Result<BasicValueEnum, BuilderError> {
        let value_str = &integer_literal.value;
        Ok(match ty {
            ConcreteType::U8 => self.eval_u8(value_str),
            ConcreteType::U32 => self.eval_u32(value_str),
            ConcreteType::I32 => self.eval_i32(value_str),
            ConcreteType::I64 => self.eval_i64(value_str),
            ConcreteType::U64 => self.eval_u64(value_str),
            ConcreteType::Ptr(_) => unreachable!(),
            ConcreteType::Void => unreachable!(),
            ConcreteType::StructLike(_) => unreachable!(),
            ConcreteType::Bool => unreachable!(),
        })
    }
    fn eval_bool_literal(
        &self,
        bool_literal: &BoolLiteral,
    ) -> Result<BasicValueEnum, BuilderError> {
        let value = bool_literal.value;
        let bool_value = self.llvm_context.bool_type().const_int(value as u64, false);
        Ok(bool_value.into())
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
        ty: &ConcreteType,
    ) -> Result<BasicValueEnum, BuilderError> {
        let ty = self.type_to_basic_type_enum(ty).unwrap();
        let ptr = self.llvm_builder.build_alloca(ty, "")?;
        for (i, (_name, field_expr)) in struct_literal.fields.iter().enumerate() {
            let value = self.gen_expression(field_expr)?.unwrap();
            let ptr = self.llvm_builder.build_struct_gep(ty, ptr, i as u32, "")?;
            self.llvm_builder.build_store(ptr, value)?;
        }
        Ok(ptr.as_basic_value_enum())
    }
    fn eval_variable_ref(
        &self,
        variable_ref: &VariableRefExpr,
        ty: &ConcreteType,
    ) -> Result<BasicValueEnum, BuilderError> {
        let ptr = self.get_variable(&variable_ref.name);
        let pointee_ty = self.type_to_basic_type_enum(ty).unwrap();
        if ty.is_struct_type() {
            Ok(ptr.as_basic_value_enum())
        } else {
            Ok(self.llvm_builder.build_load(pointee_ty, ptr, "")?)
        }
    }
    fn eval_index_access(
        &self,
        index_access: &IndexAccessExpr,
        ty: &ConcreteType,
    ) -> Result<BasicValueEnum, BuilderError> {
        let ptr = self
            .gen_expression(&index_access.target)?
            .unwrap()
            .into_pointer_value();
        let pointee_ty = self.type_to_basic_type_enum(ty).unwrap();
        let index = self.gen_expression(&index_access.index)?.unwrap();
        let ptr = unsafe {
            self.llvm_builder
                .build_in_bounds_gep(pointee_ty, ptr, &[index.into_int_value()], "")?
        };
        if ty.is_struct_type() {
            Ok(ptr.as_basic_value_enum())
        } else {
            let value = self.llvm_builder.build_load(pointee_ty, ptr, "")?;
            Ok(value)
        }
    }
    fn eval_field_access(
        &self,
        field_access: &FieldAccessExpr,
        ty: &ConcreteType,
    ) -> Result<BasicValueEnum, BuilderError> {
        if let ConcreteType::StructLike(struct_ty) = &field_access.target.ty {
            let ty_enum = self.type_to_basic_type_enum(ty).unwrap();
            let index: usize = struct_ty
                .fields
                .iter()
                .position(|x| x.0 == field_access.field_name)
                .unwrap();
            let struct_ptr = self
                .gen_expression(&field_access.target)?
                .unwrap()
                .into_pointer_value();
            let field_ptr = self.llvm_builder.build_struct_gep(
                self.type_to_basic_type_enum(&field_access.target.ty)
                    .unwrap(),
                struct_ptr,
                index as u32,
                "",
            )?;
            let value = self
                .llvm_builder
                .build_load(ty_enum, field_ptr, "")
                .unwrap();
            Ok(value)
        } else {
            unreachable!()
        }
    }
    fn eval_deref(
        &self,
        deref: &DerefExpr,
        ty: &ConcreteType,
    ) -> Result<BasicValueEnum, BuilderError> {
        let ptr = self.gen_expression(&deref.target)?.unwrap();
        let pointee_ty = self
            .type_to_basic_type_enum(ty)
            .unwrap_or(self.type_to_basic_type_enum(&ConcreteType::U8).unwrap());
        let value = self
            .llvm_builder
            .build_load(pointee_ty, ptr.into_pointer_value(), "")?;
        Ok(value)
    }
    fn eval_sizeof(&self, ty: &ConcreteType) -> BasicValueEnum {
        let size = self.type_to_basic_type_enum(ty).unwrap().size_of().unwrap();
        size.as_basic_value_enum()
    }
    pub(super) fn eval_call_expr<'a>(
        &'a self,
        call_expr: &CallExpr,
    ) -> Result<Option<BasicValueEnum<'a>>, BuilderError> {
        let mut args = call_expr
            .args
            .iter()
            .map(|arg| {
                self.gen_expression(arg).map(|x| {
                    if arg.ty.is_struct_type() {
                        let ty = self.type_to_basic_type_enum(&arg.ty).unwrap();
                        self.llvm_builder
                            .build_load(ty, x.unwrap().into_pointer_value(), "")
                            .unwrap()
                            .into()
                    } else {
                        x.unwrap().into()
                    }
                })
            })
            .collect::<Result<Vec<BasicMetadataValueEnum>, _>>()?;

        let function = *self.function_by_name.get(&call_expr.callee).unwrap();
        let func = self.gen_or_get_function(function);
        // 構造体を返す関数を呼ぶ場合、第一引数にスタックポインタを渡す
        if let ConcreteType::StructLike(_) = &function.decl.return_type {
            let return_ty = self
                .type_to_basic_type_enum(&function.decl.return_type)
                .unwrap();
            let ptr = self.llvm_builder.build_alloca(return_ty, "")?;
            args.insert(0, ptr.into());
            self.llvm_builder.build_call(func, &args, "")?;
            // let value = self.llvm_builder.build_load(return_ty, ptr, "")?;
            return Ok(Some(ptr.as_basic_value_enum()));
        }
        let value = self.llvm_builder.build_call(func, &args, "").unwrap();
        Ok(value.try_as_basic_value().basic())
    }
    pub(super) fn eval_if_expr<'a>(
        &'a self,
        if_expr: &IfExpr,
        ty: &ConcreteType,
    ) -> Result<Option<BasicValueEnum<'a>>, BuilderError> {
        // condがboolであることはresolverで保証されている
        let cond = self
            .gen_expression(&if_expr.cond)?
            .unwrap()
            .into_int_value();
        let function: inkwell::values::FunctionValue<'_> = self
            .llvm_builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();
        let then_block = self.llvm_context.append_basic_block(function, "then");
        let else_block = self.llvm_context.append_basic_block(function, "else");
        let merge_block = self.llvm_context.append_basic_block(function, "ifcont");
        self.llvm_builder
            .build_conditional_branch(cond, then_block, else_block)?;
        self.llvm_builder.position_at_end(then_block);
        let then_value = self.gen_expression(&if_expr.then)?.unwrap();
        self.llvm_builder.build_unconditional_branch(merge_block)?;
        let then_block = self.llvm_builder.get_insert_block().unwrap();
        self.llvm_builder.position_at_end(else_block);
        let else_value = self.gen_expression(&if_expr.els)?.unwrap();
        self.llvm_builder.build_unconditional_branch(merge_block)?;
        let else_block = self.llvm_builder.get_insert_block().unwrap();
        if matches!(ty, ConcreteType::Void) {
            Ok(None)
        } else {
            self.llvm_builder.position_at_end(merge_block);
            let phi = self
                .llvm_builder
                .build_phi(self.type_to_basic_type_enum(ty).unwrap(), "iftmp")?;
            phi.add_incoming(&[(&then_value, then_block), (&else_value, else_block)]);
            Ok(Some(phi.as_basic_value()))
        }
    }
    pub(super) fn eval_when_expr<'a>(
        &'a self,
        when_expr: &WhenExpr,
    ) -> Result<Option<BasicValueEnum<'a>>, BuilderError> {
        // condがboolであることはresolverで保証されている
        let cond = self
            .gen_expression(&when_expr.cond)?
            .unwrap()
            .into_int_value();
        let function: inkwell::values::FunctionValue<'_> = self
            .llvm_builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();
        let then_block = self.llvm_context.append_basic_block(function, "then");
        let else_block = self.llvm_context.append_basic_block(function, "else");
        let merge_block = self.llvm_context.append_basic_block(function, "ifcont");
        self.llvm_builder
            .build_conditional_branch(cond, then_block, else_block)?;
        self.llvm_builder.position_at_end(then_block);
        let _then_value = self.gen_expression(&when_expr.then)?.unwrap();
        self.llvm_builder.build_unconditional_branch(merge_block)?;
        self.llvm_builder.position_at_end(else_block);
        self.llvm_builder.build_unconditional_branch(merge_block)?;
        self.llvm_builder.position_at_end(merge_block);
        Ok(None)
    }
    pub(super) fn eval_variable_decls(&self, decls: &VariableDecls) -> Result<(), BuilderError> {
        for decl in &decls.decls {
            let ty = self.type_to_basic_type_enum(&decl.value.ty).unwrap();
            let value = self.gen_expression(&decl.value)?.unwrap();
            if ty.is_struct_type() {
                let ptr = self.llvm_builder.build_alloca(ty, "")?;
                self.llvm_builder.build_memcpy(
                    ptr,
                    8,
                    value.into_pointer_value(),
                    8,
                    ty.size_of().unwrap(),
                )?;
                self.add_variable(&decl.name, ptr);
            } else {
                let ptr = self.llvm_builder.build_alloca(ty, "")?;
                self.llvm_builder.build_store(ptr, value)?;
                self.add_variable(&decl.name, ptr);
            }
        }
        Ok(())
    }
    pub(super) fn eval_assignment(&self, assignment: &Assignment) -> Result<(), BuilderError> {
        let value = self.gen_expression(&assignment.value)?.unwrap();
        let pointee_type = value.get_type();
        let mut ptr = self.get_variable(&assignment.name);
        for _ in 0..assignment.deref_count {
            ptr = self
                .llvm_builder
                .build_load(pointee_type, ptr, "")
                .unwrap()
                .into_pointer_value();
        }
        if let Some(index_access) = &assignment.index_access {
            let index = self.gen_expression(index_access)?.unwrap();
            ptr = self
                .llvm_builder
                .build_load(self.llvm_context.ptr_type(AddressSpace::default()), ptr, "")
                .unwrap()
                .into_pointer_value();

            ptr = unsafe {
                self.llvm_builder
                    .build_in_bounds_gep(pointee_type, ptr, &[index.into_int_value()], "")
                    .unwrap()
            };
            if assignment.value.ty.is_struct_type() {
                self.llvm_builder.build_memcpy(
                    ptr,
                    8,
                    value.into_pointer_value(),
                    8,
                    pointee_type.size_of().unwrap(),
                )?;
                return Ok(());
            }
        }
        self.llvm_builder.build_store(ptr, value)?;
        Ok(())
    }
    pub(super) fn gen_expression<'a>(
        &'a self,
        expr: &ConcreteExpression,
    ) -> Result<Option<BasicValueEnum<'a>>, BuilderError> {
        match &expr.kind {
            ExpressionKind::NumberLiteral(number_literal) => {
                self.eval_number_literal(number_literal, &expr.ty).map(Some)
            }
            ExpressionKind::VariableRef(variable_ref) => {
                self.eval_variable_ref(variable_ref, &expr.ty).map(Some)
            }
            ExpressionKind::IndexAccess(index_access) => {
                self.eval_index_access(index_access, &expr.ty).map(Some)
            }
            ExpressionKind::Deref(deref) => self.eval_deref(deref, &expr.ty).map(Some),
            ExpressionKind::Binary(binary_expr) => self.eval_binary_expr(binary_expr).map(Some),
            ExpressionKind::Unary(unary_expr) => self.eval_unary_expr(unary_expr).map(Some),
            ExpressionKind::Multi(multi_expr) => self.eval_multi_expr(multi_expr).map(Some),
            ExpressionKind::CallExpr(call_expr) => self.eval_call_expr(call_expr),
            ExpressionKind::StringLiteral(string_literal) => {
                self.eval_string_literal(string_literal).map(Some)
            }
            ExpressionKind::StructLiteral(struct_literal) => {
                self.eval_struct_literal(struct_literal, &expr.ty).map(Some)
            }
            ExpressionKind::SizeOf(ty) => Ok(Some(self.eval_sizeof(ty))),
            ExpressionKind::FieldAccess(field_access_expr) => self
                .eval_field_access(field_access_expr, &expr.ty)
                .map(Some),
            ExpressionKind::Unknown => unreachable!(),
            ExpressionKind::BoolLiteral(bool_literal) => {
                self.eval_bool_literal(bool_literal).map(Some)
            }
            ExpressionKind::If(if_expr) => self.eval_if_expr(if_expr, &expr.ty),
            ExpressionKind::When(when_expr) => self.eval_when_expr(when_expr),
            ExpressionKind::VariableDecls(decls) => {
                self.eval_variable_decls(decls)?;
                Ok(None)
            }
            ExpressionKind::Assignment(assignment) => {
                self.eval_assignment(assignment).map(|_| None)
            }
            ExpressionKind::Return(ret) => {
                self.gen_return(ret)?;
                Ok(None)
            }
        }
    }
}
