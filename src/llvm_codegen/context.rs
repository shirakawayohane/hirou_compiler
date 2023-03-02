use std::{collections::HashMap};

use inkwell::values::{FunctionValue, PointerValue};

use crate::ast::{Function, ResolvedType, UnresolvedType};



#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScopeKind {
    Global,
    Function,
}

#[derive(Debug, Clone)]
pub struct Scope<'a> {
    pub kind: ScopeKind,
    pub variables: HashMap<String, (UnresolvedType, PointerValue<'a>)>,
}

impl Scope<'_> {
    pub fn new(kind: ScopeKind) -> Self {
        Scope {
            kind,
            variables: HashMap::new(),
        }
    }
}

pub struct RegisteredFunction<'a> {
    pub return_type: UnresolvedType,
    pub arg_types: Vec<UnresolvedType>,
    pub function_value: FunctionValue<'a>,
}

pub(super) struct GenericFunctionRegistration {
    pub scope_depth: u16,
    pub function: Function,
}

pub(super) struct FindFunctionResult {
    pub impl_type: Option<UnresolvedType>,
}

pub(super) struct Context<'a> {
    pub variables: Vec<Scope<'a>>,
    pub functions: Vec<
        HashMap<
            String,
            // TODO: make this tuple struct
            RegisteredFunction<'a>,
        >,
    >,
    pub generic_functions: Vec<HashMap<String, GenericFunctionRegistration>>,
    pub types: Vec<HashMap<UnresolvedType, ResolvedType>>,
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Context {
            variables: Vec::new(),
            functions: Vec::new(),
            types: Vec::new(),
            generic_functions: Vec::new(),
        }
    }
}
