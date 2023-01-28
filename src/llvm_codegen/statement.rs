use super::value::Value;
use super::*;
use crate::ast::*;

impl LLVMCodegenerator<'_> {
    fn gen_variable_decl(
        &self,
        ty: Type,
        name: String,
        value: Expression,
    ) -> Result<(), CompileError> {
        match ty {
            Type::I32 => {
                let variable_pointer = self
                    .llvm_builder
                    .build_alloca(self.llvm_context.i32_type(), &name);

                // Contextに登録
                self.context
                    .borrow_mut()
                    .set_variable(name, variable_pointer);

                match self.eval_expression(value)? {
                    Value::IntValue(v) => {
                        self.llvm_builder.build_store(variable_pointer, v);
                    }
                    Value::Void => {}
                };

                Ok(())
            }
            Type::I64 => todo!(),
            Type::U8 => todo!(),
            Type::Ptr(_) => todo!(),
        }
    }
    fn gen_return(&self, opt_expr: Option<Expression>) -> Result<(), CompileError> {
        if let Some(exp) = opt_expr {
            let value = self.eval_expression(exp)?;
            self.llvm_builder.build_return(match &value {
                Value::IntValue(v) => Some(v),
                Value::Void => None,
            });
        } else {
            self.llvm_builder.build_return(None);
        }
        Ok(())
    }
    fn gen_asignment(&self, name: String, expression: Expression) -> Result<(), CompileError> {
        if let Some(pointer) = self.context.borrow().find_variable(&name) {
            let value = self.eval_expression(expression)?;
            self.llvm_builder.build_store(
                pointer,
                match value {
                    Value::IntValue(v) => v,
                    Value::Void => return Ok(()),
                },
            );
        } else {
            return Err(CompileError::VariableNotFound { name });
        }
        Ok(())
    }
    fn gen_discarded_expression(&self, expression: Expression) -> Result<(), CompileError> {
        self.eval_expression(expression)?;
        Ok(())
    }
    pub(super) fn gen_statement(&self, statement: Statement) -> Result<(), CompileError> {
        match statement {
            Statement::VariableDecl { ty, name, value } => self.gen_variable_decl(ty, name, value),
            Statement::Return { expression } => self.gen_return(expression),
            Statement::Asignment { name, expression } => self.gen_asignment(name, expression),
            Statement::DiscardedExpression { expression } => {
                self.gen_discarded_expression(expression)
            }
        };
        Ok(())
    }
}
