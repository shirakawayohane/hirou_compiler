use std::collections::HashMap;

use inkwell::values::{PointerValue, FunctionValue};

use crate::ast::Type;

pub(super) struct Context<'a> {
    pub scopes: Vec<HashMap<String, (Type, PointerValue<'a>)>>,
    pub functions: Vec<HashMap<String, (Type, Vec<Type>, FunctionValue<'a>)>>
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Context { scopes: Vec::new(), functions: Vec::new() }
    }
    pub fn find_variable(&self, name: &str) -> Option<&(Type, PointerValue<'a>)> {
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v);
            }
        }
        None
    }
    pub fn find_function(&self, name: &str) -> Option<&(Type, Vec<Type>, FunctionValue<'a>)> {
        for function in self.functions.iter().rev() {
            if let Some(v) = function.get(name) {
                return Some(v);
            }
        }
        None
    }
    pub fn set_variable(&mut self, name: String, ty: Type, pointer: PointerValue<'a>) {
        self.scopes.last_mut().unwrap().insert(name, (ty, pointer));
    }
    pub fn set_function(&mut self, name: String, return_type: Type, argument_types: Vec<Type>, function: FunctionValue<'a>) {
        self.functions.last_mut().unwrap().insert(name, (return_type, argument_types, function));
    }
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }
    pub fn push_function_scope(&mut self) {
        self.functions.push(HashMap::new());
    }
    pub fn pop_function_scope(&mut self) {
        self.functions.pop();
    }
}
