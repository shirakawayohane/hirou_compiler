mod context;
mod error;
mod expression;
mod intrinsic;
mod statement;
mod toplevel;
mod value;

use self::context::{Context, GenericFunctionRegistration, RegisteredFunction, Scope, ScopeKind};
use self::error::{CompileError, CompileErrorKind};
use self::value::Value;
use crate::ast::{Function, Module, ResolvedType, UnresolvedType};
use inkwell::builder::Builder as LLVMBuilder;
use inkwell::context::Context as LLVMContext;
use inkwell::module::Module as LLVMModule;
use inkwell::types::IntType;
use inkwell::values::{FunctionValue, PointerValue};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum PointerSize {
    SixteenFour,
}

pub struct LLVMCodegenerator<'a> {
    context: Context<'a>,
    llvm_module: LLVMModule<'a>,
    llvm_builder: LLVMBuilder<'a>,
    llvm_context: &'a LLVMContext,
    i8_type: IntType<'a>,
    i32_type: IntType<'a>,
    i64_type: IntType<'a>,
    pointer_size: PointerSize,
}

impl<'a> LLVMCodegenerator<'a> {
    fn gen_load<'ptr>(
        &self,
        ty: &ResolvedType,
        pointer: PointerValue<'ptr>,
    ) -> Result<Value<'ptr>, CompileError>
    where
        'a: 'ptr,
    {
        let value = self.llvm_builder.build_load(pointer, "load");
        return Ok(match ty {
            ResolvedType::I32 => Value::I32Value(value.into_int_value()),
            ResolvedType::U8 => Value::U8Value(value.into_int_value()),
            ResolvedType::U32 => Value::U32Value(value.into_int_value()),
            ResolvedType::U64 => Value::U64Value(value.into_int_value()),
            ResolvedType::USize => Value::USizeValue(value.into_int_value()),
            ResolvedType::Ptr(pointer_of) => {
                Value::PointerValue(pointer_of.clone(), value.into_pointer_value())
            }
            ResolvedType::Void => Value::Void,
        });
    }
    pub fn new(llvm_context: &'a LLVMContext) -> LLVMCodegenerator<'a> {
        let llvm_module = llvm_context.create_module("main");
        let llvm_builder = llvm_context.create_builder();
        let i8_type = llvm_context.i8_type();
        let i32_type = llvm_context.i32_type();
        let i64_type = llvm_context.i64_type();
        Self {
            context: Context::new(),
            llvm_module,
            llvm_builder,
            llvm_context,
            i8_type,
            i32_type,
            i64_type,
            pointer_size: PointerSize::SixteenFour,
        }
    }
}

impl<'a> LLVMCodegenerator<'a> {
    pub fn gen_module(&mut self, module: Module) -> Result<LLVMModule<'a>, CompileError> {
        // Add global scope
        self.push_variable_scope(ScopeKind::Global);
        self.push_function_scope();
        self.push_type_scope();

        self.gen_intrinsic_functions_on_llvm();
        self.prepare_intrinsic_types();
        for top in module.toplevels {
            self.gen_toplevel(top.value)?;
        }
        // Pop global scope
        self.pop_type_scope();
        self.pop_function_scope();
        self.pop_variable_scope();

        Ok(self.llvm_module.clone())
    }

    pub fn find_variable(
        &self,
        name: &str,
    ) -> Result<(UnresolvedType, PointerValue), CompileError> {
        for scope in self.context.variables.iter().rev() {
            if let Some((ty, ptr_value)) = scope.variables.get(name) {
                return Ok((ty.clone(), *ptr_value));
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
                for types_in_scope in self.context.types.iter().rev() {
                    if let Some(v) = types_in_scope.get(unresolved_ty) {
                        return Ok(v.clone());
                    }
                }
                Err(CompileError::from_error_kind(
                    CompileErrorKind::TypeNotFound {
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
    pub fn find_function(&self, name: &str) -> Option<&RegisteredFunction> {
        for function in self.context.functions.iter().rev() {
            if let Some(v) = function.get(name) {
                return Some(v);
            }
        }
        None
    }
    pub fn set_variable(&mut self, name: String, ty: UnresolvedType, pointer: PointerValue<'a>) {
        self.context
            .variables
            .last_mut()
            .unwrap()
            .variables
            .insert(name, (ty, pointer));
    }
    pub fn set_type(&mut self, unresolved: UnresolvedType, resolved: ResolvedType) {
        self.context
            .types
            .last_mut()
            .unwrap()
            .insert(unresolved, resolved);
    }
    pub fn set_function(
        &mut self,
        name: String,
        return_type: UnresolvedType,
        arg_types: Vec<UnresolvedType>,
        function_value: FunctionValue<'a>,
    ) {
        self.context.functions.last_mut().unwrap().insert(
            name,
            RegisteredFunction {
                return_type,
                arg_types,
                function_value,
            },
        );
    }
    pub fn register_generic_function(&mut self, func: Function) {
        self.context.generic_functions.last_mut().unwrap().insert(
            func.decl.name.to_owned(),
            GenericFunctionRegistration {
                scope_depth: self.context.variables.len() as u16,
                function: func,
            },
        );
    }
    pub fn push_variable_scope(&mut self, kind: ScopeKind) {
        self.context.variables.push(Scope::new(kind));
    }
    pub fn pop_variable_scope(&mut self) {
        self.context.variables.pop();
    }
    pub fn push_function_scope(&mut self) {
        self.context.functions.push(HashMap::new());
        self.context.generic_functions.push(HashMap::new());
    }
    pub fn pop_function_scope(&mut self) {
        self.context.functions.pop();
        self.context.generic_functions.pop();
    }
    pub fn push_type_scope(&mut self) {
        self.context.types.push(HashMap::new());
    }
    pub fn pop_type_scope(&mut self) {
        self.context.types.pop();
    }
}
