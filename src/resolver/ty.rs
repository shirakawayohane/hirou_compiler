use crate::{
    ast::{self, UnresolvedType},
    resolved_ast::ResolvedType,
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
                        for (_, field_ty) in &struct_def.fields {
                            resolved_fields.push(resolve_type(
                                errors,
                                type_scopes,
                                type_defs,
                                field_ty,
                            )?);
                        }
                        return Ok(ResolvedType::Struct(resolved_fields));
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
