use crate::common::{binary::get_cast_type, target::PointerSizedIntWidth};

use self::ast::BinaryExpr;

use super::*;

pub(super) fn resolve_binary_expression(
    errors: &mut Vec<CompileError>,
    types: Rc<RefCell<TypeScopes>>,
    scopes: Rc<RefCell<VariableScopes>>,
    type_defs: &HashMap<String, ast::TypeDef>,
    function_by_name: &HashMap<String, ast::Function>,
    resolved_functions: &mut HashMap<String, resolved_ast::Function>,
    bin_expr: &Located<&BinaryExpr>,
    ptr_sized_int_type: PointerSizedIntWidth,
) -> Result<ResolvedExpression, FaitalError> {
    let lhs = resolve_expression(
        errors,
        types.clone(),
        scopes.clone(),
        type_defs,
        function_by_name,
        resolved_functions,
        bin_expr.lhs.as_deref(),
        None,
        ptr_sized_int_type,
    )?;
    let rhs = resolve_expression(
        errors,
        types.clone(),
        scopes.clone(),
        type_defs,
        function_by_name,
        resolved_functions,
        bin_expr.rhs.as_deref(),
        None,
        ptr_sized_int_type,
    )?;
    match bin_expr.op {
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
            if lhs.ty.is_integer_type() {
                errors.push(CompileError::new(
                    bin_expr.range,
                    CompileErrorKind::InvalidNumericOperand {
                        actual: lhs.ty.clone(),
                    },
                ));
            }
            if rhs.ty.is_integer_type() {
                errors.push(CompileError::new(
                    bin_expr.range,
                    CompileErrorKind::InvalidNumericOperand {
                        actual: rhs.ty.clone(),
                    },
                ));
            }
            let ty = match get_cast_type(ptr_sized_int_type, &lhs.ty, &rhs.ty) {
                (None, None) => todo!(),
                (None, Some(t)) => t,
                (Some(t), None) => t,
                (Some(_), Some(t)) => t,
            };
            Ok(resolved_ast::ResolvedExpression {
                kind: resolved_ast::ExpressionKind::Binary(resolved_ast::BinaryExpr {
                    op: bin_expr.op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }),
                ty,
            })
        }
        BinaryOp::Equals
        | BinaryOp::NotEquals
        | BinaryOp::LessThan
        | BinaryOp::LessThanOrEquals
        | BinaryOp::GreaterThan
        | BinaryOp::GreaterThanOrEquals => {
            if lhs.ty != rhs.ty {
                errors.push(CompileError::new(
                    bin_expr.range,
                    CompileErrorKind::TypeMismatch {
                        expected: lhs.ty.clone(),
                        actual: rhs.ty.clone(),
                    },
                ));
            }
            Ok(resolved_ast::ResolvedExpression {
                kind: resolved_ast::ExpressionKind::Binary(resolved_ast::BinaryExpr {
                    op: bin_expr.op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }),
                ty: ResolvedType::Bool,
            })
        }
    }
}
