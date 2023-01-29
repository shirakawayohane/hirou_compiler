use super::value::Value;
use super::*;
use crate::ast::*;

impl LLVMCodegenerator<'_> {
    fn gen_variable_decl(
        &self,
        ty: &Type,
        name: String,
        value: Expression,
    ) -> Result<(), CompileError> {
        match ty {
            Type::I32 | Type::U64 | Type::U8 => {
                let variable_pointer = self.llvm_builder.build_alloca(
                    match ty {
                        Type::I32 => self.i32_type,
                        Type::U64 => self.i64_type,
                        Type::U8 => self.i8_type,
                        _ => panic!(),
                    },
                    &name,
                );

                let evaluated_value = self.eval_expression(value, Some(ty))?;

                match evaluated_value {
                    Value::I32Value(v) | Value::U64Value(v) | Value::U8Value(v) => {
                        self.llvm_builder.build_store(variable_pointer, v)
                    }
                    _ => panic!(),
                };

                // Contextに登録
                self.context
                    .borrow_mut()
                    .set_variable(name, ty.clone(), variable_pointer);
            }
            Type::Ptr(_) => todo!(),
        }
        Ok(())
    }
    fn gen_return(&self, opt_expr: Option<Expression>) -> Result<(), CompileError> {
        if let Some(exp) = opt_expr {
            let value = self.eval_expression(exp, None)?;
            self.llvm_builder.build_return(match &value {
                Value::U8Value(v) => Some(v),
                Value::I32Value(v) => Some(v),
                Value::U64Value(v) => Some(v),
                Value::Void => None,
            });
        } else {
            self.llvm_builder.build_return(None);
        }
        Ok(())
    }
    fn gen_asignment(&self, name: String, expression: Expression) -> Result<(), CompileError> {
        if let Some((ty, ptr)) = self.context.borrow().find_variable(&name) {
            let value = self.eval_expression(expression, None)?;
            self.llvm_builder.build_store(
                *ptr,
                match value {
                    Value::I32Value(v) => {
                        if *ty != Type::I32 {
                            return Err(CompileError::AsignValueDoesNotMatch {
                                expected: Box::new(Type::I32),
                                actual: Box::new(ty.clone()),
                            });
                        }
                        v
                    }
                    Value::U64Value(v) => {
                        if *ty != Type::U64 {
                            return Err(CompileError::AsignValueDoesNotMatch {
                                expected: Box::new(Type::U64),
                                actual: Box::new(ty.clone()),
                            });
                        }
                        v
                    }
                    Value::U8Value(v) => {
                        if *ty != Type::U8 {
                            return Err(CompileError::AsignValueDoesNotMatch {
                                expected: Box::new(Type::U8),
                                actual: Box::new(ty.clone()),
                            });
                        }
                        v
                    }
                    Value::Void => return Ok(()),
                },
            );
        } else {
            return Err(CompileError::VariableNotFound { name });
        }
        Ok(())
    }
    fn gen_discarded_expression(&self, expression: Expression) -> Result<(), CompileError> {
        self.eval_expression(expression, None)?;
        Ok(())
    }
    pub(super) fn gen_statement(&self, statement: Statement) -> Result<(), CompileError> {
        match statement {
            Statement::VariableDecl { ty, name, value } => self.gen_variable_decl(&ty, name, value),
            Statement::Return { expression } => self.gen_return(expression),
            Statement::Asignment { name, expression } => self.gen_asignment(name, expression),
            Statement::DiscardedExpression { expression } => {
                self.gen_discarded_expression(expression)
            }
        }?;
        Ok(())
    }
}
