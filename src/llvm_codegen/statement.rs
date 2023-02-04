use inkwell::values::{BasicValue, PointerValue};
use inkwell::AddressSpace;

use super::error::ContextType;
use super::value::Value;
use super::*;
use crate::{ast::*, error_context};

impl LLVMCodegenerator<'_> {
    fn gen_variable_decl(
        &self,
        ty: Type,
        name: String,
        value: Expression,
    ) -> Result<(), CompileError> {
        match &ty {
            Type::I32 | Type::U8 | Type::U32 | Type::U64 | Type::USize => {
                let variable_pointer = self.llvm_builder.build_alloca(
                    match ty {
                        Type::I32 => self.i32_type,
                        Type::USize => match self.pointer_size {
                            PointerSize::SixteenFour => self.i64_type,
                        },
                        Type::U32 => self.i32_type,
                        Type::U64 => self.i64_type,
                        Type::U8 => self.i8_type,
                        _ => panic!(),
                    },
                    &name,
                );

                let evaluated_value = self.eval_expression(&value, Some(&ty))?;

                match evaluated_value {
                    Value::I32Value(v) | Value::U64Value(v) | Value::U8Value(v) => {
                        self.llvm_builder.build_store(variable_pointer, v)
                    }
                    _ => panic!(),
                };

                // Contextに登録
                self.context
                    .borrow_mut()
                    .set_variable(name, ty, variable_pointer);
            }
            Type::Ptr(ptr_ty) => {
                let variable_pointer = self.llvm_builder.build_alloca(
                    match **ptr_ty {
                        Type::I32 => self.i32_type,
                        Type::USize => match self.pointer_size {
                            PointerSize::SixteenFour => self.i64_type,
                        },
                        Type::U32 => self.i32_type,
                        Type::U64 => self.i64_type,
                        Type::U8 => self.i8_type,
                        _ => panic!(),
                    }
                    .ptr_type(AddressSpace::default()),
                    &name,
                );
                let evaluated_value = self.eval_expression(&value, Some(&ty))?;

                match evaluated_value {
                    Value::I32Value(v)
                    | Value::U32Value(v)
                    | Value::U64Value(v)
                    | Value::U8Value(v)
                    | Value::USizeValue(v) => {
                        self.llvm_builder.build_store(variable_pointer, v);
                    }
                    Value::PointerValue(_, ptr) => {
                        self.llvm_builder.build_store(variable_pointer, ptr);
                    }
                    Value::Void => (),
                };
                // Contextに登録
                self.context
                    .borrow_mut()
                    .set_variable(name, ty, variable_pointer);
            }
            Type::Void => {
                let _result = self.eval_expression(&value, Some(&ty));
                unsafe {
                    let null_pointer = 0 as *const PointerValue;
                    self.context
                        .borrow_mut()
                        .set_variable(name, ty, *null_pointer)
                };
            }
        }
        Ok(())
    }
    fn gen_return(&self, opt_expr: Option<Expression>) -> Result<(), CompileError> {
        if let Some(exp) = opt_expr {
            let value = self.eval_expression(&exp, None)?;
            let return_value: Option<&dyn BasicValue> = match &value {
                Value::U8Value(v) => Some(v),
                Value::I32Value(v) => Some(v),
                Value::U32Value(v) => Some(v),
                Value::U64Value(v) => Some(v),
                Value::USizeValue(v) => Some(v),
                Value::PointerValue(_, ptr) => Some(ptr),
                Value::Void => None,
            };
            self.llvm_builder.build_return(return_value);
        } else {
            self.llvm_builder.build_return(None);
        }
        Ok(())
    }
    fn gen_asignment(
        &self,
        deref_count: u32,
        name: String,
        expression: Expression,
    ) -> Result<(), CompileError> {
        if let Some((ty, ptr)) = self.context.borrow().find_variable(&name) {
            let mut ptr_to_asign = ptr;
            let mut asign_type = ty;
            for _ in 0..deref_count {
                ptr_to_asign = match self.llvm_builder.build_load(ptr_to_asign, "deref") {
                    inkwell::values::BasicValueEnum::PointerValue(ptr) => ptr,
                    _ => {
                        return Err(CompileError::from_error_kind(
                            CompileErrorKind::CannotDeref { name, deref_count },
                        ))
                    }
                };
                asign_type = match asign_type {
                    Type::Ptr(pointer_of) => &pointer_of,
                    _ => {
                        return Err(CompileError::from_error_kind(
                            CompileErrorKind::CannotDeref { name, deref_count },
                        ))
                    }
                };
            }

            let value = self.eval_expression(&expression, Some(asign_type))?;

            if let Type::Ptr(_) = asign_type {
                self.llvm_builder.build_store(
                    ptr_to_asign,
                    match value {
                        Value::PointerValue(_, v) => v,
                        _ => {
                            return Err(CompileError::from_error_kind(
                                CompileErrorKind::TypeMismatch {
                                    expected: Box::new(ty.clone()),
                                    actual: Box::new(value.get_type()),
                                },
                            ));
                        }
                    },
                );
            } else {
                let value_type = value.get_type();
                if value_type != *asign_type {
                    return Err(CompileError::from_error_kind(
                        CompileErrorKind::TypeMismatch {
                            expected: Box::new(asign_type.clone()),
                            actual: Box::new(value_type),
                        },
                    ));
                }
                if value.is_integer() {
                    self.llvm_builder
                        .build_store(ptr_to_asign, value.unwrap_int_value());
                } else {
                    unimplemented!();
                }
            };
        } else {
            return Err(CompileError::from_error_kind(
                CompileErrorKind::VariableNotFound {
                    name: name.to_string(),
                },
            ));
        }
        Ok(())
    }
    fn gen_discarded_expression(&self, expression: Expression) -> Result<(), CompileError> {
        self.eval_expression(&expression, None)?;
        Ok(())
    }
    pub(super) fn gen_statement(&self, statement: Statement) -> Result<(), CompileError> {
        match statement {
            Statement::VariableDecl {
                ty: loc_ty,
                name,
                value: loc_value,
            } => {
                error_context!(
                    ContextType::VariableDeclStatement,
                    self.gen_variable_decl(loc_ty.value, name, loc_value.value)
                )
            }
            Statement::Return {
                expression: loc_expr,
            } => {
                error_context!(
                    ContextType::ReturnStatement,
                    self.gen_return(loc_expr.map(|x| x.value))
                )
            }
            Statement::Asignment {
                deref_count,
                name,
                expression,
            } => error_context!(
                ContextType::AsignStatement,
                self.gen_asignment(deref_count, name, expression.value)
            ),
            Statement::DiscardedExpression {
                expression: loc_expr,
            } => error_context!(
                ContextType::DiscardedExpressionStatement,
                self.gen_discarded_expression(loc_expr.value)
            ),
        }?;
        Ok(())
    }
}
