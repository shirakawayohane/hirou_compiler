mod expression;
mod statement;
mod toplevel;
mod ty;

use inkwell::OptimizationLevel;

use crate::common::target::TargetPlatform;
use crate::concrete_ast::*;
use inkwell::builder::Builder as LLVMBuilder;
use inkwell::context::Context as LLVMContext;
use inkwell::module::Module as LLVMModule;
use inkwell::targets::{InitializationConfig, Target};
use inkwell::values::PointerValue;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScopeKind {
    Global,
    Function,
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
            values: HashMap::new(),
        }
    }
}

pub struct LLVMCodeGenerator<'a> {
    llvm_module: LLVMModule<'a>,
    llvm_builder: LLVMBuilder<'a>,
    llvm_context: &'a LLVMContext,
    scopes: Vec<RefCell<Scope<'a>>>,
    function_by_name: HashMap<String, &'a Function>,
}

impl<'a> LLVMCodeGenerator<'a> {
    pub fn new(
        llvm_context: &'a LLVMContext,
        _target: TargetPlatform,
        _optimization_level: OptimizationLevel,
        module: &'a ConcreteModule,
    ) -> Self {
        let llvm_module = llvm_context.create_module("main");
        let llvm_builder = llvm_context.create_builder();

        Target::initialize_all(&InitializationConfig {
            asm_parser: false,
            asm_printer: false,
            base: true,
            disassembler: false,
            info: true,
            machine_code: true,
        });

        // let triple = TargetTriple::create(target.metrics().target_triplet);
        // let target = Target::from_triple(&triple).unwrap();
        // let host_cpu = TargetMachine::get_host_cpu_name();
        // let features = TargetMachine::get_host_cpu_features();
        // let target_machine = target
        //     .create_target_machine(
        //         &triple,
        //         host_cpu.to_str().unwrap(),
        //         features.to_str().unwrap(),
        //         optimization_level,
        //         RelocMode::Default,
        //         CodeModel::Default,
        //     )
        //     .unwrap();

        let mut function_by_name = HashMap::new();
        for toplevel in &module.toplevels {
            match &toplevel {
                TopLevel::Function(func) => {
                    function_by_name.insert(func.decl.name.clone(), func);
                }
            }
        }

        Self {
            llvm_module,
            llvm_builder,
            llvm_context,
            scopes: Vec::new(),
            function_by_name,
        }
    }
    pub fn gen_module(&mut self, module: &'a ConcreteModule) {
        self.scopes
            .push(RefCell::new(Scope::new(ScopeKind::Global)));

        // self.gen_intrinsic_functions_on_llvm();
        for top in &module.toplevels {
            self.gen_toplevel(top);
        }

        for top in &module.toplevels {
            match top {
                TopLevel::Function(func) => self.gen_function_body(func).unwrap(),
            }
        }

        self.scopes.pop();
    }
    pub fn get_module(self) -> LLVMModule<'a> {
        self.llvm_module
    }
    fn add_variable(&self, name: &str, value: PointerValue<'a>) {
        self.scopes
            .last()
            .unwrap()
            .borrow_mut()
            .values
            .insert(name.into(), value);
    }
    fn get_variable(&self, name: &str) -> PointerValue<'a> {
        *self
            .scopes
            .last()
            .unwrap()
            .borrow()
            .values
            .get(name)
            .unwrap()
    }
    fn pop_scope(&mut self) {
        self.scopes.pop();
    }
    fn push_scope(&mut self, scope: Scope<'a>) {
        self.scopes.push(RefCell::new(scope));
    }
}
