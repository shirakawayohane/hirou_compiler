use crate::{ast::UnresolvedType, resolver::expression};

use super::*;

pub fn resolve_call_with_generic_args(
    errors: &mut Vec<CompileError>,
    types: Rc<RefCell<TypeScopes>>,
    scopes: Rc<RefCell<VariableScopes>>,
    type_defs: &HashMap<String, ast::TypeDef>,
    function_by_name: &HashMap<String, ast::Function>,
    resolved_functions: &mut HashMap<String, resolved_ast::Function>,
    call_expr: &ast::CallExpr,
    callee: &ast::Function,
    // 推論に成功した場合のみtrueを返す
) -> Result<bool, FaitalError> {
    if let Some(generic_args) = &callee.decl.generic_args {
        // スコープに解決したジェネリックの型を追加する
        if let Some(actual_generic_args) = &call_expr.generic_args {
            if generic_args.len() != actual_generic_args.len() {
                dbg!(errors.push(CompileError::from_error_kind(
                    CompileErrorKind::MismatchGenericArgCount {
                        name: call_expr.name.to_owned(),
                        expected: generic_args.len(),
                        actual: actual_generic_args.len(),
                    },
                )));
                return Ok(false);
            };

            in_global_scope!(scopes, {
                in_global_scope!(types, {
                    // 正常系
                    for i in 0..generic_args.len() {
                        let generic_arg = &generic_args[i];
                        let actual_arg = &actual_generic_args[i];
                        let resolved_type = resolve_type(
                            errors,
                            types.borrow_mut().deref_mut(),
                            type_defs,
                            &actual_arg,
                        )?;
                        types
                            .borrow_mut()
                            .add(generic_arg.name.clone(), resolved_type);
                    }
                    resolve_function(
                        errors,
                        types.clone(),
                        scopes.clone(),
                        type_defs,
                        function_by_name,
                        resolved_functions,
                        callee,
                    )?;
                });
            });
            return Ok(true);
        }
    }
    Ok(false)
}

fn infer_generic_args_recursively(
    errors: &mut Vec<CompileError>,
    types: Rc<RefCell<TypeScopes>>,
    scopes: Rc<RefCell<VariableScopes>>,
    type_defs: &HashMap<String, ast::TypeDef>,
    function_by_name: &HashMap<String, ast::Function>,
    resolved_functions: &mut HashMap<String, resolved_ast::Function>,
    callee: &ast::Function,
    current_callee_return_ty: &UnresolvedType,
    current_annotation: &ResolvedType,
) -> Result<bool, FaitalError> {
    let callee_name = &callee.decl.name;
    let callee_generic_args = callee.decl.generic_args.as_ref().unwrap();
    match current_callee_return_ty {
        UnresolvedType::TypeRef(return_ty_typeref) => {
            if return_ty_typeref.generic_args.is_none() {
                if let Some(generic_arg) = callee_generic_args.iter().find(|x| {
                    if x.value.name == return_ty_typeref.name {
                        true
                    } else {
                        false
                    }
                }) {
                    types
                        .borrow_mut()
                        .add(generic_arg.value.name.clone(), current_annotation.clone());
                    return Ok(true);
                } else {
                    return Ok(false);
                }
            }
            if let Some(generic_args) = &return_ty_typeref.generic_args {
                match current_annotation {
                    ResolvedType::Struct(resolved_struct) => {
                        if resolved_struct.non_generic_name == return_ty_typeref.name {
                            let mut generic_arg_inferred = false;
                            for (i, resolved_generic_ty) in
                                resolved_struct.generic_args.iter().enumerate()
                            {
                                if infer_generic_args_recursively(
                                    errors,
                                    types.clone(),
                                    scopes.clone(),
                                    type_defs,
                                    function_by_name,
                                    resolved_functions,
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
                    errors,
                    types.clone(),
                    scopes.clone(),
                    type_defs,
                    function_by_name,
                    resolved_functions,
                    callee,
                    &return_ty_pointer_ty,
                    inner,
                )? {
                    resolve_function(
                        errors,
                        types.clone(),
                        scopes.clone(),
                        type_defs,
                        function_by_name,
                        resolved_functions,
                        callee,
                    )?;
                    return Ok(true);
                }
            } else {
                return Ok(false);
            }
        }
    }

    Ok(false)
}

pub fn resolve_infer_generic_from_arguments(
    errors: &mut Vec<CompileError>,
    types: Rc<RefCell<TypeScopes>>,
    scopes: Rc<RefCell<VariableScopes>>,
    type_defs: &HashMap<String, ast::TypeDef>,
    function_by_name: &HashMap<String, ast::Function>,
    resolved_functions: &mut HashMap<String, resolved_ast::Function>,
    call_expr: &ast::CallExpr,
    callee: &ast::Function,
    resolved_args: &[ResolvedExpression], // 推論に成功した場合のみtrueを返す
) -> Result<bool, FaitalError> {
    if call_expr.generic_args.is_some() {
        return Ok(false);
    }
    if callee.decl.generic_args.is_none() {
        return Ok(false);
    }
    if callee
        .decl
        .args
        .iter()
        .find(|x| match x {
            ast::Argument::VarArgs => true,
            ast::Argument::Normal(_, _) => false,
        })
        .is_some()
    {
        return Ok(false);
    }
    let mut inferred = false;
    in_global_scope!(scopes, {
        in_global_scope!(types, {
            for (arg_idx, arg) in call_expr.args.iter().enumerate() {
                let callee_arg = &callee.decl.args[arg_idx];
                // 引数にジェネリクスを含まない場合は推論しない
                // 含む場合、unknownになるはずなので推論する
                match callee_arg {
                    ast::Argument::VarArgs => unreachable!(),
                    ast::Argument::Normal(callee_ty, _name) => {
                        if infer_generic_args_recursively(
                            errors,
                            types.clone(),
                            scopes.clone(),
                            type_defs,
                            function_by_name,
                            resolved_functions,
                            callee,
                            callee_ty,
                            &resolved_args[arg_idx].ty.clone(),
                        )? {
                            inferred = true;
                        }
                    }
                }
            }
            resolve_function(
                errors,
                types.clone(),
                scopes.clone(),
                type_defs,
                function_by_name,
                resolved_functions,
                callee,
            )
        });
    });
    Ok(inferred)
}

pub fn resolve_infer_generic_from_annotation(
    errors: &mut Vec<CompileError>,
    types: Rc<RefCell<TypeScopes>>,
    scopes: Rc<RefCell<VariableScopes>>,
    type_defs: &HashMap<String, ast::TypeDef>,
    function_by_name: &HashMap<String, ast::Function>,
    resolved_functions: &mut HashMap<String, resolved_ast::Function>,
    call_expr: &ast::CallExpr,
    callee: &ast::Function,
    annotation: Option<&ResolvedType>,
    // 推論に成功した場合のみtrueを返す
) -> Result<bool, FaitalError> {
    if call_expr.generic_args.is_some() {
        return Ok(false);
    }
    if callee.decl.generic_args.is_none() {
        return Ok(false);
    }
    if let Some(annotation) = &annotation {
        return in_global_scope!(scopes, {
            in_global_scope!(types, {
                let inferred = infer_generic_args_recursively(
                    errors,
                    types.clone(),
                    scopes.clone(),
                    type_defs,
                    function_by_name,
                    resolved_functions,
                    callee,
                    &callee.decl.return_type.value,
                    *annotation,
                )?;
                resolve_function(
                    errors,
                    types.clone(),
                    scopes.clone(),
                    type_defs,
                    function_by_name,
                    resolved_functions,
                    callee,
                )?;
                Ok(inferred)
            })
        });
    }
    Ok(false)
}

pub fn resolve_non_generic_function(
    errors: &mut Vec<CompileError>,
    types: Rc<RefCell<TypeScopes>>,
    scopes: Rc<RefCell<VariableScopes>>,
    type_defs: &HashMap<String, ast::TypeDef>,
    function_by_name: &HashMap<String, ast::Function>,
    resolved_functions: &mut HashMap<String, resolved_ast::Function>,
    callee: &ast::Function,
    // 推論に成功した場合のみtrueを返す
) -> Result<bool, FaitalError> {
    if callee.decl.generic_args.is_some() {
        return Ok(false);
    }
    in_global_scope!(scopes, {
        in_global_scope!(types, {
            resolve_function(
                errors,
                types.clone(),
                scopes.clone(),
                type_defs,
                function_by_name,
                resolved_functions,
                callee,
            )?;
        });
    });
    Ok(true)
}

pub fn resolve_call_expr(
    errors: &mut Vec<CompileError>,
    types: Rc<RefCell<TypeScopes>>,
    scopes: Rc<RefCell<VariableScopes>>,
    type_defs: &HashMap<String, ast::TypeDef>,
    function_by_name: &HashMap<String, ast::Function>,
    resolved_functions: &mut HashMap<String, resolved_ast::Function>,
    call_expr: &ast::CallExpr,
    annotation: Option<ResolvedType>,
) -> Result<ResolvedExpression, FaitalError> {
    let callee = function_by_name
        .get(&call_expr.name)
        .ok_or_else(|| FaitalError(format!("No function named {}", call_expr.name)))?;

    let mut inferred = true;
    let mut dummy_vec = Vec::new();
    if !resolve_call_with_generic_args(
        &mut dummy_vec,
        types.clone(),
        scopes.clone(),
        type_defs,
        function_by_name,
        resolved_functions,
        call_expr,
        callee,
    )? && !resolve_non_generic_function(
        &mut dummy_vec,
        types.clone(),
        scopes.clone(),
        type_defs,
        function_by_name,
        resolved_functions,
        callee,
    )? && !resolve_infer_generic_from_annotation(
        &mut dummy_vec,
        types.clone(),
        scopes.clone(),
        type_defs,
        function_by_name,
        resolved_functions,
        call_expr,
        callee,
        annotation.as_ref(),
    )? {
        inferred = false;
    }

    let mut resolved_args = Vec::new();
    let has_var_args = callee.decl.args.last() == Some(&ast::Argument::VarArgs);
    for (i, arg) in call_expr.args.iter().enumerate() {
        let calee_arg = if has_var_args && i >= callee.decl.args.len() {
            &callee.decl.args[callee.decl.args.len() - 1]
        } else {
            &callee.decl.args[i]
        };
        match calee_arg {
            ast::Argument::VarArgs => {
                resolved_args.push(resolve_expression(
                    errors,
                    types.clone(),
                    scopes.clone(),
                    type_defs,
                    function_by_name,
                    resolved_functions,
                    arg,
                    None,
                )?);
            }
            ast::Argument::Normal(ty, _name) => {
                let resolved_ty =
                    resolve_type(errors, types.borrow_mut().deref_mut(), type_defs, ty)?;
                let resolved_arg = resolve_expression(
                    errors,
                    types.clone(),
                    scopes.clone(),
                    type_defs,
                    function_by_name,
                    resolved_functions,
                    arg,
                    Some(resolved_ty.clone()),
                )?;
                if !resolved_arg.ty.can_insert(&resolved_ty) {
                    dbg!(errors.push(CompileError::from_error_kind(
                        CompileErrorKind::TypeMismatch {
                            expected: resolved_ty.clone(),
                            actual: resolved_arg.ty.clone(),
                        },
                    )));
                }

                resolved_args.push(resolved_arg);
            }
        }
    }

    if !inferred {
        if !resolve_infer_generic_from_arguments(
            &mut dummy_vec,
            types.clone(),
            scopes.clone(),
            type_defs,
            function_by_name,
            resolved_functions,
            call_expr,
            callee,
            &resolved_args,
        )? {
            dbg!(errors.push(CompileError::from_error_kind(
                CompileErrorKind::CannotInferGenericArgs {
                    name: call_expr.name.to_owned(),
                    message: "".to_string()
                },
            )));
            return Ok(ResolvedExpression {
                ty: resolve_type(
                    &mut Vec::new(),
                    types.borrow_mut().deref_mut(),
                    type_defs,
                    &callee.decl.return_type.value,
                )?,
                kind: ExpressionKind::Unknown,
            });
        }
    }

    let mut resolved_return_ty = resolve_type(
        errors,
        types.borrow_mut().deref_mut(),
        type_defs,
        &callee.decl.return_type.value,
    )?;
    // void* はアノテーションがあればその型として扱う
    if let Some(annotation) = &annotation {
        if let ResolvedType::Ptr(inner) = &resolved_return_ty {
            if let ResolvedType::Void = **inner {
                resolved_return_ty = annotation.clone();
            }
        }
    };

    // varargsはintrinsicでしか定義しないので、引数の最後に来ないケースは想定しない。
    if call_expr.args.len() != callee.decl.args.len() && !has_var_args {
        dbg!(errors.push(CompileError::from_error_kind(
            CompileErrorKind::MismatchFunctionArgCount {
                name: call_expr.name.to_owned(),
                expected: callee.decl.args.len(),
                actual: call_expr.args.len(),
            },
        )));
        return Ok(ResolvedExpression {
            ty: resolved_return_ty,
            kind: ExpressionKind::Unknown,
        });
    }

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
