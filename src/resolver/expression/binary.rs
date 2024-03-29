use crate::{common::binary::get_cast_type, resolver::ResolverContext};

use self::ast::BinaryExpr;

use super::*;

pub(super) fn resolve_binary_expression(
    context: &ResolverContext,
    bin_expr: &Located<&BinaryExpr>,
) -> Result<ResolvedExpression, FaitalError> {
    let lhs = resolve_expression(context, bin_expr.lhs.as_deref(), None)?;
    let rhs = resolve_expression(context, bin_expr.rhs.as_deref(), None)?;
    match bin_expr.op {
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
            if !lhs.ty.is_integer_type() {
                context.errors.borrow_mut().push(CompileError::new(
                    bin_expr.range,
                    CompileErrorKind::InvalidNumericOperand {
                        actual: lhs.ty.clone(),
                    },
                ));
            }
            if !rhs.ty.is_integer_type() {
                context.errors.borrow_mut().push(CompileError::new(
                    bin_expr.range,
                    CompileErrorKind::InvalidNumericOperand {
                        actual: rhs.ty.clone(),
                    },
                ));
            }
            let ty: ResolvedType = match get_cast_type(
                &lhs.ty
                    .unwrap_primitive_into_concrete_type(context.is_64_bit()),
                &rhs.ty
                    .unwrap_primitive_into_concrete_type(context.is_64_bit()),
            ) {
                (None, None) => lhs
                    .ty
                    .unwrap_primitive_into_concrete_type(context.is_64_bit()),
                (None, Some(t)) => t,
                (Some(t), None) => t,
                (Some(_), Some(t)) => t,
            }
            .unwrap_primitive_into_resolved_type();
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
                context.errors.borrow_mut().push(CompileError::new(
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
