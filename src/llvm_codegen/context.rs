use std::collections::HashMap;

use inkwell::values::{FunctionValue, PointerValue};

use crate::ast::{Function, ResolvedType, UnresolvedType};

type MangledName = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScopeKind {
    Global,
    Function,
}

#[derive(Debug)]
pub enum Registration<'a> {
    Variable {
        ns: String,
        ty: UnresolvedType,
        value: PointerValue<'a>,
    },
    Function {
        ns: String,
        function: Function,
    },
    Type {
        ns: String,
        unresolved_ty: UnresolvedType,
    },
}

#[derive(Debug)]
pub struct Scope<'a> {
    pub kind: ScopeKind,
    registrations: HashMap<String, Registration<'a>>,
}

impl Scope<'_> {
    pub fn new(kind: ScopeKind) -> Self {
        Scope {
            kind,
            registrations: HashMap::new(),
        }
    }
    pub fn find(&self, key: &str) -> Option<&Registration> {
        self.registrations.get(key)
    }
    pub fn set(&mut self, key: String, reg: Registration) {
        self.registrations.insert(key, reg);
    }
}

pub struct FunctionImpl<'a> {
    pub return_type: ResolvedType,
    pub arg_types: Vec<ResolvedType>,
    pub function_value: FunctionValue<'a>,
}

pub(super) struct FindFunctionResult {
    pub impl_type: Option<UnresolvedType>,
}

pub(super) struct Context<'a> {
    pub scopes: Vec<Scope<'a>>,
    pub function_impls: HashMap<MangledName, FunctionValue<'a>>,
    pub resolved_types: HashMap<MangledName, ResolvedType>,
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Context {
            scopes: Vec::new(),
            resolved_types: HashMap::new(),
            function_impls: HashMap::new(),
        }
    }
}

impl Context<'_> {
    pub fn get_current_ns(&self) -> String {
        todo!();
    }
}

pub fn mangled_type_name(name: &str, types: &[ResolvedType]) -> String {
    let mut mangled = name.to_string();
    mangled.push_str("_");
    for ty in types {
        mangled.push_str(&ty.to_string());
        mangled.push_str("_");
    }
    mangled
}
