use std::collections::HashMap;

use inkwell::values::{FunctionValue, PointerValue};

use crate::ast::{ResolvedType, UnresolvedType};

pub(super) struct Context<'a> {
    pub scopes: Vec<HashMap<String, (UnresolvedType<'a>, PointerValue<'a>)>>,
    pub functions: Vec<
        HashMap<
            String,
            // TODO: make this tuple struct
            (
                UnresolvedType<'a>,
                Vec<UnresolvedType<'a>>,
                FunctionValue<'a>,
            ),
        >,
    >,
    pub types: Vec<HashMap<UnresolvedType<'a>, ResolvedType>>,
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Context {
            scopes: Vec::new(),
            functions: Vec::new(),
            types: Vec::new(),
        }
    }
    pub fn find_variable(&'a self, name: &str) -> Option<(&UnresolvedType, PointerValue)> {
        for scope in self.scopes.iter().rev() {
            if let Some((ty, ptr_value)) = scope.get(name) {
                return Some((ty, *ptr_value));
            }
        }
        None
    }
    pub fn resolve_type(&self, unresolved_ty: &UnresolvedType) -> Option<&ResolvedType> {
        for types_in_scope in self.types.iter().rev() {
            if let Some(v) = types_in_scope.get(unresolved_ty) {
                return Some(v);
            }
        }
        None
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
        self.scopes.last_mut().unwrap().insert(name, (ty, pointer));
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
        self.scopes.push(HashMap::new());
    }
    pub fn pop_variable_scope(&mut self) {
        self.scopes.pop();
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
