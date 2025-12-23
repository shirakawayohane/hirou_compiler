use self::error::CompileErrorKind;

use super::*;

pub(crate) fn try_implements_interface(
    context: &ResolverContext,
    ty: &ResolvedType,
    interface: &Interface,
) -> Result<(), CompileError> {
    let impls_by_name = context.impls_by_name.borrow();

    // Check if there's an implementation of this interface for the given type
    if let Some(impls) = impls_by_name.get(&interface.name) {
        for implementation in impls {
            if let Ok(impl_target_ty) =
                crate::resolver::ty::resolve_type(context, &implementation.decl.target_ty)
            {
                if impl_target_ty == *ty {
                    return Ok(());
                }
            }
        }
    }

    Err(CompileError::new(
        crate::ast::Range::default(),
        CompileErrorKind::InterfaceNotImplemented {
            name: interface.name.clone(),
            ty: ty.clone(),
        },
    ))
}

pub(crate) fn infer_generic_argument_from_args(
    _context: &ResolverContext,
    declared_generic_args: &[Located<ast::GenericArgument>],
    _resolved_args: &[ResolvedType],
    generic_annotations: &[ResolvedType],
) -> Result<Vec<ResolvedType>, CompileError> {
    // If no generic arguments are declared, return empty vector
    if declared_generic_args.is_empty() {
        return Ok(vec![]);
    }
    // For now, just return the annotations if they match the count
    if generic_annotations.len() == declared_generic_args.len() {
        return Ok(generic_annotations.to_vec());
    }
    // TODO: Implement actual type inference from arguments
    Ok(vec![])
}

pub(crate) fn check_generic_bounds(
    context: &ResolverContext,
    declared_generic_args: &[Located<ast::GenericArgument>],
    resolved_arg_types: &[ResolvedType],
    actual_generic_args: &[ResolvedType],
) -> Result<(), CompileError> {
    // Infer or use explicit generic arguments
    let generic_args_to_check = if actual_generic_args.len() == declared_generic_args.len() {
        // Use explicit generic arguments if provided
        actual_generic_args.to_vec()
    } else {
        // Try to infer generic arguments from function arguments
        infer_generic_argument_from_args(
            context,
            declared_generic_args,
            resolved_arg_types,
            actual_generic_args,
        )?
    };

    // Check interface bounds for each generic argument
    for (i, arg) in declared_generic_args.iter().enumerate() {
        // Get the concrete type for this generic argument
        let concrete_ty = generic_args_to_check
            .get(i)
            .cloned()
            .unwrap_or(ResolvedType::Unknown);

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
                    try_implements_interface(context, &concrete_ty, &interface)?;
                }
            }
        }
    }
    Ok(())
}

// NOTE: This function is currently unused - generic argument resolution is handled
// at call sites in resolve_function_call_expr. Preserved for potential future use.
#[allow(dead_code)]
pub(crate) fn resolve_generic_arguments(
    _context: &ResolverContext,
    generic_args: &[ast::GenericArgument],
    _resolved_arg_types: &[ResolvedType],
    generic_annotations: &[ResolvedType],
    _type_annotation: &ResolvedType,
) -> Result<Vec<ResolvedType>, CompileError> {
    // If explicit generic annotations are provided and match the count, use them
    if generic_annotations.len() == generic_args.len() {
        return Ok(generic_annotations.to_vec());
    }
    // TODO: Implement type inference from arguments and return type annotation
    // For now, return empty if annotations don't match
    Ok(vec![])
}
