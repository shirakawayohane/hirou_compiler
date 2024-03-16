use super::*;

use crate::{common::target::PointerSizedIntWidth, resolver::AssignExpr};

//上記を参考にして、Statementではなく、Effectとして扱うことにする
pub(super) fn resolve_assignment(
    errors: &mut Vec<CompileError>,
    types: Rc<RefCell<TypeScopes>>,
    scopes: Rc<RefCell<VariableScopes>>,
    type_defs: &HashMap<String, ast::TypeDef>,
    function_by_name: &HashMap<String, ast::Function>,
    resolved_functions: &mut HashMap<String, resolved_ast::Function>,
    assignment_expr: &Located<&AssignExpr>,
    ptr_sized_int_type: PointerSizedIntWidth,
) -> Result<ResolvedExpression, FaitalError> {
    let resolved_expr = resolve_expression(
        errors,
        types.clone(),
        scopes.clone(),
        type_defs,
        function_by_name,
        resolved_functions,
        assignment_expr.value.value.as_inner_deref(),
        None,
        ptr_sized_int_type,
    )?;
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
                    resolve_expression(
                        errors,
                        types.clone(),
                        scopes.clone(),
                        type_defs,
                        function_by_name,
                        resolved_functions,
                        x.as_inner_deref(),
                        Some(&ResolvedType::USize),
                        ptr_sized_int_type,
                    )
                })
                .transpose()?
                .map(Box::new),
        }),
    })
}
