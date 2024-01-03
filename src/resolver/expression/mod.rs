mod call;

use std::ops::DerefMut;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::ast::{Expression, Located, TypeDefKind};
use crate::resolved_ast::{ExpressionKind, ResolvedExpression, ResolvedStructType, ResolvedType};
use crate::resolver::ty::resolve_type;
use crate::{ast, in_global_scope, in_new_scope, resolved_ast};

use self::call::resolve_call_expr;

use super::ty::get_resolved_struct_name;
use super::{error::*, mangle_fn_name, resolve_function, TypeScopes, VariableScopes};

pub(crate) fn resolve_expression(
    errors: &mut Vec<CompileError>,
    types: Rc<RefCell<TypeScopes>>,
    scopes: Rc<RefCell<VariableScopes>>,
    type_defs: &HashMap<String, ast::TypeDef>,
    function_by_name: &HashMap<String, ast::Function>,
    resolved_functions: &mut HashMap<String, resolved_ast::Function>,
    loc_expr: Located<&ast::Expression>,
    annotation: Option<ResolvedType>,
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
                    ty.clone()
                };

                return Ok(resolved_ast::ResolvedExpression {
                    ty: resolved_type,
                    kind: expr_kind,
                });
            } else {
                errors.push(CompileError::new(
                    loc_expr.range,
                    CompileErrorKind::VariableNotFound {
                        name: variable_ref.name.to_owned(),
                    },
                ));
                return Ok(ResolvedExpression {
                    ty: ResolvedType::Unknown,
                    kind: expr_kind,
                });
            }
        }
        Expression::NumberLiteral(number_literal) => {
            let kind = resolved_ast::ExpressionKind::NumberLiteral(resolved_ast::NumberLiteral {
                value: number_literal.value.clone(),
            });
            let ty = if let Some(annotation) = annotation {
                annotation
            } else {
                if number_literal.value.parse::<i32>().is_ok() {
                    ResolvedType::I32
                } else if number_literal.value.parse::<i64>().is_ok() {
                    ResolvedType::I64
                } else if number_literal.value.parse::<u64>().is_ok() {
                    ResolvedType::U64
                } else {
                    unreachable!()
                }
            };

            return Ok(ResolvedExpression { ty, kind });
        }
        Expression::BinaryExpr(bin_expr) => {
            let lhs = resolve_expression(
                errors,
                types.clone(),
                scopes.clone(),
                type_defs,
                function_by_name,
                resolved_functions,
                bin_expr.lhs.as_deref(),
                None,
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
            )?;
            return Ok(resolved_ast::ResolvedExpression {
                kind: resolved_ast::ExpressionKind::BinaryExpr(resolved_ast::BinaryExpr {
                    op: bin_expr.op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }),
                ty: ResolvedType::I32,
            });
        }
        Expression::Call(call_expr) => {
            return resolve_call_expr(
                errors,
                types,
                scopes,
                type_defs,
                function_by_name,
                resolved_functions,
                &Located {
                    range: loc_expr.range,
                    value: call_expr,
                },
                annotation,
            )
        }
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
            )?;
            return Ok(resolved_ast::ResolvedExpression {
                kind: resolved_ast::ExpressionKind::Deref(resolved_ast::DerefExpr {
                    target: Box::new(target),
                }),
                ty: ResolvedType::I32,
            });
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
            )?;
            let index = resolve_expression(
                errors,
                types.clone(),
                scopes.clone(),
                type_defs,
                function_by_name,
                resolved_functions,
                index_access_expr.index.as_deref(),
                Some(ResolvedType::USize),
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
            return Ok(resolved_ast::ResolvedExpression {
                kind: resolved_ast::ExpressionKind::IndexAccess(resolved_ast::IndexAccessExor {
                    target: Box::new(target),
                    index: Box::new(index),
                }),
                ty: resolved_ty,
            });
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
            return Ok(resolved_ast::ResolvedExpression {
                kind: resolved_ast::ExpressionKind::FieldAccess(resolved_ast::FieldAccessExpr {
                    target: Box::new(target),
                    field_name: field_access_expr.field_name.clone(),
                }),
                ty: resolved_ty,
            });
        }
        Expression::StringLiteral(str_literal) => {
            return Ok(resolved_ast::ResolvedExpression {
                kind: resolved_ast::ExpressionKind::StringLiteral(resolved_ast::StringLiteral {
                    value: str_literal.value.clone(),
                }),
                ty: ResolvedType::Ptr(Box::new(ResolvedType::U8)),
            });
        }
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
                        Some(expected_ty.clone()),
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
            return Ok(resolved_ast::ResolvedExpression {
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
            });
        }
        Expression::SizeOf(sizeof_expr) => {
            let resolved_ty = resolve_type(
                errors,
                types.borrow_mut().deref_mut(),
                type_defs,
                &sizeof_expr.ty,
            )?;
            return Ok(resolved_ast::ResolvedExpression {
                kind: resolved_ast::ExpressionKind::SizeOf(resolved_ty),
                ty: ResolvedType::USize,
            });
        }
    };
}
