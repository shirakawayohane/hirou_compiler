use super::*;

use crate::resolver::{AssignExpr, ResolverContext};

//上記を参考にして、Statementではなく、Effectとして扱うことにする
pub(super) fn resolve_assignment(
    context: &ResolverContext,
    assignment_expr: &Located<&AssignExpr>,
) -> Result<ResolvedExpression, FaitalError> {
    let resolved_expr =
        resolve_expression(context, assignment_expr.value.value.as_inner_deref(), None)?;
    Ok(ResolvedExpression {
        ty: ResolvedType::Void,
        kind: ExpressionKind::Assignment(resolved_ast::Assignment {
            name: assignment_expr.name.clone(),
            value: Box::new(resolved_expr),
            deref_count: assignment_expr.deref_count as usize,
            index_access: assignment_expr
                .index_access
                .as_ref()
                .map(|x| {
                    resolve_expression(context, x.as_inner_deref(), Some(&ResolvedType::USize))
                })
                .transpose()?
                .map(Box::new),
        }),
    })
}
