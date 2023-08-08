mod context;
mod error;
mod expression;
mod intrinsic;
mod result_value;
mod statement;
mod toplevel;

use self::context::{Context, FunctionImpl, Registration, Scope, ScopeKind};
use self::error::{CompileError, CompileErrorKind};
use self::result_value::Value;
use crate::ast::{Function, FunctionDecl, Module, ResolvedType, UnresolvedType};
use inkwell::builder::Builder as LLVMBuilder;
use inkwell::context::Context as LLVMContext;
use inkwell::module::Module as LLVMModule;
use inkwell::types::IntType;
use inkwell::values::PointerValue;

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

pub fn mangle_function_name(ns: &str, function_decl: &FunctionDecl) -> String {
    todo!()
}

pub fn mangle_type_name(ns: &str, unresolved_ty: &UnresolvedType) -> String {
    todo!()
}

pub fn mangle_global_variable_name(ns: &str) -> String {
    todo!()
}

impl<'a> LLVMCodegenerator<'a> {
    fn gen_load<'ptr>(
        &self,
        resolved_ty: &ResolvedType,
        pointer: PointerValue<'ptr>,
    ) -> Result<Value<'ptr>, CompileError>
    where
        'a: 'ptr,
    {
        let value = self.llvm_builder.build_load(pointer, "load");
        return Ok(match resolved_ty {
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
        self.push_scope(ScopeKind::Global);

        self.gen_intrinsic_functions_on_llvm();
        self.prepare_intrinsic_types();
        for top in module.toplevels {
            self.gen_toplevel(top.value)?;
        }

        self.pop_scope();

        Ok(self.llvm_module.clone())
    }

    pub fn find_variable(
        &self,
        name: &str,
    ) -> Result<(UnresolvedType, PointerValue), CompileError> {
        for scope in self.context.scopes.iter().rev() {
            if let Some(reg) = scope.find(name) {
                match reg {
                    Registration::Variable { ns: _, ty, value } => return Ok((ty.clone(), *value)),
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
    pub fn resolve_type(
        &self,
        unresolved_ty: &UnresolvedType,
    ) -> Result<ResolvedType, CompileError> {
        match unresolved_ty {
            UnresolvedType::TypeRef {
                name,
                generic_args: _,
            } => {
                for scope in self.context.scopes.iter().rev() {
                    if let Some(reg) = scope.find(name) {
                        match reg {
                            Registration::Type { ns, unresolved_ty } => {
                                let mangled_type_name = mangle_type_name(ns, &unresolved_ty);
                                if let Some(ty) =
                                    self.context.resolved_types.get(&mangled_type_name)
                                {
                                    return Ok(ty.clone());
                                }
                            }
                            _ => {
                                return Err(CompileError::from_error_kind(
                                    CompileErrorKind::IsNotType {
                                        name: name.to_string(),
                                    },
                                ))
                            }
                        }
                    }
                }
                Err(CompileError::from_error_kind(
                    CompileErrorKind::TypeNotFound {
                        name: format!("{}", unresolved_ty),
                    },
                ))
            }
            UnresolvedType::Ptr(inner_ty) => {
                let resolved_inner_ty = self.resolve_type(inner_ty)?;
                Ok(ResolvedType::Ptr(
                    Box::new(resolved_inner_ty.clone()).clone(),
                ))
            }
        }
    }
    pub fn find_function_impl(
        &self,
        name: &str,
        types: &[ResolvedType],
    ) -> Result<&FunctionImpl, CompileError> {
        for scope in self.context.scopes.iter().rev() {
            if let Some(reg) = scope.find(name) {
                match reg {
                    Registration::Function { ns, function } => todo!(),
                    _ => {
                        return Err(CompileError::from_error_kind(
                            CompileErrorKind::CallNotFunctionValue {
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
    pub fn set_variable(&mut self, name: String, ty: UnresolvedType, pointer: PointerValue<'a>) {
        let current_ns = self.context.get_current_ns();
        self.context.scopes.last_mut().unwrap().set(
            name,
            Registration::Variable {
                ns: current_ns,
                ty,
                value: pointer,
            },
        );
    }
    pub fn set_type(&mut self, name: String, unresolved_ty: UnresolvedType) {
        let current_ns = self.context.get_current_ns();
        self.context.scopes.last_mut().unwrap().set(
            name,
            Registration::Type {
                ns: current_ns,
                unresolved_ty,
            },
        )
    }
    pub fn register_function(&mut self, func: Function) {
        let current_ns = self.context.get_current_ns();
        let func_name = func.decl.name.clone();
        self.context.scopes.last_mut().unwrap().set(
            func_name,
            Registration::Function {
                ns: current_ns,
                function: func,
            },
        );
    }
    pub fn push_scope(&mut self, kind: ScopeKind) {
        self.context.scopes.push(Scope::new(kind));
    }
    pub fn pop_scope(&mut self) {
        self.context.scopes.pop();
    }
}
