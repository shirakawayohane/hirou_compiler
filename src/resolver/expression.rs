use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::ast::Expression;
use crate::resolved_ast::{ExpressionKind, ResolvedExpression, ResolvedType};
use crate::{ast, resolved_ast};

use super::{error::*, gen_function_impls_recursively, mangle_fn_name, resolve_type, Scopes};

pub(crate) fn resolve_expression(
    errors: &mut Vec<CompileError>,
    types: Rc<RefCell<HashMap<String, &ResolvedType>>>,
    scopes: Rc<RefCell<Scopes>>,
    function_by_name: &HashMap<String, ast::Function>,
    resolved_functions: &mut HashMap<String, resolved_ast::Function>,
    expr: &ast::Expression,
    annotation: Option<ResolvedType>,
) -> Result<resolved_ast::ResolvedExpression, FaitalError> {
    match expr {
        Expression::VariableRef(variable_ref) => {
            let expr_kind =
                resolved_ast::ExpressionKind::VariableRef(resolved_ast::VariableRefExpr {
                    name: variable_ref.name.clone(),
                });

            if variable_ref.name == "size" {
                dbg!(scopes.clone());
            }
            if let Some(ty) = scopes.borrow().get(&variable_ref.name) {
                let resolved_type = if let Some(annotation) = annotation {
                    if variable_ref.name == "size" {
                        dbg!(annotation.clone());
                    }
                    annotation
                } else {
                    if variable_ref.name == "size" {
                        dbg!(ty);
                    }
                    ty.clone()
                };

                return Ok(resolved_ast::ResolvedExpression {
                    ty: resolved_type,
                    kind: expr_kind,
                });
            } else {
                errors.push(CompileError::from_error_kind(
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
                function_by_name,
                resolved_functions,
                &bin_expr.lhs,
                None,
            )?;
            let rhs = resolve_expression(
                errors,
                types.clone(),
                scopes.clone(),
                function_by_name,
                resolved_functions,
                &bin_expr.rhs,
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
            let callee = function_by_name
                .get(&call_expr.name)
                .ok_or_else(|| FaitalError(format!("No function named {}", call_expr.name)))?;

            scopes.borrow_mut().push_scope();
            if let Some(generic_args) = &callee.decl.generic_args {
                // スコープに解決したジェネリックの型を追加する
                if let Some(actual_generic_args) = &call_expr.generic_args {
                    if generic_args.len() != actual_generic_args.len() {
                        errors.push(CompileError::from_error_kind(
                            CompileErrorKind::MismatchGenericArgCount {
                                fn_name: call_expr.name.to_owned(),
                                expected: generic_args.len() as u32,
                                actual: actual_generic_args.len() as u32,
                            },
                        ));
                        gen_function_impls_recursively(
                            errors,
                            types.clone(),
                            scopes.clone(),
                            function_by_name,
                            resolved_functions,
                            callee,
                        )?;
                        // ジェネリック引数の数が合わない場合はUnknown扱いにして継続する
                        return Ok(ResolvedExpression {
                            kind: ExpressionKind::CallExpr(resolved_ast::CallExpr {
                                callee: call_expr.name.clone(),
                                args: Vec::new(),
                            }),
                            ty: ResolvedType::Unknown,
                        });
                    };

                    for i in 0..generic_args.len() {
                        let generic_arg = &generic_args[i];
                        let actual_arg = &actual_generic_args[i];
                        let resolved_type = resolve_type(errors, types.clone(), &actual_arg)?;
                        scopes
                            .borrow_mut()
                            .insert(generic_arg.value.name.clone(), resolved_type);
                    }
                } else {
                    errors.push(CompileError::from_error_kind(
                        CompileErrorKind::NoGenericArgs {
                            fn_name: call_expr.name.to_owned(),
                            expected: generic_args.len() as u32,
                        },
                    ));
                    gen_function_impls_recursively(
                        errors,
                        types.clone(),
                        scopes.clone(),
                        function_by_name,
                        resolved_functions,
                        callee,
                    )?;
                    return Ok(ResolvedExpression {
                        kind: ExpressionKind::CallExpr(resolved_ast::CallExpr {
                            callee: call_expr.name.clone(),
                            args: Vec::new(),
                        }),
                        ty: ResolvedType::Unknown,
                    });
                };
            };
            // 上記でスコープに実際の型が登録されているはず。
            // 未生成であれば関数の実装を生成する
            gen_function_impls_recursively(
                errors,
                types.clone(),
                scopes.clone(),
                function_by_name,
                resolved_functions,
                callee,
            )?;

            let resolved_return_ty =
                resolve_type(errors, types.clone(), &callee.decl.return_type.value)?;
            let mut resolved_args = Vec::new();
            for (i, arg) in call_expr.args.iter().enumerate() {
                let calee_arg = &callee.decl.args[i];
                match calee_arg {
                    ast::Argument::VarArgs => {
                        resolved_args.push(resolve_expression(
                            errors,
                            types.clone(),
                            scopes.clone(),
                            function_by_name,
                            resolved_functions,
                            arg,
                            None,
                        )?);
                    }
                    ast::Argument::Normal(ty, _name) => {
                        let resolved_ty = resolve_type(errors, types.clone(), ty)?;

                        if (callee.decl.name == "malloc") {
                            dbg!(resolved_ty.clone(), arg);
                        }
                        resolved_args.push(resolve_expression(
                            errors,
                            types.clone(),
                            scopes.clone(),
                            function_by_name,
                            resolved_functions,
                            arg,
                            Some(resolved_ty),
                        )?);
                    }
                }
            }

            if (callee.decl.name == "malloc") {
                dbg!(resolved_args.clone());
            }

            scopes.borrow_mut().pop_scope();

            return Ok(resolved_ast::ResolvedExpression {
                kind: resolved_ast::ExpressionKind::CallExpr(resolved_ast::CallExpr {
                    callee: mangle_fn_name(
                        &call_expr.name,
                        resolved_args
                            .iter()
                            .map(|x| &x.ty)
                            .collect::<Vec<_>>()
                            .as_slice(),
                        &resolved_return_ty,
                    ),
                    args: resolved_args,
                }),
                ty: resolved_return_ty,
            });
        }
        Expression::DerefExpr(deref_expr) => {
            let target = resolve_expression(
                errors,
                types.clone(),
                scopes.clone(),
                function_by_name,
                resolved_functions,
                &deref_expr.target,
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
                function_by_name,
                resolved_functions,
                &index_access_expr.target,
                None,
            )?;
            let index = resolve_expression(
                errors,
                types.clone(),
                scopes.clone(),
                function_by_name,
                resolved_functions,
                &index_access_expr.index,
                Some(ResolvedType::USize),
            )?;
            let resolved_ty = if let ResolvedType::Ptr(ptr) = &target.ty {
                *ptr.clone()
            } else {
                errors.push(CompileError::from_error_kind(
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
        Expression::StringLiteral(str_literal) => {
            return Ok(resolved_ast::ResolvedExpression {
                kind: resolved_ast::ExpressionKind::StringLiteral(resolved_ast::StringLiteral {
                    value: str_literal.value.clone(),
                }),
                ty: ResolvedType::Ptr(Box::new(ResolvedType::Ptr(Box::new(ResolvedType::U8)))),
            });
        }
    };
}
