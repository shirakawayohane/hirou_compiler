use crate::{resolved_ast::VariableDecls, resolver::ResolverContext};

use super::*;
use ast::*;

pub(super) fn resolve_variable_decl(
    context: &ResolverContext,
    variable_decls_expr: &Located<&VariableDeclsExpr>,
) -> Result<ResolvedExpression, FaitalError> {
    in_new_scope!(context.types, {
        let mut decls = Vec::new();
        for variable_decl_expr in &variable_decls_expr.decls {
            let resolved_annotation = variable_decl_expr
                .ty
                .clone()
                .map(|unresolved_ty| resolve_type(context, &unresolved_ty))
                .transpose()?;
            let resolved_expr = resolve_expression(
                context,
                variable_decl_expr.value.value.as_deref(),
                resolved_annotation.as_ref(),
            )?;
            if let Some(resolved_annotation) = resolved_annotation {
                if !resolved_annotation.can_insert(&resolved_expr.ty) {
                    context.errors.borrow_mut().push(CompileError::new(
                        variable_decl_expr.range,
                        CompileErrorKind::TypeMismatch {
                            expected: resolved_annotation.clone(),
                            actual: resolved_expr.ty.clone(),
                        },
                    ));
                }
            }
            context
                .scopes
                .borrow_mut()
                .add(variable_decl_expr.name.clone(), resolved_expr.ty.clone());
            decls.push(resolved_ast::VariableDecl {
                name: variable_decl_expr.name.clone(),
                value: Box::new(resolved_expr),
            });
        }
        Ok(ResolvedExpression {
            ty: ResolvedType::Void,
            kind: ExpressionKind::VariableDecls(VariableDecls { decls }),
        })
    })
}
