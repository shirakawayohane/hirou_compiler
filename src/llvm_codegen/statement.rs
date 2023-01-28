use super::value::Value;
use super::*;
use crate::ast::*;

impl LLVMCodegenerator<'_> {
    pub(super) fn gen_statement(&self, statement: Statement) -> Result<(), CompileError> {
        match statement {
            Statement::VariableDecl { name, value } => {
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
                }
            }
            Statement::Return { expression } => {
                if let Some(exp) = expression {
                    let value = self.eval_expression(exp)?;
                    self.llvm_builder.build_return(match &value {
                        Value::IntValue(v) => Some(v),
                        Value::Void => None,
                    });
                } else {
                    self.llvm_builder.build_return(None);
                }
            }
            Statement::Asignment { name, expression } => {
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
            }
            Statement::DiscardedExpression { expression } => {
                self.eval_expression(expression)?;
            }
        };
        Ok(())
    }
}
