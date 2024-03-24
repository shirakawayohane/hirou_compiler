use self::error::CompileErrorKind;

use super::*;

pub(crate) fn try_implements_interface(
    context: &ResolverContext,
    ty: &ResolvedType,
    interface: &Interface,
) -> Result<(), CompileError> {
    let impls_by_name = context.impls_by_name.borrow();
    if interface.
}

pub(crate) fn check_generic_bounds(
    context: &ResolverContext,
    generic_args: &[ast::GenericArgument],
    resolved_args: &[ResolvedType],
) -> Result<(), FaitalError> {
    for (i, arg) in generic_args.iter().enumerate() {
        for restriction in &arg.restrictions {
            match restriction {
                ast::Restriction::Interface(name) => {
                    let interface = context.interface_by_name.borrow().get(name).unwrap();
                    let resolved_arg = &resolved_args[i];
                    if !resolved_arg.implements_interface(context, interface) {
                        context.errors.borrow_mut().push(CompileError::new(
                            arg.range,
                            CompileErrorKind::InterfaceNotImplemented {
                                name: name.clone(),
                                ty: resolved_arg.clone(),
                            },
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}
