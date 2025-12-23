use itertools::Itertools;

use crate::{
    ast::UnresolvedType,
    resolver::{generics::check_generic_bounds, resolve_implementation, ResolverContext},
};

use super::*;

/// Resolves a function call with explicit generic arguments.
/// Returns true if resolution was successful, false if generic args were missing or mismatched.
pub fn resolve_call_with_generic_args(
    context: &ResolverContext,
    call_expr: &Located<&ast::CallExpr>,
    callee: &ast::Function,
) -> Result<bool, FaitalError> {
    let Some(callee_generic_args) = &callee.decl.generic_args else {
        return Ok(false);
    };
    let Some(call_generic_args) = &call_expr.generic_args else {
        return Ok(false);
    };

    // Check generic argument count matches
    if call_generic_args.len() != callee_generic_args.len() {
        context.errors.borrow_mut().push(CompileError::new(
            call_expr.range,
            CompileErrorKind::MismatchGenericArgCount {
                name: call_expr.name.to_owned(),
                expected: callee_generic_args.len(),
                actual: call_generic_args.len(),
            },
        ));
        return Ok(false);
    }

    // Resolve and register generic arguments
    let mut resolved_generic_args = Vec::new();
    for (i, call_generic_arg) in call_generic_args.iter().enumerate() {
        let resolved_generic_arg = resolve_type(context, call_generic_arg)?;
        resolved_generic_args.push(resolved_generic_arg.clone());
        context.types.borrow_mut().add(
            callee_generic_args[i].name.clone(),
            resolved_generic_arg,
        );
    }

    // Check interface bounds for generic arguments
    if let Err(e) = check_generic_bounds(
        context,
        callee_generic_args,
        &resolved_generic_args,
        &resolved_generic_args,
    ) {
        context.errors.borrow_mut().push(e);
    }

    // Resolve the function with concrete type parameters
    resolve_function(context, callee)?;
    Ok(true)
}

fn infer_generic_args_recursively(
    tmp_errors: &mut Vec<CompileError>,
    context: &ResolverContext,
    callee: &ast::Function,
    current_callee_return_ty: &UnresolvedType,
    current_annotation: &ResolvedType,
) -> Result<bool, FaitalError> {
    let callee_generic_args = callee.decl.generic_args.as_ref().unwrap();
    match current_callee_return_ty {
        UnresolvedType::TypeRef(return_ty_typeref) => {
            if return_ty_typeref.generic_args.is_none() {
                if let Some(generic_arg) = callee_generic_args
                    .iter()
                    .find(|x| x.value.name == return_ty_typeref.name)
                {
                    context
                        .types
                        .borrow_mut()
                        .add(generic_arg.value.name.clone(), current_annotation.clone());
                    return Ok(true);
                } else {
                    return Ok(false);
                }
            }
            if let Some(generic_args) = &return_ty_typeref.generic_args {
                match current_annotation {
                    ResolvedType::StructLike(resolved_struct) => {
                        if resolved_struct.non_generic_name == return_ty_typeref.name {
                            let mut generic_arg_inferred = false;
                            for (i, resolved_generic_ty) in
                                resolved_struct.generic_args.iter().enumerate()
                            {
                                if infer_generic_args_recursively(
                                    tmp_errors,
                                    context,
                                    callee,
                                    &generic_args[i],
                                    &resolved_generic_ty[i],
                                )? {
                                    generic_arg_inferred = true;
                                }
                            }
                            if generic_arg_inferred {
                                return Ok(true);
                            }
                        } else {
                            return Ok(false);
                        }
                    }
                    _ => {
                        return Ok(false);
                    }
                }
            }
        }
        UnresolvedType::Ptr(return_ty_pointer_ty) => {
            if let ResolvedType::Ptr(inner) = current_annotation {
                if infer_generic_args_recursively(
                    tmp_errors,
                    context,
                    callee,
                    return_ty_pointer_ty,
                    inner,
                )? {
                    return Ok(true);
                }
            }
        }
        UnresolvedType::Infer => {
            return Ok(false);
        }
    }
    Ok(false)
}

// 関数のジェネリック引数を引数から推論する関数
// TODO: 実際の推論ロジックを実装する
pub fn resolve_infer_generic_from_arguments(
    context: &ResolverContext,
    call_expr: &Located<&ast::CallExpr>,
    callee: &ast::Function,
    resolved_args: &[ResolvedExpression],
) -> Result<Vec<usize>, FaitalError> {
    // ジェネリック引数が既に指定されている場合、または宣言されていない場合は推論を行わない
    if call_expr.generic_args.is_some() {
        return Ok(vec![]);
    }
    let Some(callee_generic_args) = &callee.decl.generic_args else {
        return Ok(vec![]);
    };
    // 可変長引数を持つ関数は、推論を行わない
    if callee.decl.args.iter().any(|x| matches!(x, ast::Argument::VarArgs)) {
        return Ok(vec![]);
    }

    let mut inferred_indices = Vec::new();

    // 各引数からジェネリック型を推論
    for (i, callee_arg) in callee.decl.args.iter().enumerate() {
        if let ast::Argument::Normal(arg_ty, _) = callee_arg {
            if let UnresolvedType::TypeRef(typeref) = &arg_ty.value {
                // 引数の型がジェネリックパラメータ名と一致するか確認
                for (gen_idx, gen_arg) in callee_generic_args.iter().enumerate() {
                    if typeref.name == gen_arg.name {
                        // 実引数から型を取得し、ジェネリック型として登録
                        if let Some(resolved_arg) = resolved_args.get(i) {
                            context.types.borrow_mut().add(
                                gen_arg.name.clone(),
                                resolved_arg.ty.clone(),
                            );
                            if !inferred_indices.contains(&gen_idx) {
                                inferred_indices.push(gen_idx);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(inferred_indices)
}

// 関数のジェネリック引数をアノテーションから推論する関数
pub fn resolve_infer_generic_from_annotation(
    context: &ResolverContext,
    call_expr: &Located<&ast::CallExpr>,
    callee: &ast::Function,
    annotation: Option<&ResolvedType>,
    // 推論に成功したジェネリクス引数のインデックスを返す
) -> Result<Vec<usize>, FaitalError> {
    // ジェネリック引数が存在しない場合は、推論を行わない
    if call_expr.generic_args.is_some() || callee.decl.generic_args.is_none() {
        return Ok(vec![]);
    }
    // アノテーションが存在する場合、推論を試みる
    if let Some(annotation) = &annotation {
        in_global_scope!(context.scopes, {
            in_global_scope!(context.types, {
                let mut temp_errors = Vec::new();
                let inferred = infer_generic_args_recursively(
                    &mut temp_errors,
                    context,
                    callee,
                    &callee.decl.return_type.value,
                    annotation,
                )?;
                // 推論が成功した場合、一時エラーを追加し、関数の解決を試みる
                if inferred {
                    context.errors.borrow_mut().extend(temp_errors);
                    resolve_function(context, callee)?;
                }
                Ok((0..callee.decl.generic_args.as_ref().unwrap().len()).collect_vec())
            })
        })
    } else {
        Ok(vec![])
    }
}

// ジェネリック引数を持たない関数の解決を試みる関数
pub fn resolve_non_generic_function(
    context: &ResolverContext,
    callee: &ast::Function,
    // 推論に成功した場合のみtrueを返す
) -> Result<bool, FaitalError> {
    // ジェネリック引数が存在する場合は、解決を行わない
    if callee.decl.generic_args.is_some() {
        return Ok(false);
    }
    // グローバルスコープで関数の解決を試みる
    in_global_scope!(context.scopes, {
        in_global_scope!(context.types, {
            resolve_function(context, callee)?;
        });
    });
    Ok(true)
}

fn resolve_function_call_expr(
    context: &ResolverContext,
    call_expr: &Located<&ast::CallExpr>,
    callee: &ast::Function,
    annotation: Option<&ResolvedType>,
) -> Result<ResolvedExpression, FaitalError> {
    let mut resolved_args = Vec::new();
    let has_var_args = callee.decl.args.last() == Some(&ast::Argument::VarArgs);

    // 可変長引数を持たない場合、引数の数が一致しなければエラーを返す
    if !has_var_args && callee.decl.args.len() != call_expr.args.len() {
        context.errors.borrow_mut().push(CompileError::new(
            call_expr.range,
            CompileErrorKind::MismatchFunctionArgCount {
                name: call_expr.name.to_owned(),
                expected: callee.decl.args.len(),
                actual: call_expr.args.len(),
            },
        ));
        return Ok(ResolvedExpression {
            ty: ResolvedType::Unknown,
            kind: ExpressionKind::Unknown,
        });
    }

    // ジェネリック関数の解決
    if callee.decl.generic_args.is_some() {
        // 1. 明示的なジェネリック引数がある場合
        let explicit_resolved = resolve_call_with_generic_args(context, call_expr, callee)?;

        // 2. アノテーションからの推論を試みる
        let annotation_inferred = if !explicit_resolved {
            !resolve_infer_generic_from_annotation(context, call_expr, callee, annotation)?
                .is_empty()
        } else {
            false
        };

        // 3. 引数からの推論を試みる
        if !explicit_resolved && !annotation_inferred {
            // まず引数をアノテーションなしで解決（ジェネリック推論のため）
            let mut pre_resolved_args = Vec::new();
            for arg in &call_expr.args {
                pre_resolved_args.push(resolve_expression(context, arg.as_inner_deref(), None)?);
            }

            let inferred_indices = resolve_infer_generic_from_arguments(
                context,
                call_expr,
                callee,
                &pre_resolved_args,
            )?;

            // 推論されたジェネリクスの境界をチェック
            if !inferred_indices.is_empty() {
                if let Some(callee_generic_args) = &callee.decl.generic_args {
                    let resolved_generic_args: Vec<_> = callee_generic_args
                        .iter()
                        .map(|g| {
                            context
                                .types
                                .borrow()
                                .get(&g.name)
                                .cloned()
                                .unwrap_or(ResolvedType::Unknown)
                        })
                        .collect();
                    if let Err(e) = check_generic_bounds(
                        context,
                        callee_generic_args,
                        &resolved_generic_args,
                        &resolved_generic_args,
                    ) {
                        context.errors.borrow_mut().push(e);
                    }
                }
            }
        }
    } else {
        // 非ジェネリック関数の解決
        resolve_non_generic_function(context, callee)?;
    }

    // 各引数を解決し、型の不一致があればエラーを返す
    for (i, arg) in call_expr.args.iter().enumerate() {
        let callee_arg = if has_var_args && i >= callee.decl.args.len() {
            &callee.decl.args[callee.decl.args.len() - 1]
        } else {
            &callee.decl.args[i]
        };
        match callee_arg {
            ast::Argument::VarArgs => {
                resolved_args.push(resolve_expression(context, arg.as_inner_deref(), None)?);
            }
            ast::Argument::SelfArg => {
                // SelfArg is only valid in interface implementations
                unreachable!("SelfArg is not allowed in regular function calls")
            }
            ast::Argument::Normal(ty, _name) => {
                let resolved_ty = resolve_type(context, ty)?;
                let resolved_arg =
                    resolve_expression(context, arg.as_inner_deref(), Some(&resolved_ty))?;
                if !resolved_ty.can_insert(&resolved_arg.ty) {
                    context.errors.borrow_mut().push(CompileError::new(
                        arg.range,
                        CompileErrorKind::TypeMismatch {
                            expected: resolved_ty.clone(),
                            actual: resolved_arg.ty.clone(),
                        },
                    ));
                }
                resolved_args.push(resolved_arg);
            }
        }
    }

    // 戻り値の型を解決する
    let mut resolved_return_ty = resolve_type(context, &callee.decl.return_type)?;
    // void* はアノテーションがあればその型として扱う
    if let Some(annotation) = annotation {
        if let ResolvedType::Ptr(inner) = &resolved_return_ty {
            if let ResolvedType::Void = **inner {
                resolved_return_ty = annotation.clone();
            }
        }
    }

    // 解決された式を返す
    Ok(resolved_ast::ResolvedExpression {
        kind: resolved_ast::ExpressionKind::CallExpr(resolved_ast::CallExpr {
            callee: if callee.decl.generic_args.is_some() {
                mangle_fn_name(
                    &call_expr.name,
                    resolved_args
                        .iter()
                        .map(|x| &x.ty)
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &resolved_return_ty,
                )
            } else {
                call_expr.name.clone()
            },
            args: resolved_args,
        }),
        ty: resolved_return_ty,
    })
}

// 関数呼び出し式の解決を試みる関数
pub fn resolve_call_expr(
    context: &ResolverContext,
    call_expr: &Located<&ast::CallExpr>,
    annotation: Option<&ResolvedType>,
) -> Result<ResolvedExpression, FaitalError> {
    // 関数名から関数を取得し、見つからない場合はエラーを返す
    let function_by_name = context.function_by_name.borrow();
    let interface_by_name = context.interface_by_name.borrow();
    let impls_by_name = context.impls_by_name.borrow();
    if let Some(callee) = function_by_name.get(&call_expr.name) {
        resolve_function_call_expr(context, call_expr, callee, annotation)
    } else if let Some(interface) = interface_by_name.get(&call_expr.name) {
        let mut resolved_arg_types = vec![];
        for arg in &call_expr.args {
            resolved_arg_types.push(resolve_expression(context, arg.as_inner_deref(), None)?.ty);
        }
        let mut generic_annotations: Vec<ResolvedType> = vec![];
        if let Some(generic_args) = &call_expr.generic_args {
            for generic_arg in generic_args {
                generic_annotations.push(resolve_type(context, generic_arg)?);
            }
        } else {
            let required_generic_args_len = interface.generic_args.len();
            for _ in 0..required_generic_args_len {
                generic_annotations.push(ResolvedType::Unknown);
            }
        }
        if let Some(impls) = impls_by_name.get(&interface.name) {
            // Find the implementation that matches the argument type
            if let Some(implementation) = impls.iter().find(|implementation| {
                let resolved_target_ty =
                    resolve_type(context, &implementation.decl.target_ty).unwrap();
                // Check if the first argument type matches the implementation's target type
                if let Some(first_arg_ty) = resolved_arg_types.first() {
                    *first_arg_ty == resolved_target_ty
                } else {
                    false
                }
            }) {
                // Resolve the implementation and generate a function call
                let resolved_target_ty =
                    resolve_type(context, &implementation.decl.target_ty).unwrap();
                let impl_fn_name = format!(
                    "impl_{}_for_{}",
                    interface.name.replace("->", "to_"),
                    resolved_target_ty.to_string()
                );

                // Resolve implementation body as a function
                resolve_implementation(context, implementation, &impl_fn_name)?;

                // Resolve the return type from interface
                let resolved_return_ty = resolve_type(context, &interface.return_type)?;

                // Generate call expression to the implementation function
                let resolved_args: Vec<ResolvedExpression> = call_expr
                    .args
                    .iter()
                    .map(|arg| resolve_expression(context, arg.as_inner_deref(), None))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(ResolvedExpression {
                    kind: ExpressionKind::CallExpr(resolved_ast::CallExpr {
                        callee: impl_fn_name,
                        args: resolved_args,
                    }),
                    ty: resolved_return_ty,
                })
            } else {
                context.errors.borrow_mut().push(CompileError::new(
                    call_expr.range,
                    CompileErrorKind::InterfaceNotImplemented {
                        name: call_expr.name.to_owned(),
                        ty: resolved_arg_types.first().cloned().unwrap_or(ResolvedType::Unknown),
                    },
                ));
                Ok(ResolvedExpression {
                    ty: ResolvedType::Unknown,
                    kind: ExpressionKind::Unknown,
                })
            }
        } else {
            context.errors.borrow_mut().push(CompileError::new(
                call_expr.range,
                CompileErrorKind::InterfaceNotImplemented {
                    name: call_expr.name.to_owned(),
                    ty: ResolvedType::Unknown,
                },
            ));
            Ok(ResolvedExpression {
                ty: ResolvedType::Unknown,
                kind: ExpressionKind::Unknown,
            })
        }
    } else {
        context.errors.borrow_mut().push(CompileError::new(
            call_expr.range,
            CompileErrorKind::FunctionNotFound {
                name: call_expr.name.to_owned(),
            },
        ));
        Ok(ResolvedExpression {
            ty: ResolvedType::Unknown,
            kind: ExpressionKind::Unknown,
        })
    }
}
