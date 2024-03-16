use crate::ast::{Located, Statement};
use crate::resolved_ast::{self};

use super::error::FaitalError;
use super::expression::resolve_expression;
use super::ResolverContext;

pub fn resolve_statement(
    context: &ResolverContext,
    loc_statement: &Located<Statement>,
) -> Result<resolved_ast::Statement, FaitalError> {
    Ok(match &loc_statement.value {
        Statement::Return(ret) => {
            if let Some(expr) = &ret.expression {
                resolved_ast::Statement::Return(resolved_ast::Return {
                    expression: Some(resolve_expression(context, expr.as_ref().into(), None)?),
                })
            } else {
                resolved_ast::Statement::Return(resolved_ast::Return { expression: None })
            }
        }
        Statement::Effect(effect) => resolved_ast::Statement::Effect(resolved_ast::Effect {
            expression: resolve_expression(context, effect.expression.as_ref(), None)?,
        }),
    })
}
