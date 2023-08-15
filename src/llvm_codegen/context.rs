use std::collections::HashMap;

use inkwell::values::{FunctionValue, PointerValue};

use crate::ast::{Function, ResolvedType, TypeRegistration, UnresolvedType, IdentifierPrefix};

use super::{
    error::{CompileError, CompileErrorKind},
    mangle_function_impl_name,
};

type MangledName = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScopeKind {
    Global,
    Function,
}

#[derive(Debug)]
pub struct VariableRegistration<'a> {
    ns: String,
    resolved_ty: ResolvedType,
    value: PointerValue<'a>,
}

#[derive(Debug)]
pub struct FunctionRegistration {
    pub ns: String,
    pub function: Function,
}

#[derive(Debug)]
pub enum Registration<'a> {
    Variable(VariableRegistration<'a>),
    Function(FunctionRegistration),
    Type(TypeRegistration),
}

#[derive(Debug)]
pub struct Scope<'a> {
    pub kind: ScopeKind,
    registrations: HashMap<MangledName, Registration<'a>>,
}

impl<'a> Scope<'a> {
    pub fn new(kind: ScopeKind) -> Self {
        Scope {
            kind,
            registrations: HashMap::new(),
        }
    }
    pub fn find(&self, key: &str) -> Option<&Registration> {
        self.registrations.get(key)
    }
    pub fn set(&mut self, key: String, reg: Registration<'a>) {
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

pub struct Context<'a> {
    scopes: Vec<Scope<'a>>,
    pub function_impls: HashMap<MangledName, FunctionImpl<'a>>,
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

impl<'a> Context<'a> {
    pub fn get_current_ns(&self) -> String {
        String::from("global")
    }
    pub fn push_scope(&mut self, scope: Scope<'a>) {
        self.scopes.push(scope);
    }
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }
    pub fn find_function_impl(
        &self,
        reg: &FunctionRegistration,
        arg_tys: &[ResolvedType],
    ) -> Option<&FunctionImpl> {
        let mangled_name = mangle_function_impl_name(&reg.ns, &reg.function.decl.name, &arg_tys);
        self.function_impls.get(&mangled_name)
    }

    pub fn find_function_registration(
        &self,
        prefix: Option<IdentifierPrefix>,
        name: &str,
    ) -> Result<&FunctionRegistration, CompileError> {
        if let Some(_identifier_prefix) = prefix {
            todo!();
        } else {
            for scope in self.scopes.iter().rev() {
                if let Some(reg) = scope.find(&name) {
                    match reg {
                        Registration::Function(fn_reg) => return Ok(fn_reg),
                        _ => {
                            return Err(CompileError::from_error_kind(
                                CompileErrorKind::IsNotFunction {
                                    name: name.to_string(),
                                },
                            ))
                        }
                    }
                }
            }
            Err(CompileError::from_error_kind(
                CompileErrorKind::FunctionNotFound {
                    name: name.to_string(),
                },
            ))
        }
    }

    pub fn find_variable(&self, name: &str) -> Result<(ResolvedType, PointerValue), CompileError> {
        for scope in self.scopes.iter().rev() {
            if let Some(reg) = scope.find(name) {
                match reg {
                    Registration::Variable(VariableRegistration {
                        ns: _,
                        resolved_ty,
                        value,
                    }) => return Ok((resolved_ty.clone(), *value)),
                    _ => {
                        return Err(CompileError::from_error_kind(
                            CompileErrorKind::IsNotVariable {
                                name: name.to_string(),
                            },
                        ))
                    }
                }
            }
        }
        Err(CompileError::from_error_kind(
            CompileErrorKind::VariableNotFound {
                name: name.to_string(),
            },
        ))
    }
    pub fn find_type(&self, name: &str) -> Result<ResolvedType, CompileError> {
        for scope in self.scopes.iter().rev() {
            if let Some(reg) = scope.find(name) {
                match reg {
                    Registration::Type(TypeRegistration {
                        ns: _,
                        name: _,
                        resolved_ty,
                    }) => return Ok(resolved_ty.clone()),
                    _ => {
                        return Err(CompileError::from_error_kind(
                            CompileErrorKind::IsNotVariable {
                                name: name.to_string(),
                            },
                        ))
                    }
                }
            }
        }
        Err(CompileError::from_error_kind(
            CompileErrorKind::VariableNotFound {
                name: name.to_string(),
            },
        ))
    }
    pub fn register_into_current_scope(&mut self, name: &str, reg: Registration<'a>) {
        let current_scope = self.scopes.last_mut().unwrap();
        current_scope.set(name.to_string(), reg);
    }
}
