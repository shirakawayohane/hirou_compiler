use crate::{in_new_scope, resolved_ast::ResolvedStructType};

#[cfg(test)]
use resolved_ast::{I32_TYPE_NAME, USIZE_TYPE_NAME};

use super::*;

pub(super) fn resolve_type(
    context: &ResolverContext,
    loc_ty: &Located<ast::UnresolvedType>,
) -> Result<ResolvedType> {
    match &loc_ty.value {
        UnresolvedType::TypeRef(typ_ref) => {
            if let Some(type_def) = context.type_defs.borrow().get(&typ_ref.name) {
                match &type_def.kind {
                    TypeDefKind::StructLike(struct_def) => {
                        let mut resolved_generic_args = Vec::new();
                        if let Some(generic_args) = &typ_ref.generic_args {
                            if let Some(generic_args_in_def) = &struct_def.generic_args {
                                if generic_args.len() != generic_args_in_def.len() {
                                    context.errors.borrow_mut().push(CompileError::new(
                                        loc_ty.range,
                                        error::CompileErrorKind::MismatchGenericArgCount {
                                            name: typ_ref.name.clone(),
                                            expected: generic_args_in_def.len(),
                                            actual: generic_args.len(),
                                        },
                                    ));
                                    Ok(ResolvedType::Unknown)
                                } else {
                                    in_new_scope!(context.types, {
                                        for (i, generic_arg) in generic_args.iter().enumerate() {
                                            let resolved_generic_arg =
                                                resolve_type(context, generic_arg)?;
                                            resolved_generic_args
                                                .push(resolved_generic_arg.clone());
                                            context.types.borrow_mut().add(
                                                generic_args_in_def[i].name.clone(),
                                                resolved_generic_arg.clone(),
                                            )
                                        }
                                        Ok(ResolvedType::StructLike(ResolvedStructType {
                                            name: get_resolved_struct_name(
                                                &type_def.name,
                                                Some(&resolved_generic_args),
                                            ),
                                            fields: struct_def
                                                .fields
                                                .iter()
                                                .map(|(name, unresolved_ty)| {
                                                    match resolve_type(context, unresolved_ty) {
                                                        Ok(resolved_ty) => {
                                                            Ok((name.clone(), resolved_ty.clone()))
                                                        }
                                                        Err(err) => Err(err),
                                                    }
                                                })
                                                .collect::<Result<Vec<_>>>()?,
                                            generic_args: if struct_def.generic_args.is_some() {
                                                Some(resolved_generic_args)
                                            } else {
                                                None
                                            },
                                            non_generic_name: type_def.name.clone(),
                                        }))
                                    })
                                }
                            } else {
                                context.errors.borrow_mut().push(CompileError::new(
                                    loc_ty.range,
                                    error::CompileErrorKind::NoGenericArgs {
                                        name: typ_ref.name.clone(),
                                    },
                                ));
                                Ok(ResolvedType::Unknown)
                            }
                        } else if struct_def.generic_args.is_some() {
                            context.errors.borrow_mut().push(CompileError::new(
                                loc_ty.range,
                                error::CompileErrorKind::NoGenericArgs {
                                    name: typ_ref.name.clone(),
                                },
                            ));
                            Ok(ResolvedType::Unknown)
                        } else {
                            Ok(ResolvedType::StructLike(ResolvedStructType {
                                name: get_resolved_struct_name(&type_def.name, None),
                                fields: struct_def
                                    .fields
                                    .iter()
                                    .map(|(name, unresolved_ty)| {
                                        match resolve_type(context, unresolved_ty) {
                                            Ok(resolved_ty) => {
                                                Ok((name.clone(), resolved_ty.clone()))
                                            }
                                            Err(err) => Err(err),
                                        }
                                    })
                                    .collect::<Result<Vec<_>>>()?,
                                generic_args: None,
                                non_generic_name: type_def.name.clone(),
                            }))
                        }
                    }
                }
            } else {
                let resolved_type = context
                    .types
                    .borrow()
                    .get(&typ_ref.name)
                    .cloned()
                    .unwrap_or_else(|| {
                        context.errors.borrow_mut().push(CompileError::new(
                            loc_ty.range,
                            error::CompileErrorKind::TypeNotFound {
                                name: typ_ref.name.clone(),
                            },
                        ));
                        ResolvedType::Unknown
                    });
                Ok(resolved_type.clone())
            }
        }
        UnresolvedType::Ptr(inner_type) => {
            let inner_type: ResolvedType = resolve_type(context, inner_type)?;
            Ok(ResolvedType::Ptr(Box::new(inner_type)))
        }
    }
}

pub(crate) fn get_resolved_struct_name(
    name: &str,
    generic_args: Option<&[ResolvedType]>,
) -> String {
    if let Some(generic_args) = generic_args {
        format!(
            "{}<{}>",
            name,
            generic_args
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    } else {
        name.to_string()
    }
}

#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::common::StructKind;

    #[test]
    fn test_resolve_type() {
        let context = ResolverContext::new(PointerSizedIntWidth::SixtyFour);
        context.type_defs.borrow_mut().insert(
            "Vec".to_string(),
            TypeDef {
                name: "Vec".to_string(),
                kind: TypeDefKind::StructLike(StructLikeTypeDef {
                    struct_kind: StructKind::Struct,
                    fields: vec![
                        (
                            "ptr".to_string(),
                            Located::default_from(UnresolvedType::Ptr(Box::new(
                                Located::default_from(UnresolvedType::TypeRef(TypeRef {
                                    name: "T".to_string(),
                                    generic_args: None,
                                })),
                            ))),
                        ),
                        (
                            "len".to_string(),
                            Located::default_from(UnresolvedType::TypeRef(TypeRef {
                                name: "usize".to_string(),
                                generic_args: None,
                            })),
                        ),
                    ],
                    generic_args: Some(vec![Located {
                        range: Range::default(),
                        value: GenericArgument {
                            name: "T".to_string(),
                        },
                    }]),
                }),
            },
        );
        context.types.borrow_mut().push(
            [
                (I32_TYPE_NAME.to_string(), ResolvedType::I32),
                (USIZE_TYPE_NAME.to_string(), ResolvedType::USize),
            ]
            .into_iter()
            .collect::<HashMap<_, _>>(),
        );
        let resolved_ty = resolve_type(
            &context,
            &Located::default_from(UnresolvedType::TypeRef(TypeRef {
                name: "Vec".to_string(),
                generic_args: Some(vec![Located::default_from(UnresolvedType::TypeRef(
                    TypeRef {
                        name: I32_TYPE_NAME.to_string(),
                        generic_args: None,
                    },
                ))]),
            })),
        )
        .unwrap();
        assert_eq!(context.errors.borrow().len(), 0);
        assert_eq!(
            resolved_ty,
            ResolvedType::StructLike(ResolvedStructType {
                name: "Vec<i32>".to_string(),
                non_generic_name: "Vec".to_string(),
                fields: vec![
                    (
                        "ptr".to_string(),
                        ResolvedType::Ptr(Box::new(ResolvedType::I32))
                    ),
                    ("len".to_string(), ResolvedType::USize),
                ],
                generic_args: Some(vec![ResolvedType::I32]),
            })
        )
    }
}
