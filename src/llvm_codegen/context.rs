use std::collections::HashMap;

use inkwell::values::{FunctionValue, PointerValue};

use crate::ast::{ResolvedType, UnresolvedType};

use super::error::{CompileError, CompileErrorKind};

pub(super) struct Context<'a> {
    pub variables: Vec<HashMap<String, (UnresolvedType, PointerValue<'a>)>>,
    pub functions: Vec<
        HashMap<
            String,
            // TODO: make this tuple struct
            (UnresolvedType, Vec<UnresolvedType>, FunctionValue<'a>),
        >,
    >,
    pub types: Vec<HashMap<UnresolvedType, ResolvedType>>,
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Context {
            variables: Vec::new(),
            functions: Vec::new(),
            types: Vec::new(),
        }
    }
    pub fn find_variable(
        &'a self,
        name: &str,
    ) -> Result<(&UnresolvedType, PointerValue), CompileError> {
        for scope in self.variables.iter().rev() {
            if let Some((ty, ptr_value)) = scope.get(name) {
                return Ok((ty, *ptr_value));
            }
        }
        Err(CompileError::from_error_kind(
            CompileErrorKind::CallNotFunctionValue {
                name: name.to_string(),
            },
        ))
    }
    pub fn resolve_type(
        &self,
        unresolved_ty: &UnresolvedType,
    ) -> Result<ResolvedType, CompileError> {
        match unresolved_ty {
            UnresolvedType::TypeRef {
                name: _,
                generic_args: _,
            } => {
                for types_in_scope in self.types.iter().rev() {
                    if let Some(v) = types_in_scope.get(unresolved_ty) {
                        return Ok(v.clone());
                    }
                }
                Err(CompileError::from_error_kind(
                    super::error::CompileErrorKind::TypeNotFound {
                        name: format!("{}", unresolved_ty),
                    },
                ))
            }
            UnresolvedType::Array(inner_ty) => {
                let resolved_inner_ty = self.resolve_type(inner_ty)?;
                Ok(ResolvedType::Ptr(
                    Box::new(resolved_inner_ty.clone()).clone(),
                ))
            }
        }
    }
    pub fn find_function(
        &self,
        name: &str,
    ) -> Option<&(UnresolvedType, Vec<UnresolvedType>, FunctionValue<'a>)> {
        for function in self.functions.iter().rev() {
            if let Some(v) = function.get(name) {
                return Some(v);
            }
        }
        None
    }
    pub fn set_variable(&mut self, name: String, ty: UnresolvedType, pointer: PointerValue<'a>) {
        self.variables
            .last_mut()
            .unwrap()
            .insert(name, (ty, pointer));
    }
    pub fn set_type(&mut self, unresolved: UnresolvedType, resolved: ResolvedType) {
        self.types.last_mut().unwrap().insert(unresolved, resolved);
    }
    pub fn set_function(
        &mut self,
        name: String,
        return_type: UnresolvedType,
        argument_types: Vec<UnresolvedType>,
        function: FunctionValue<'a>,
    ) {
        self.functions
            .last_mut()
            .unwrap()
            .insert(name, (return_type, argument_types, function));
    }
    pub fn push_variable_scope(&mut self) {
        self.variables.push(HashMap::new());
    }
    pub fn pop_variable_scope(&mut self) {
        self.variables.pop();
    }
    pub fn push_function_scope(&mut self) {
        self.functions.push(HashMap::new());
    }
    pub fn pop_function_scope(&mut self) {
        self.functions.pop();
    }
    pub fn push_type_scope(&mut self) {
        self.types.push(HashMap::new());
    }
    pub fn pop_type_scope(&mut self) {
        self.types.pop();
    }
}
