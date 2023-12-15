mod expression;
mod statement;
mod target;
mod toplevel;
mod ty;
mod util;

use inkwell::OptimizationLevel;
use inkwell::types::IntType;
pub use target::TargetPlatform;

use self::target::TargetMetrics;
use crate::resolved_ast::*;
use inkwell::builder::Builder as LLVMBuilder;
use inkwell::context::Context as LLVMContext;
use inkwell::module::Module as LLVMModule;
use inkwell::targets::{CodeModel, RelocMode, Target, TargetMachine, TargetTriple};
use inkwell::values::{IntValue, PointerValue};
use std::collections::{HashMap, HashSet};

type MangledName = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScopeKind {
    Global,
    Function,
}

#[derive(Debug)]
pub struct VariableRegistration<'a> {
    pub ns: String,
    pub resolved_ty: ResolvedType,
    pub value: PointerValue<'a>,
}

#[derive(Debug)]
pub struct TypeRegistration<'a> {
    pub ns: String,
    pub name: String,
    pub resolved_ty: &'a ResolvedType,
}

#[derive(Debug)]
pub struct Scope<'a> {
    pub kind: ScopeKind,
    pub values: HashMap<String, PointerValue<'a>>,
}

impl Scope<'_> {
    pub fn new(kind: ScopeKind) -> Self {
        Self {
            kind,
            values: todo!(),
        }
    }
}

pub struct GenericFunctionImpl {
    pub generic_args: Vec<ResolvedType>,
    pub arg_types: Vec<ResolvedType>,
}

pub struct LLVMCodeGenerator<'a> {
    llvm_module: LLVMModule<'a>,
    llvm_builder: LLVMBuilder<'a>,
    llvm_context: &'a LLVMContext,
    ptr_sized_int_type: IntType<'a>,
    scopes: Vec<Scope<'a>>,
}


impl<'a> LLVMCodeGenerator<'a> {
    pub fn new(llvm_context: &'a LLVMContext, target: TargetPlatform, optimization_level: OptimizationLevel) -> Self {
        let llvm_module = llvm_context.create_module("main");
        let llvm_builder = llvm_context.create_builder();

        let triple = TargetTriple::create(target.metrics().target_triplet);
        let target = Target::from_triple(&triple).unwrap();
        let host_cpu = TargetMachine::get_host_cpu_name();
        let features = TargetMachine::get_host_cpu_features();
        let target_machine = target
            .create_target_machine(
                &triple,
                host_cpu.to_str().unwrap(),
                features.to_str().unwrap(),
                optimization_level.into(),
                RelocMode::Default,
                CodeModel::Default,
            )
            .unwrap();

        Self {
            llvm_module,
            llvm_builder,
            llvm_context,
            ptr_sized_int_type: llvm_context
                .ptr_sized_int_type(&target_machine.get_target_data(), None),
            scopes: Vec::new(),
        }
    }
    pub fn gen_module(&mut self, module: &'a Module) -> LLVMModule<'a> {
        self.scopes.push(Scope::new(ScopeKind::Global));

        // self.gen_intrinsic_functions_on_llvm();
        for top in &module.toplevels {
            self.gen_toplevel(top);
        }

        self.scopes.pop();

        self.llvm_module.clone()
    }
    fn add_variable(&mut self, name: &str, value: PointerValue<'a>) {
        self.scopes
            .last_mut()
            .unwrap()
            .values
            .insert(name.into(), value);
    }
    fn get_variable(&self, name: &str) -> PointerValue<'a> {
        *self.scopes.last().unwrap().values.get(name).unwrap()
    }
    fn pop_scope(&mut self) {
        self.scopes.pop();
    }
    fn push_scope(&mut self, scope: Scope<'a>) {
        self.scopes.push(scope);
    }
}
