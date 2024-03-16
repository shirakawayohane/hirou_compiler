use super::*;
use crate::ast::UnaryOp;

impl LLVMCodeGenerator<'_> {
    pub(super) fn eval_unary_expr(
        &self,
        unary_expr: &UnaryExpr,
    ) -> Result<BasicValueEnum, BuilderError> {
        let operand = self.gen_expression(&unary_expr.operand)?.unwrap();

        match unary_expr.op {
            UnaryOp::Not => Ok(self
                .llvm_builder
                .build_not(operand.into_int_value(), "not")?
                .into()),
        }
    }
}
