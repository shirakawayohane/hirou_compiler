use self::error::CompileErrorKind;

use super::*;

pub(crate) fn try_implements_interface(
    context: &ResolverContext,
    ty: &ResolvedType,
    interface: &Interface,
) -> Result<(), CompileError> {
    let impls_by_name = context.impls_by_name.borrow();
    todo!();
}

pub(crate) fn infer_generic_argument_from_args(
    context: &ResolverContext,
    declared_generic_args: &[Located<ast::GenericArgument>],
    resolved_args: &[ResolvedType],
    generic_annotations: &[ResolvedType],
) -> Result<Vec<ResolvedType>, CompileError> {
    todo!();
}

pub(crate) fn check_generic_bounds(
    context: &ResolverContext,
    declared_generic_args: &[Located<ast::GenericArgument>],
    resolved_arg_types: &[ResolvedType],
    actual_generic_args: &[ResolvedType],
) -> Result<(), CompileError> {
    let inferred_generic_args = infer_generic_argument_from_args(
        context,
        declared_generic_args,
        resolved_arg_types,
        actual_generic_args,
    )?;
    for (i, arg) in declared_generic_args.iter().enumerate() {
        for restriction in &arg.restrictions {
            match restriction {
                ast::Restriction::Interface(name) => {
                    let interface_by_name = context.interface_by_name.borrow();
                    let interface = interface_by_name.get(name).ok_or_else(|| {
                        CompileError::new(
                            arg.range,
                            CompileErrorKind::InterfaceNotFound { name: name.clone() },
                        )
                    })?;
                    let resolved_arg_ty = &resolved_arg_types[i];
                    try_implements_interface(context, resolved_arg_ty, &interface)?;
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn resolve_generic_arguments(
    context: &ResolverContext,
    generic_args: &[ast::GenericArgument],
    resolved_arg_types: &[ResolvedType],
    generic_annotations: &[ResolvedType],
    type_annotation: &ResolvedType,
) -> Result<Vec<ResolvedType>, CompileError> {
    todo!();
}
