use crate::{
    ast::{self, UnresolvedType},
    resolved_ast::{ResolvedStructType, ResolvedType},
};

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
                        let mut resolved_fields = Vec::new();
                        for (field_name, field_ty) in &struct_def.fields {
                            resolved_fields.push((
                                field_name.to_string(),
                                resolve_type(errors, type_scopes, type_defs, field_ty)?,
                            ));
                        }

                        let mut resolved_generic_args = Vec::new();
                        if let Some(generic_args) = &typ_ref.generic_args {
                            if let Some(generic_args_in_def) = &struct_def.generic_args {
                                if generic_args.len() != generic_args_in_def.len() {
                                    errors.push(CompileError::from_error_kind(
                                        error::CompileErrorKind::MismatchGenericArgCount {
                                            name: typ_ref.name.clone(),
                                            expected: generic_args_in_def.len(),
                                            actual: generic_args.len(),
                                        },
                                    ));
                                    return Ok(ResolvedType::Unknown);
                                } else {
                                    for generic_arg in generic_args {
                                        let resolved_generic_arg = resolve_type(
                                            errors,
                                            type_scopes,
                                            type_defs,
                                            generic_arg,
                                        )?;
                                        resolved_generic_args.push(resolved_generic_arg);
                                    }
                                }
                            } else {
                                errors.push(CompileError::from_error_kind(
                                    error::CompileErrorKind::NoGenericArgs {
                                        name: typ_ref.name.clone(),
                                    },
                                ));
                                return Ok(ResolvedType::Unknown);
                            }
                        } else {
                            if struct_def.generic_args.is_some() {
                                errors.push(CompileError::from_error_kind(
                                    error::CompileErrorKind::NoGenericArgs {
                                        name: typ_ref.name.clone(),
                                    },
                                ));
                                return Ok(ResolvedType::Unknown);
                            }
                        }

                        return Ok(ResolvedType::Struct(ResolvedStructType {
                            name: get_resolved_struct_name(
                                &type_def.name,
                                Some(&resolved_generic_args),
                            ),
                            fields: resolved_fields,
                        }));
                    }
                }
            }
            let resolved_type = type_scopes.get(&typ_ref.name).unwrap_or_else(|| {
                dbg!(type_scopes.clone());
                errors.push(CompileError::from_error_kind(
                    error::CompileErrorKind::TypeNotFound {
                        name: typ_ref.name.clone(),
                    },
                ));
                &&ResolvedType::Unknown
            });
            Ok(resolved_type.clone())
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
