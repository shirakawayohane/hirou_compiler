mod assignment;
mod binary;
mod call;
mod variable_decl;

use crate::ast::{Expression, Located, TypeDefKind};
use crate::resolved_ast::{
    ExpressionKind, IndexAccessExpr, ResolvedExpression, ResolvedStructType, ResolvedType,
};
use crate::resolver::ty::resolve_type;
use crate::{ast, in_global_scope, in_new_scope, resolved_ast};

use self::assignment::resolve_assignment;
use self::binary::resolve_binary_expression;
use self::call::resolve_call_expr;
use self::variable_decl::resolve_variable_decl;

use super::ty::get_resolved_struct_name;
use super::{
    error::*, mangle_fn_name, resolve_function, BinaryOp, MultiOp, ResolverContext, UnaryOp,
};

pub(crate) fn resolve_expression(
    context: &ResolverContext,
    loc_expr: Located<&ast::Expression>,
    annotation: Option<&ResolvedType>,
) -> Result<resolved_ast::ResolvedExpression, FaitalError> {
    match loc_expr.value {
        Expression::VariableRef(variable_ref) => {
            let expr_kind =
                resolved_ast::ExpressionKind::VariableRef(resolved_ast::VariableRefExpr {
                    name: variable_ref.name.clone(),
                });

            if let Some(ty) = context.scopes.borrow().get(&variable_ref.name) {
                let resolved_type = if let Some(annotation) = annotation {
                    annotation
                } else {
                    ty
                };

                Ok(resolved_ast::ResolvedExpression {
                    ty: resolved_type.clone(),
                    kind: expr_kind,
                })
            } else {
                context.errors.borrow_mut().push(CompileError::new(
                    loc_expr.range,
                    CompileErrorKind::VariableNotFound {
                        name: variable_ref.name.to_owned(),
                    },
                ));
                Ok(ResolvedExpression {
                    ty: ResolvedType::Unknown,
                    kind: expr_kind,
                })
            }
        }
        Expression::NumberLiteral(number_literal) => {
            let kind = resolved_ast::ExpressionKind::NumberLiteral(resolved_ast::NumberLiteral {
                value: number_literal.value.clone(),
            });
            let ty = if let Some(annotation) = annotation {
                annotation.clone()
            } else if number_literal.value.parse::<i32>().is_ok() {
                ResolvedType::I32
            } else if number_literal.value.parse::<i64>().is_ok() {
                ResolvedType::I64
            } else if number_literal.value.parse::<u64>().is_ok() {
                ResolvedType::U64
            } else {
                unreachable!()
            };

            Ok(ResolvedExpression { ty, kind })
        }
        Expression::Binary(bin_expr) => {
            resolve_binary_expression(context, &Located::transfer(loc_expr, bin_expr))
        }
        Expression::Unary(unary_expr) => {
            let operand = resolve_expression(context, unary_expr.operand.as_deref(), None)?;
            if matches!(unary_expr.op, UnaryOp::Not) && !matches!(operand.ty, ResolvedType::Bool) {
                context.errors.borrow_mut().push(CompileError::new(
                    loc_expr.range,
                    CompileErrorKind::TypeMismatch {
                        expected: ResolvedType::Bool,
                        actual: operand.ty.clone(),
                    },
                ));
            }
            Ok(resolved_ast::ResolvedExpression {
                kind: resolved_ast::ExpressionKind::Unary(resolved_ast::UnaryExpr {
                    op: unary_expr.op,
                    operand: Box::new(operand),
                }),
                ty: ResolvedType::Bool,
            })
        }
        Expression::Multi(multi_expr) => {
            let mut resolved_operands = Vec::new();
            for operand in &multi_expr.operands {
                let resolved_operand = resolve_expression(context, operand.as_deref(), None)?;
                resolved_operands.push(resolved_operand);
            }
            match multi_expr.op {
                MultiOp::And | MultiOp::Or => {
                    for operand in &resolved_operands {
                        if !matches!(operand.ty, ResolvedType::Bool) {
                            context.errors.borrow_mut().push(CompileError::new(
                                loc_expr.range,
                                CompileErrorKind::TypeMismatch {
                                    expected: ResolvedType::Bool,
                                    actual: operand.ty.clone(),
                                },
                            ));
                        }
                    }
                    Ok(resolved_ast::ResolvedExpression {
                        kind: resolved_ast::ExpressionKind::Multi(resolved_ast::MultiExpr {
                            op: multi_expr.op,
                            operands: resolved_operands,
                        }),
                        ty: ResolvedType::Bool,
                    })
                }
            }
        }
        Expression::Call(call_expr) => {
            resolve_call_expr(context, &Located::transfer(loc_expr, call_expr), annotation)
        }
        Expression::DerefExpr(deref_expr) => {
            let target = resolve_expression(context, deref_expr.target.as_deref(), None)?;
            Ok(resolved_ast::ResolvedExpression {
                kind: resolved_ast::ExpressionKind::Deref(resolved_ast::DerefExpr {
                    target: Box::new(target),
                }),
                ty: ResolvedType::I32,
            })
        }
        Expression::IndexAccess(index_access_expr) => {
            let target = resolve_expression(context, index_access_expr.target.as_deref(), None)?;
            let index = resolve_expression(
                context,
                index_access_expr.index.as_deref(),
                Some(&ResolvedType::USize),
            )?;
            let resolved_ty = if let ResolvedType::Ptr(ptr) = &target.ty {
                *ptr.clone()
            } else {
                context.errors.borrow_mut().push(CompileError::new(
                    loc_expr.range,
                    CompileErrorKind::InvalidIndexAccess {
                        ty: target.clone().ty,
                    },
                ));
                ResolvedType::Unknown
            };
            Ok(resolved_ast::ResolvedExpression {
                kind: resolved_ast::ExpressionKind::IndexAccess(IndexAccessExpr {
                    target: Box::new(target),
                    index: Box::new(index),
                }),
                ty: resolved_ty,
            })
        }
        Expression::FieldAccess(field_access_expr) => {
            let target = resolve_expression(context, field_access_expr.target.as_deref(), None)?;
            let resolved_ty = if let ResolvedType::StructLike(struct_ty) = &target.ty {
                if let Some((_name, ty)) = struct_ty
                    .fields
                    .iter()
                    .find(|x| x.0 == field_access_expr.field_name)
                {
                    ty.clone()
                } else {
                    context.errors.borrow_mut().push(CompileError::new(
                        loc_expr.range,
                        CompileErrorKind::FieldNotFound {
                            field_name: field_access_expr.field_name.clone(),
                            type_name: struct_ty.name.clone(),
                        },
                    ));
                    ResolvedType::Unknown
                }
            } else {
                context.errors.borrow_mut().push(CompileError::new(
                    loc_expr.range,
                    CompileErrorKind::InvalidFieldAccess {
                        ty: target.clone().ty,
                        name: field_access_expr.field_name.clone(),
                    },
                ));
                ResolvedType::Unknown
            };
            Ok(resolved_ast::ResolvedExpression {
                kind: resolved_ast::ExpressionKind::FieldAccess(resolved_ast::FieldAccessExpr {
                    target: Box::new(target),
                    field_name: field_access_expr.field_name.clone(),
                }),
                ty: resolved_ty,
            })
        }
        Expression::StringLiteral(str_literal) => Ok(resolved_ast::ResolvedExpression {
            kind: resolved_ast::ExpressionKind::StringLiteral(resolved_ast::StringLiteral {
                value: str_literal.value.clone(),
            }),
            ty: ResolvedType::Ptr(Box::new(ResolvedType::U8)),
        }),
        Expression::BoolLiteral(bool_literal) => Ok(resolved_ast::ResolvedExpression {
            kind: resolved_ast::ExpressionKind::BoolLiteral(resolved_ast::BoolLiteral {
                value: bool_literal.value,
            }),
            ty: ResolvedType::Bool,
        }),
        Expression::StructLiteral(struct_literal_expr) => {
            let mut resolved_fields = Vec::new();
            let mut resolved_generic_args = Vec::new();

            let typedef = context
                .type_defs
                .borrow()
                .get(&struct_literal_expr.name)
                .cloned();
            if typedef.is_none() {
                context.errors.borrow_mut().push(CompileError::new(
                    loc_expr.range,
                    CompileErrorKind::TypeNotFound {
                        name: struct_literal_expr.name.clone(),
                    },
                ));
                return Ok(resolved_ast::ResolvedExpression {
                    ty: ResolvedType::Unknown,
                    kind: resolved_ast::ExpressionKind::StructLiteral(
                        resolved_ast::StructLiteral { fields: Vec::new() },
                    ),
                });
            };
            let typedef = typedef.unwrap();
            let TypeDefKind::StructLike(struct_def) = &typedef.kind;

            in_new_scope!(context.types, {
                if let Some(generic_args_in_def) = &struct_def.generic_args {
                    for (i, generic_arg) in generic_args_in_def.iter().enumerate() {
                        let resolved_generic_arg = resolve_type(
                            context,
                            &struct_literal_expr.generic_args.as_ref().unwrap()[i],
                        )?;
                        resolved_generic_args.push(resolved_generic_arg.clone());
                        context
                            .types
                            .borrow_mut()
                            .add(generic_arg.name.clone(), resolved_generic_arg);
                    }
                }
                for (field_name, ty) in &struct_def.fields {
                    let field_in_expr = if let Some(expr) = struct_literal_expr
                        .fields
                        .iter()
                        .find(|x| &x.0 == field_name)
                    {
                        expr
                    } else {
                        context.errors.borrow_mut().push(CompileError::new(
                            loc_expr.range,
                            CompileErrorKind::FieldNotFound {
                                field_name: field_name.clone(),
                                type_name: struct_literal_expr.name.clone(),
                            },
                        ));
                        continue;
                    };
                    if let Some(generic_args) = &struct_def.generic_args {
                        for (i, generic_arg) in generic_args.iter().enumerate() {
                            let resolved_type = resolve_type(
                                context,
                                &struct_literal_expr.generic_args.as_ref().unwrap()[i],
                            )?;
                            context
                                .types
                                .borrow_mut()
                                .add(generic_arg.name.clone(), resolved_type);
                        }
                    }

                    let expected_ty = resolve_type(context, ty)?;
                    let resolved_field = resolve_expression(
                        context,
                        field_in_expr.1.as_deref(),
                        Some(&expected_ty.clone()),
                    )?;

                    if !expected_ty.can_insert(&resolved_field.ty) {
                        context.errors.borrow_mut().push(CompileError::new(
                            loc_expr.range,
                            CompileErrorKind::TypeMismatch {
                                expected: expected_ty.clone(),
                                actual: resolved_field.ty.clone(),
                            },
                        ));
                    }

                    resolved_fields.push((field_name.clone(), resolved_field));
                }
            });

            let struct_name = get_resolved_struct_name(
                &struct_literal_expr.name,
                if struct_def.generic_args.is_some() {
                    Some(&resolved_generic_args)
                } else {
                    None
                },
            );
            Ok(resolved_ast::ResolvedExpression {
                ty: ResolvedType::StructLike(ResolvedStructType {
                    name: struct_name,
                    non_generic_name: typedef.name.clone(),
                    fields: resolved_fields
                        .iter()
                        .map(|(name, expr)| (name.clone(), expr.ty.clone()))
                        .collect(),
                    generic_args: if resolved_generic_args.is_empty() {
                        None
                    } else {
                        Some(resolved_generic_args)
                    },
                }),
                kind: resolved_ast::ExpressionKind::StructLiteral(resolved_ast::StructLiteral {
                    fields: resolved_fields,
                }),
            })
        }
        Expression::SizeOf(sizeof_expr) => {
            let resolved_ty = resolve_type(context, &sizeof_expr.ty)?;
            Ok(resolved_ast::ResolvedExpression {
                kind: resolved_ast::ExpressionKind::SizeOf(resolved_ty),
                ty: ResolvedType::USize,
            })
        }
        Expression::If(if_expr) => {
            let condition_expr =
                resolve_expression(context, if_expr.cond.as_deref(), Some(&ResolvedType::Bool))?;
            if !matches!(condition_expr.ty, ResolvedType::Bool) {
                context.errors.borrow_mut().push(CompileError::new(
                    loc_expr.range,
                    CompileErrorKind::TypeMismatch {
                        expected: ResolvedType::Bool,
                        actual: condition_expr.ty.clone(),
                    },
                ));
            }
            let then_expr = resolve_expression(context, if_expr.then.as_deref(), annotation)?;
            let else_expr = resolve_expression(context, if_expr.els.as_deref(), annotation)?;
            if then_expr.ty != else_expr.ty {
                context.errors.borrow_mut().push(CompileError::new(
                    loc_expr.range,
                    CompileErrorKind::TypeMismatch {
                        expected: then_expr.ty.clone(),
                        actual: else_expr.ty.clone(),
                    },
                ));
            }
            Ok(resolved_ast::ResolvedExpression {
                ty: then_expr.ty.clone(),
                kind: resolved_ast::ExpressionKind::If(resolved_ast::IfExpr {
                    cond: Box::new(condition_expr),
                    then: Box::new(then_expr),
                    els: Box::new(else_expr),
                }),
            })
        }
        Expression::When(when_expr) => {
            let condition_expr = resolve_expression(
                context,
                when_expr.cond.as_deref(),
                Some(&ResolvedType::Bool),
            )?;
            if !matches!(condition_expr.ty, ResolvedType::Bool) {
                context.errors.borrow_mut().push(CompileError::new(
                    loc_expr.range,
                    CompileErrorKind::TypeMismatch {
                        expected: ResolvedType::Bool,
                        actual: condition_expr.ty.clone(),
                    },
                ));
            }
            let then_expr = resolve_expression(context, when_expr.then.as_deref(), annotation)?;
            Ok(resolved_ast::ResolvedExpression {
                ty: ResolvedType::Void,
                kind: resolved_ast::ExpressionKind::When(resolved_ast::WhenExpr {
                    cond: Box::new(condition_expr),
                    then: Box::new(then_expr),
                }),
            })
        }
        Expression::Assignment(assign_expr) => {
            resolve_assignment(context, &Located::transfer(loc_expr, assign_expr))
        }
        Expression::VariableDecl(variable_decl_expr) => {
            resolve_variable_decl(context, &Located::transfer(loc_expr, variable_decl_expr))
        }
    }
}
