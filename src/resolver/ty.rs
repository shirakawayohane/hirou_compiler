use std::{
    borrow::{Borrow, BorrowMut},
    ops::Deref,
};

use crate::{
    ast::{self, UnresolvedType},
    in_new_scope,
    resolved_ast::{ResolvedStructType, ResolvedType},
};

#[cfg(test)]
use resolved_ast::{I32_TYPE_NAME, USIZE_TYPE_NAME};

use super::*;

pub(super) fn resolve_type<'a>(
    errors: &mut Vec<CompileError>,
    type_scopes: &mut TypeScopes,
    type_defs: &HashMap<String, ast::TypeDef>,
    ty: &ast::UnresolvedType,
) -> Result<ResolvedType> {
    match ty {
        UnresolvedType::TypeRef(typ_ref) => {
            if let Some(type_def) = type_defs.get(&typ_ref.name) {
                match &type_def.kind {
                    TypeDefKind::Struct(struct_def) => {
                        let mut resolved_generic_args = Vec::new();
                        if let Some(generic_args) = &typ_ref.generic_args {
                            if let Some(generic_args_in_def) = &struct_def.generic_args {
                                if generic_args.len() != generic_args_in_def.len() {
                                    dbg!(errors.push(CompileError::from_error_kind(
                                        error::CompileErrorKind::MismatchGenericArgCount {
                                            name: typ_ref.name.clone(),
                                            expected: generic_args_in_def.len(),
                                            actual: generic_args.len(),
                                        },
                                    )));
                                    return Ok(ResolvedType::Unknown);
                                } else {
                                    in_new_scope!(type_scopes, {
                                        for (i, generic_arg) in generic_args.iter().enumerate() {
                                            let resolved_generic_arg = resolve_type(
                                                errors,
                                                type_scopes,
                                                type_defs,
                                                generic_arg,
                                            )?;
                                            resolved_generic_args
                                                .push(resolved_generic_arg.clone());
                                            type_scopes.add(
                                                generic_args_in_def[i].name.clone(),
                                                resolved_generic_arg.clone(),
                                            )
                                        }
                                        Ok(ResolvedType::Struct(ResolvedStructType {
                                            name: get_resolved_struct_name(
                                                &type_def.name,
                                                Some(&resolved_generic_args),
                                            ),
                                            fields: struct_def
                                                .fields
                                                .iter()
                                                .map(|(name, unresolved_ty)| {
                                                    match resolve_type(
                                                        errors,
                                                        type_scopes.borrow_mut().deref_mut(),
                                                        type_defs.borrow().deref(),
                                                        unresolved_ty,
                                                    ) {
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
                                dbg!(errors.push(CompileError::from_error_kind(
                                    error::CompileErrorKind::NoGenericArgs {
                                        name: typ_ref.name.clone(),
                                    },
                                )));
                                return Ok(ResolvedType::Unknown);
                            }
                        } else {
                            if struct_def.generic_args.is_some() {
                                dbg!(errors.push(CompileError::from_error_kind(
                                    error::CompileErrorKind::NoGenericArgs {
                                        name: typ_ref.name.clone(),
                                    },
                                )));
                                return Ok(ResolvedType::Unknown);
                            } else {
                                return Ok(ResolvedType::Struct(ResolvedStructType {
                                    name: get_resolved_struct_name(&type_def.name, None),
                                    fields: struct_def
                                        .fields
                                        .iter()
                                        .map(|(name, unresolved_ty)| {
                                            match resolve_type(
                                                errors,
                                                type_scopes.borrow_mut().deref_mut(),
                                                type_defs.borrow().deref(),
                                                unresolved_ty,
                                            ) {
                                                Ok(resolved_ty) => {
                                                    Ok((name.clone(), resolved_ty.clone()))
                                                }
                                                Err(err) => Err(err),
                                            }
                                        })
                                        .collect::<Result<Vec<_>>>()?,
                                    generic_args: None,
                                    non_generic_name: type_def.name.clone(),
                                }));
                            }
                        }
                    }
                }
            } else {
                let resolved_type = type_scopes.get(&typ_ref.name).unwrap_or_else(|| {
                    dbg!(errors.push(CompileError::from_error_kind(
                        error::CompileErrorKind::TypeNotFound {
                            name: typ_ref.name.clone(),
                        },
                    )));
                    &&ResolvedType::Unknown
                });
                Ok(resolved_type.clone())
            }
        }
        UnresolvedType::Ptr(inner_type) => {
            let inner_type: ResolvedType =
                resolve_type(errors, type_scopes, type_defs, inner_type)?;
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

#[test]
fn test_resolve_type() {
    let mut type_defs = HashMap::new();
    type_defs.insert(
        "Vec".to_string(),
        TypeDef {
            name: "Vec".to_string(),
            kind: TypeDefKind::Struct(StructTypeDef {
                fields: vec![
                    (
                        "ptr".to_string(),
                        UnresolvedType::Ptr(Box::new(UnresolvedType::TypeRef(TypeRef {
                            name: "T".to_string(),
                            generic_args: None,
                        }))),
                    ),
                    (
                        "len".to_string(),
                        UnresolvedType::TypeRef(TypeRef {
                            name: "usize".to_string(),
                            generic_args: None,
                        }),
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
    let mut errors = Vec::new();
    let mut type_scopes = TypeScopes::new();
    type_scopes.push(
        [
            (I32_TYPE_NAME.to_string(), ResolvedType::I32),
            (USIZE_TYPE_NAME.to_string(), ResolvedType::USize),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>(),
    );
    let resolved_ty = resolve_type(
        &mut errors,
        &mut type_scopes,
        &type_defs,
        &UnresolvedType::TypeRef(TypeRef {
            name: "Vec".to_string(),
            generic_args: Some(vec![UnresolvedType::TypeRef(TypeRef {
                name: I32_TYPE_NAME.to_string(),
                generic_args: None,
            })]),
        }),
    )
    .unwrap();
    assert_eq!(errors, Vec::new());
    assert_eq!(
        resolved_ty,
        ResolvedType::Struct(ResolvedStructType {
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
