mod context;
mod expression;
mod intrinsic;
mod statement;
mod toplevel;
mod value;

use crate::ast::{Module, TopLevel};
use inkwell::builder::Builder as LLVMBuilder;
use inkwell::context::Context as LLVMContext;
use inkwell::module::Module as LLVMModule;
use inkwell::values::PointerValue;

use std::cell::RefCell;

use std::rc::Rc;
use thiserror::Error;

use self::context::Context;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("Variable `{name:?}` is not found in this scope.")]
    VariableNotFound { name: String },
    #[error("Function `{name:?}` is not found.")]
    FunctionNotFound { name: String },
    #[error("`{name:?}` is not a function")]
    CallNotFunctionValue { name: String },
    #[error("Invalid operand.")]
    InvalidOperand,
    #[error("Invalid operand.")]
    InvalidArgument,
}

pub struct LLVMCodegenerator<'a> {
    context: Rc<RefCell<Context<'a>>>,
    llvm_module: LLVMModule<'a>,
    llvm_builder: LLVMBuilder<'a>,
    llvm_context: &'a LLVMContext,
}

impl<'a> LLVMCodegenerator<'a> {
    pub fn new(llvm_context: &'a LLVMContext) -> LLVMCodegenerator<'a> {
        let llvm_module = llvm_context.create_module("main");
        let llvm_builder = llvm_context.create_builder();
        Self {
            context: Rc::new(RefCell::new(Context::new())),
            llvm_module,
            llvm_builder,
            llvm_context,
        }
    }
}

impl<'a> LLVMCodegenerator<'a> {
    fn get_variable(&self, name: &str) -> Result<PointerValue, CompileError> {
        for scope in self.context.borrow().scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Ok(*v);
            }
        }
        Err(CompileError::VariableNotFound {
            name: name.to_string(),
        })
    }

    pub fn gen_module(self, module: Module) -> Result<LLVMModule<'a>, CompileError> {
        self.gen_intrinsic_functions();
        for top in module.toplevels {
            match top {
                TopLevel::Function { decl, body } => self.gen_function(decl, body)?,
            }
        }
        Ok(self.llvm_module)
    }
}
