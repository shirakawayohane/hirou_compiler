use std::collections::HashMap;

use inkwell::values::PointerValue;

use crate::ast::Type;

use super::value::Value;

pub(super) struct Context<'a> {
    pub scopes: Vec<HashMap<String, (Type, PointerValue<'a>)>>,
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Context { scopes: Vec::new() }
    }
    pub fn find_variable(&'a self, name: &str) -> Option<&(Type, PointerValue)> {
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v);
            }
        }
        None
    }
    pub fn set_variable(&mut self, name: String, ty: Type, pointer: PointerValue<'a>) {
        self.scopes.last_mut().unwrap().insert(name, (ty, pointer));
    }
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }
}
