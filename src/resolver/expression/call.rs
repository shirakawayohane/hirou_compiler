use crate::{ast::UnresolvedType, resolver::ResolverContext};

use super::*;

pub fn resolve_call_with_generic_args(
    context: &ResolverContext,
    call_expr: &Located<&ast::CallExpr>,
    callee: &ast::Function,
    // 推論に成功した場合のみtrueを返す
) -> Result<bool, FaitalError> {
    if let Some(generic_args) = &callee.decl.generic_args {
        // スコープに解決したジェネリックの型を追加する
        if let Some(actual_generic_args) = &call_expr.generic_args {
            if generic_args.len() != actual_generic_args.len() {
                context.errors.borrow_mut().push(CompileError::new(
                    call_expr.range,
                    CompileErrorKind::MismatchGenericArgCount {
                        name: call_expr.name.to_owned(),
                        expected: generic_args.len(),
                        actual: actual_generic_args.len(),
                    },
                ));
                return Ok(false);
            };

            in_global_scope!(context.scopes, {
                in_global_scope!(context.types, {
                    // 正常系
                    for i in 0..generic_args.len() {
                        let generic_arg = &generic_args[i];
                        let actual_arg = &actual_generic_args[i];
                        let resolved_type = resolve_type(context, actual_arg)?;
                        context
                            .types
                            .borrow_mut()
                            .add(generic_arg.name.clone(), resolved_type);
                    }
                    resolve_function(context, callee)?;
                });
            });
            return Ok(true);
        }
    }
    Ok(false)
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
    }
    Ok(false)
}
// 関数のジェネリック引数を引数から推論する関数
pub fn resolve_infer_generic_from_arguments(
    context: &ResolverContext,
    call_expr: &Located<&ast::CallExpr>,
    callee: &ast::Function,
    resolved_args: &[ResolvedExpression], // 推論に成功した場合のみtrueを返す
) -> Result<bool, FaitalError> {
    // ジェネリック引数が存在しない場合は、推論を行わない
    if call_expr.generic_args.is_some() || callee.decl.generic_args.is_none() {
        return Ok(false);
    }
    // 可変長引数を持つ関数は、推論を行わない
    if callee
        .decl
        .args
        .iter()
        .any(|x| matches!(x, ast::Argument::VarArgs))
    {
        return Ok(false);
    }
    let mut inferred = false;
    let mut temp_errors = Vec::new();
    // グローバルスコープで処理を行う
    in_global_scope!(context.scopes, {
        in_global_scope!(context.types, {
            for (arg_idx, _arg) in call_expr.args.iter().enumerate() {
                let callee_arg = &callee.decl.args[arg_idx];
                // 引数にジェネリクスを含まない場合は推論しない
                // 含む場合、unknownになるはずなので推論する
                match callee_arg {
                    ast::Argument::VarArgs => unreachable!(),
                    ast::Argument::Normal(callee_ty, _name) => {
                        if infer_generic_args_recursively(
                            &mut temp_errors,
                            context,
                            callee,
                            callee_ty,
                            &resolved_args[arg_idx].ty.clone(),
                        )? {
                            inferred = true;
                        }
                    }
                }
            }
            // 関数の解決を試みる
            resolve_function(context, callee)?;
        });
    });

    // 推論が成功した場合、一時エラーを追加する
    if inferred {
        context.errors.borrow_mut().extend(temp_errors);
    }

    Ok(inferred)
}

// 関数のジェネリック引数をアノテーションから推論する関数
pub fn resolve_infer_generic_from_annotation(
    context: &ResolverContext,
    call_expr: &Located<&ast::CallExpr>,
    callee: &ast::Function,
    annotation: Option<&ResolvedType>,
    // 推論に成功した場合のみtrueを返す
) -> Result<bool, FaitalError> {
    // ジェネリック引数が存在しない場合は、推論を行わない
    if call_expr.generic_args.is_some() || callee.decl.generic_args.is_none() {
        return Ok(false);
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
                Ok(inferred)
            })
        })
    } else {
        Ok(false)
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

// 関数呼び出し式の解決を試みる関数
pub fn resolve_call_expr(
    context: &ResolverContext,
    call_expr: &Located<&ast::CallExpr>,
    annotation: Option<&ResolvedType>,
) -> Result<ResolvedExpression, FaitalError> {
    // 関数名から関数を取得し、見つからない場合はエラーを返す
    let binding = context.function_by_name.borrow();
    let callee = match binding.get(&call_expr.name) {
        Some(callee) => callee,
        None => {
            context.errors.borrow_mut().push(CompileError::new(
                call_expr.range,
                CompileErrorKind::FunctionNotFound {
                    name: call_expr.name.to_owned(),
                },
            ));
            return Ok(ResolvedExpression {
                ty: ResolvedType::Unknown,
                kind: ExpressionKind::Unknown,
            });
        }
    };

    // ジェネリック引数を持たない関数、ジェネリック引数を持つ関数、アノテーションからの推論を試みる
    let mut inferred = true;
    if !resolve_non_generic_function(context, callee)?
        && !resolve_call_with_generic_args(context, call_expr, callee)?
        && !resolve_infer_generic_from_annotation(context, call_expr, callee, annotation)?
    {
        inferred = false;
    }

    // 引数の解決を試みる
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

    // 推論が失敗した場合、引数からの推論を試みる
    if !inferred
        && !resolve_infer_generic_from_arguments(context, call_expr, callee, &resolved_args)?
    {
        context.errors.borrow_mut().push(CompileError::new(
            call_expr.range,
            CompileErrorKind::CannotInferGenericArgs {
                name: call_expr.name.to_owned(),
                message: "".to_string(),
            },
        ));
        return Ok(ResolvedExpression {
            ty: resolve_type(context, &callee.decl.return_type)?,
            kind: ExpressionKind::Unknown,
        });
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
    };

    // varargsはintrinsicでしか定義しないので、引数の最後に来ないケースは想定しない。
    if call_expr.args.len() != callee.decl.args.len() && !has_var_args {
        context.errors.borrow_mut().push(CompileError::new(
            call_expr.range,
            CompileErrorKind::MismatchFunctionArgCount {
                name: call_expr.name.to_owned(),
                expected: callee.decl.args.len(),
                actual: call_expr.args.len(),
            },
        ));
        return Ok(ResolvedExpression {
            ty: resolved_return_ty,
            kind: ExpressionKind::Unknown,
        });
    }

    // 解決された式を返す
    return Ok(resolved_ast::ResolvedExpression {
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
    });
}
