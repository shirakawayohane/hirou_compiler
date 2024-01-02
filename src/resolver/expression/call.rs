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
    // 条件に合致した場合はtrueを返す。
) -> Result<bool, FaitalError> {
    if let Some(generic_args) = &callee.decl.generic_args {
        // スコープに解決したジェネリックの型を追加する
        if let Some(actual_generic_args) = &call_expr.generic_args {
            if generic_args.len() != actual_generic_args.len() {
                errors.push(CompileError::from_error_kind(
                    CompileErrorKind::MismatchGenericArgCount {
                        name: call_expr.name.to_owned(),
                        expected: generic_args.len(),
                        actual: actual_generic_args.len(),
                    },
                ));
                return Ok(true);
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
) -> Result<bool, FaitalError> {
    if call_expr.generic_args.is_some() {
        return Ok(false);
    }
    if callee.decl.generic_args.is_none() {
        return Ok(false);
    }
    if let Some(annotation) = &annotation {
        if let ast::UnresolvedType::TypeRef(TypeRef { name, generic_args }) =
            &callee.decl.return_type.value
        {
            if let ResolvedType::Struct(resolved_struct) = &annotation {
                dbg!(name, &resolved_struct.non_generic_name);
                if &resolved_struct.non_generic_name == name {
                    in_global_scope!(scopes, {
                        in_global_scope!(types, {
                            // 正常系
                            let resolved_generic_args =
                                resolved_struct.generic_args.as_ref().unwrap();
                            for i in 0..resolved_generic_args.len() {
                                let actual_arg = resolved_generic_args[i].clone();
                                dbg!(name.clone(), actual_arg.clone());
                                if let UnresolvedType::TypeRef(typeref) =
                                    &generic_args.as_ref().unwrap()[i]
                                {
                                    types.borrow_mut().add(typeref.name.clone(), actual_arg);
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
                            )?
                        });
                    });
                } else {
                    errors.push(CompileError::from_error_kind(
                        CompileErrorKind::CannotInferGenericArgs {
                            name: callee.decl.name.clone(),
                            message: format!(
                                "Expected type {}<_> but found {}<_>",
                                resolved_struct.non_generic_name, name
                            ),
                        },
                    ));
                }
                return Ok(true);
            }
        }
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
            )?
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

    if !resolve_call_with_generic_args(
        errors,
        types.clone(),
        scopes.clone(),
        type_defs,
        function_by_name,
        resolved_functions,
        call_expr,
        callee,
    )? && !resolve_non_generic_function(
        errors,
        types.clone(),
        scopes.clone(),
        type_defs,
        function_by_name,
        resolved_functions,
        callee,
    )? && !resolve_infer_generic_from_annotation(
        errors,
        types.clone(),
        scopes.clone(),
        type_defs,
        function_by_name,
        resolved_functions,
        call_expr,
        callee,
        annotation.as_ref(),
    )? {
        // 解析できるケースに当てはまらない場合はUnknownを返す
        return Ok(ResolvedExpression {
            kind: ExpressionKind::CallExpr(resolved_ast::CallExpr {
                callee: call_expr.name.clone(),
                args: Vec::new(),
            }),
            ty: ResolvedType::Unknown,
        });
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
    let mut resolved_args = Vec::new();
    for (i, arg) in call_expr.args.iter().enumerate() {
        let calee_arg = &callee.decl.args[i];
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

                resolved_args.push(resolve_expression(
                    errors,
                    types.clone(),
                    scopes.clone(),
                    type_defs,
                    function_by_name,
                    resolved_functions,
                    arg,
                    Some(resolved_ty),
                )?);
            }
        }
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
