mod assignment;
mod binary;
mod call;
mod variable_decl;

use std::ops::DerefMut;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::ast::{Expression, Located, TypeDefKind};
use crate::common::target::PointerSizedIntWidth;
use crate::resolved_ast::{ExpressionKind, ResolvedExpression, ResolvedStructType, ResolvedType};
use crate::resolver::ty::resolve_type;
use crate::{ast, in_global_scope, in_new_scope, resolved_ast};

use self::assignment::resolve_assignment;
use self::binary::resolve_binary_expression;
use self::call::resolve_call_expr;
use self::variable_decl::resolve_variable_decl;

use super::ty::get_resolved_struct_name;
use super::{
    error::*, mangle_fn_name, resolve_function, BinaryOp, MultiOp, TypeScopes, UnaryOp,
    VariableScopes,
};

pub(crate) fn resolve_expression(
    errors: &mut Vec<CompileError>,
    types: Rc<RefCell<TypeScopes>>,
    scopes: Rc<RefCell<VariableScopes>>,
    type_defs: &HashMap<String, ast::TypeDef>,
    function_by_name: &HashMap<String, ast::Function>,
    resolved_functions: &mut HashMap<String, resolved_ast::Function>,
    loc_expr: Located<&ast::Expression>,
    annotation: Option<&ResolvedType>,
    ptr_sized_int_type: PointerSizedIntWidth,
) -> Result<resolved_ast::ResolvedExpression, FaitalError> {
    match loc_expr.value {
        Expression::VariableRef(variable_ref) => {
            let expr_kind =
                resolved_ast::ExpressionKind::VariableRef(resolved_ast::VariableRefExpr {
                    name: variable_ref.name.clone(),
                });

            if let Some(ty) = scopes.borrow().get(&variable_ref.name) {
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
                errors.push(CompileError::new(
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
        Expression::Binary(bin_expr) => resolve_binary_expression(
            errors,
            types.clone(),
            scopes.clone(),
            type_defs,
            function_by_name,
            resolved_functions,
            &Located::transfer(loc_expr, bin_expr),
            ptr_sized_int_type,
        ),
        Expression::Unary(unary_expr) => {
            let operand = resolve_expression(
                errors,
                types.clone(),
                scopes.clone(),
                type_defs,
                function_by_name,
                resolved_functions,
                unary_expr.operand.as_deref(),
                None,
                ptr_sized_int_type,
            )?;
            if matches!(unary_expr.op, UnaryOp::Not) && !matches!(operand.ty, ResolvedType::Bool) {
                errors.push(CompileError::new(
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
                let resolved_operand = resolve_expression(
                    errors,
                    types.clone(),
                    scopes.clone(),
                    type_defs,
                    function_by_name,
                    resolved_functions,
                    operand.as_deref(),
                    None,
                    ptr_sized_int_type,
                )?;
                resolved_operands.push(resolved_operand);
            }
            match multi_expr.op {
                MultiOp::And | MultiOp::Or => {
                    for operand in &resolved_operands {
                        if !matches!(operand.ty, ResolvedType::Bool) {
                            errors.push(CompileError::new(
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
        Expression::Call(call_expr) => resolve_call_expr(
            errors,
            types,
            scopes,
            type_defs,
            function_by_name,
            resolved_functions,
            &Located::transfer(loc_expr, call_expr),
            annotation,
            ptr_sized_int_type,
        ),
        Expression::DerefExpr(deref_expr) => {
            let target = resolve_expression(
                errors,
                types.clone(),
                scopes.clone(),
                type_defs,
                function_by_name,
                resolved_functions,
                deref_expr.target.as_deref(),
                None,
                ptr_sized_int_type,
            )?;
            Ok(resolved_ast::ResolvedExpression {
                kind: resolved_ast::ExpressionKind::Deref(resolved_ast::DerefExpr {
                    target: Box::new(target),
                }),
                ty: ResolvedType::I32,
            })
        }
        Expression::IndexAccess(index_access_expr) => {
            let target = resolve_expression(
                errors,
                types.clone(),
                scopes.clone(),
                type_defs,
                function_by_name,
                resolved_functions,
                index_access_expr.target.as_deref(),
                None,
                ptr_sized_int_type,
            )?;
            let index = resolve_expression(
                errors,
                types.clone(),
                scopes.clone(),
                type_defs,
                function_by_name,
                resolved_functions,
                index_access_expr.index.as_deref(),
                Some(&ResolvedType::USize),
                ptr_sized_int_type,
            )?;
            let resolved_ty = if let ResolvedType::Ptr(ptr) = &target.ty {
                *ptr.clone()
            } else {
                errors.push(CompileError::new(
                    loc_expr.range,
                    CompileErrorKind::InvalidIndexAccess {
                        ty: target.clone().ty,
                    },
                ));
                ResolvedType::Unknown
            };
            Ok(resolved_ast::ResolvedExpression {
                kind: resolved_ast::ExpressionKind::IndexAccess(resolved_ast::IndexAccessExor {
                    target: Box::new(target),
                    index: Box::new(index),
                }),
                ty: resolved_ty,
            })
        }
        Expression::FieldAccess(field_access_expr) => {
            let target = resolve_expression(
                errors,
                types.clone(),
                scopes.clone(),
                type_defs,
                function_by_name,
                resolved_functions,
                field_access_expr.target.as_deref(),
                None,
                ptr_sized_int_type,
            )?;
            let resolved_ty = if let ResolvedType::Struct(struct_ty) = &target.ty {
                if let Some((_name, ty)) = struct_ty
                    .fields
                    .iter()
                    .find(|x| x.0 == field_access_expr.field_name)
                {
                    ty.clone()
                } else {
                    errors.push(CompileError::new(
                        loc_expr.range,
                        CompileErrorKind::FieldNotFound {
                            field_name: field_access_expr.field_name.clone(),
                            type_name: struct_ty.name.clone(),
                        },
                    ));
                    ResolvedType::Unknown
                }
            } else {
                errors.push(CompileError::new(
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

            let typedef = if let Some(typedef) = type_defs.get(&struct_literal_expr.name) {
                typedef
            } else {
                errors.push(CompileError::new(
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
            let TypeDefKind::Struct(struct_def) = &typedef.kind;

            in_new_scope!(types, {
                if let Some(generic_args_in_def) = &struct_def.generic_args {
                    for (i, generic_arg) in generic_args_in_def.iter().enumerate() {
                        let resolved_generic_arg = resolve_type(
                            errors,
                            types.borrow_mut().deref_mut(),
                            type_defs,
                            &struct_literal_expr.generic_args.as_ref().unwrap()[i],
                        )?;
                        resolved_generic_args.push(resolved_generic_arg.clone());
                        types
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
                        errors.push(CompileError::new(
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
                                errors,
                                types.borrow_mut().deref_mut(),
                                type_defs,
                                &struct_literal_expr.generic_args.as_ref().unwrap()[i],
                            )?;
                            types
                                .borrow_mut()
                                .add(generic_arg.name.clone(), resolved_type);
                        }
                    }

                    let expected_ty =
                        resolve_type(errors, types.borrow_mut().deref_mut(), type_defs, ty)?;
                    let resolved_field = resolve_expression(
                        errors,
                        types.clone(),
                        scopes.clone(),
                        type_defs,
                        function_by_name,
                        resolved_functions,
                        field_in_expr.1.as_deref(),
                        Some(&expected_ty.clone()),
                        ptr_sized_int_type,
                    )?;

                    if !expected_ty.can_insert(&resolved_field.ty) {
                        dbg!(errors.push(CompileError::new(
                            loc_expr.range,
                            CompileErrorKind::TypeMismatch {
                                expected: expected_ty.clone(),
                                actual: resolved_field.ty.clone(),
                            },
                        )));
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
                ty: ResolvedType::Struct(ResolvedStructType {
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
            let resolved_ty = resolve_type(
                errors,
                types.borrow_mut().deref_mut(),
                type_defs,
                &sizeof_expr.ty,
            )?;
            Ok(resolved_ast::ResolvedExpression {
                kind: resolved_ast::ExpressionKind::SizeOf(resolved_ty),
                ty: ResolvedType::USize,
            })
        }
        Expression::If(if_expr) => {
            let condition_expr = resolve_expression(
                errors,
                types.clone(),
                scopes.clone(),
                type_defs,
                function_by_name,
                resolved_functions,
                if_expr.cond.as_deref(),
                Some(&ResolvedType::Bool),
                ptr_sized_int_type,
            )?;
            if !matches!(condition_expr.ty, ResolvedType::Bool) {
                errors.push(CompileError::new(
                    loc_expr.range,
                    CompileErrorKind::TypeMismatch {
                        expected: ResolvedType::Bool,
                        actual: condition_expr.ty.clone(),
                    },
                ));
            }
            let then_expr = resolve_expression(
                errors,
                types.clone(),
                scopes.clone(),
                type_defs,
                function_by_name,
                resolved_functions,
                if_expr.then.as_deref(),
                annotation,
                ptr_sized_int_type,
            )?;
            let else_expr = resolve_expression(
                errors,
                types.clone(),
                scopes.clone(),
                type_defs,
                function_by_name,
                resolved_functions,
                if_expr.els.as_deref(),
                annotation,
                ptr_sized_int_type,
            )?;
            if then_expr.ty != else_expr.ty {
                errors.push(CompileError::new(
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
                errors,
                types.clone(),
                scopes.clone(),
                type_defs,
                function_by_name,
                resolved_functions,
                when_expr.cond.as_deref(),
                Some(&ResolvedType::Bool),
                ptr_sized_int_type,
            )?;
            if !matches!(condition_expr.ty, ResolvedType::Bool) {
                errors.push(CompileError::new(
                    loc_expr.range,
                    CompileErrorKind::TypeMismatch {
                        expected: ResolvedType::Bool,
                        actual: condition_expr.ty.clone(),
                    },
                ));
            }
            let then_expr = resolve_expression(
                errors,
                types.clone(),
                scopes.clone(),
                type_defs,
                function_by_name,
                resolved_functions,
                when_expr.then.as_deref(),
                annotation,
                ptr_sized_int_type,
            )?;
            Ok(resolved_ast::ResolvedExpression {
                ty: ResolvedType::Void,
                kind: resolved_ast::ExpressionKind::When(resolved_ast::WhenExpr {
                    cond: Box::new(condition_expr),
                    then: Box::new(then_expr),
                }),
            })
        }
        Expression::Assignment(assign_expr) => resolve_assignment(
            errors,
            types,
            scopes,
            type_defs,
            function_by_name,
            resolved_functions,
            &Located::transfer(loc_expr, assign_expr),
            ptr_sized_int_type,
        ),
        Expression::VariableDecl(variable_decl_expr) => resolve_variable_decl(
            errors,
            types,
            scopes,
            type_defs,
            function_by_name,
            resolved_functions,
            &Located::transfer(loc_expr, variable_decl_expr),
            ptr_sized_int_type,
        ),
    }
}
