use crate::resolved_ast::VariableDecls;

use super::*;
use ast::*;

pub(super) fn resolve_variable_decl(
    errors: &mut Vec<CompileError>,
    types: Rc<RefCell<TypeScopes>>,
    scopes: Rc<RefCell<VariableScopes>>,
    type_defs: &HashMap<String, ast::TypeDef>,
    function_by_name: &HashMap<String, ast::Function>,
    resolved_functions: &mut HashMap<String, resolved_ast::Function>,
    variable_decls_expr: &Located<&VariableDeclsExpr>,
) -> Result<ResolvedExpression, FaitalError> {
    in_new_scope!(types, {
        let mut decls = Vec::new();
        for variable_decl_expr in &variable_decls_expr.decls {
            let resolved_annotation = variable_decl_expr
                .ty
                .clone()
                .map(|unresolved_ty| {
                    resolve_type(
                        errors,
                        types.borrow_mut().deref_mut(),
                        type_defs,
                        &unresolved_ty,
                    )
                })
                .transpose()?;
            let resolved_expr = resolve_expression(
                errors,
                types.clone(),
                scopes.clone(),
                type_defs,
                function_by_name,
                resolved_functions,
                variable_decl_expr.value.value.as_deref(),
                resolved_annotation.as_ref(),
            )?;
            if let Some(resolved_annotation) = resolved_annotation {
                if !resolved_annotation.can_insert(&resolved_expr.ty) {
                    errors.push(CompileError::new(
                        variable_decl_expr.range,
                        CompileErrorKind::TypeMismatch {
                            expected: resolved_annotation.clone(),
                            actual: resolved_expr.ty.clone(),
                        },
                    ));
                }
            }
            scopes
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
