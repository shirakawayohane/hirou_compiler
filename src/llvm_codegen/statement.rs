use core::panic;

use inkwell::values::{BasicValue, PointerValue};
use inkwell::AddressSpace;

use super::error::ContextType;
use super::result_value::Value;
use super::*;
use crate::{ast::*, error_context};

impl LLVMCodegenerator<'_> {
    fn gen_variable_decl(
        &mut self,
        ctx: Rc<RefCell<Context>>,
        ty: UnresolvedType,
        name: String,
        value: Expression,
    ) -> Result<(), CompileError> {
        let resolved_ty = self.resolve_type(ctx.clone(), &ty)?;
        match &resolved_ty {
            ResolvedType::I32
            | ResolvedType::U8
            | ResolvedType::U32
            | ResolvedType::U64
            | ResolvedType::USize => {
                let variable_pointer = self.llvm_builder.build_alloca(
                    match resolved_ty {
                        ResolvedType::I32 => self.i32_type,
                        ResolvedType::USize => match self.pointer_size {
                            PointerSize::SixteenFour => self.i64_type,
                        },
                        ResolvedType::U32 => self.i32_type,
                        ResolvedType::U64 => self.i64_type,
                        ResolvedType::U8 => self.i8_type,
                        _ => panic!(),
                    },
                    &name,
                );

                {
                    let (_, evaluated_value) =
                        self.eval_expression(ctx.clone(), &value, Some(&resolved_ty))?;

                    match evaluated_value {
                        Value::I32Value(v) | Value::U64Value(v) | Value::U8Value(v) => {
                            self.llvm_builder.build_store(variable_pointer, v)
                        }
                        _ => panic!(),
                    };
                }

                // Contextに登録
                self.set_local_variable(ctx.clone(), name, resolved_ty, variable_pointer);
            }
            ResolvedType::Ptr(ptr_ty) => {
                let variable_pointer = self.llvm_builder.build_alloca(
                    match **ptr_ty {
                        ResolvedType::I32 => self.i32_type,
                        ResolvedType::USize => match self.pointer_size {
                            PointerSize::SixteenFour => self.i64_type,
                        },
                        ResolvedType::U32 => self.i32_type,
                        ResolvedType::U64 => self.i64_type,
                        ResolvedType::U8 => self.i8_type,
                        _ => panic!(),
                    }
                    .ptr_type(AddressSpace::default()),
                    &name,
                );
                let (_, evaluated_value) =
                    self.eval_expression(ctx.clone(), &value, Some(&resolved_ty))?;

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
                self.set_local_variable(ctx.clone(), name, *ptr_ty.clone(), variable_pointer);
            }
            ResolvedType::Void => {
                let _result = self.eval_expression(ctx.clone(), &value, Some(&resolved_ty));
                unsafe {
                    let null_pointer = 0 as *const PointerValue;
                    self.set_local_variable(ctx.clone(), name, resolved_ty, *null_pointer)
                };
            }
        }
        Ok(())
    }
    fn gen_return(
        &self,
        ctx: Rc<RefCell<Context>>,
        opt_expr: &Option<Located<Expression>>,
    ) -> Result<(), CompileError> {
        if let Some(exp) = opt_expr {
            let (_, value) = self.eval_expression(ctx.clone(), &exp.value, None)?;
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
        &mut self,
        ctx: Rc<RefCell<Context>>,
        deref_count: u32,
        index_access: &Option<Located<Expression>>,
        name: String,
        expression: Located<&Expression>,
    ) -> Result<(), CompileError> {
        let (ty, ptr) = ctx.borrow().find_variable(&name)?;
        let mut asign_type = ty;
        let mut ptr_to_asign = ptr;

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
                ResolvedType::Ptr(pointer_of) => *pointer_of,
                _ => {
                    return Err(CompileError::from_error_kind(
                        CompileErrorKind::CannotDeref { name, deref_count },
                    ))
                }
            };
        }

        if let Some(index_expr) = index_access {
            // Check type first
            asign_type = match asign_type {
                ResolvedType::Ptr(v) => *v,
                _ => {
                    return Err(CompileError::from_error_kind(
                        CompileErrorKind::CannotIndexAccess {
                            name: name.to_string(),
                            ty: asign_type.clone(),
                        },
                    ))
                }
            };

            let (_, index_value) =
                self.eval_expression(ctx.clone(), &index_expr.value, Some(&ResolvedType::USize))?;
            // deref and move ptr by sizeof(T) * index
            ptr_to_asign = match self.llvm_builder.build_load(ptr_to_asign, "deref") {
                inkwell::values::BasicValueEnum::PointerValue(ptr) => ptr,
                _ => {
                    return Err(CompileError::from_error_kind(
                        CompileErrorKind::CannotDeref { name, deref_count },
                    ))
                }
            };
            ptr_to_asign = unsafe {
                self.llvm_builder.build_gep(
                    ptr_to_asign,
                    &[index_value.unwrap_int_value()],
                    "array_indexing",
                )
            };
        }

        let (_, value) =
            self.eval_expression(ctx.clone(), &expression.value, Some(&asign_type.clone()))?;

        let value_type = value.get_type();

        // Type checking
        if value_type != asign_type {
            return Err(CompileError::from_error_kind(
                CompileErrorKind::TypeMismatch {
                    expected: asign_type.clone(),
                    actual: value_type,
                },
            ));
        }

        if value.get_type().is_integer_type() {
            self.llvm_builder
                .build_store(ptr_to_asign, value.unwrap_int_value());
        } else if value.get_type().is_pointer_type() {
            self.llvm_builder
                .build_store(ptr_to_asign, value.unwrap_pointer_value());
        } else {
            panic!()
        };
        Ok(())
    }
    fn gen_discarded_expression(
        &mut self,
        ctx: Rc<RefCell<Context>>,
        expression: &Expression,
    ) -> Result<(), CompileError> {
        self.eval_expression(ctx.clone(), expression, None)?;
        Ok(())
    }
    pub(super) fn gen_statement(
        &mut self,
        ctx: Rc<RefCell<Context>>,
        statement: &Statement,
    ) -> Result<(), CompileError> {
        match &statement {
            Statement::VariableDecl {
                ty: loc_ty,
                name,
                value: loc_value,
            } => {
                error_context!(
                    ContextType::VariableDeclStatement,
                    self.gen_variable_decl(
                        ctx.clone(),
                        loc_ty.value.clone(),
                        name.clone(),
                        loc_value.value.clone()
                    )
                )
            }
            Statement::Return {
                expression: loc_expr,
            } => {
                error_context!(
                    ContextType::ReturnStatement,
                    self.gen_return(ctx.clone(), loc_expr)
                )
            }
            Statement::Asignment {
                deref_count,
                index_access,
                name,
                expression,
            } => error_context!(
                ContextType::AsignStatement,
                self.gen_asignment(
                    ctx.clone(),
                    *deref_count,
                    index_access,
                    name.clone(),
                    expression.as_inner_ref()
                )
            ),
            Statement::Effect {
                expression: loc_expr,
            } => error_context!(
                ContextType::DiscardedExpressionStatement,
                self.gen_discarded_expression(ctx.clone(), &loc_expr.value)
            ),
        }?;
        Ok(())
    }
}
