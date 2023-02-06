use crate::ast::UnresolvedType;

use super::{
    error::{CompileError, CompileErrorKind},
    *,
};

impl LLVMCodegenerator<'_> {
    pub(crate) fn resolve_type(&self, ty: &UnresolvedType) -> Result<&ResolvedType, CompileError> {
        if let Some(resolved) = self.context.borrow().resolve_type(ty) {
            Ok(resolved)
        } else {
            Err(CompileError::from_error_kind(
                CompileErrorKind::TypeNotFound {
                    name: format!("{}", ty),
                },
            ))
        }
    }
}
