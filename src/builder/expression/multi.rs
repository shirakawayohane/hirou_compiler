use super::*;
use crate::ast::MultiOp;

impl LLVMCodeGenerator<'_> {
    pub(super) fn eval_multi_expr(
        &self,
        multi_expr: &MultiExpr,
    ) -> Result<BasicValueEnum, BuilderError> {
        let operands = multi_expr
            .operands
            .iter()
            .map(|operand| self.gen_expression(operand))
            .collect::<Result<Vec<_>, _>>()?;

        match multi_expr.op {
            MultiOp::And | MultiOp::Or => {
                let mut result = self.llvm_context.bool_type().const_int(1, false);
                for operand in operands {
                    let operand = operand.unwrap().into_int_value();
                    result = match multi_expr.op {
                        MultiOp::And => self.llvm_builder.build_and(result, operand, "and")?,
                        MultiOp::Or => self.llvm_builder.build_or(result, operand, "or")?,
                    };
                }
                Ok(result.as_basic_value_enum())
            }
        }
    }
}
