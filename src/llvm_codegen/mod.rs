mod context;
mod error;
mod expression;
mod intrinsic;
mod statement;
mod toplevel;
mod value;

use crate::ast::{Module, ResolvedType};
use inkwell::builder::Builder as LLVMBuilder;
use inkwell::context::Context as LLVMContext;
use inkwell::module::Module as LLVMModule;
use inkwell::types::IntType;
use inkwell::values::PointerValue;

use std::cell::RefCell;

use std::rc::Rc;

use self::context::Context;
use self::error::{CompileError, CompileErrorKind};
use self::value::Value;

#[derive(Debug, Clone, Copy)]
pub enum PointerSize {
    SixteenFour,
}

pub struct LLVMCodegenerator<'a> {
    context: Rc<RefCell<Context<'a>>>,
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
            context: Rc::new(RefCell::new(Context::new())),
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
    pub fn gen_module(self, module: Module) -> Result<LLVMModule<'a>, CompileError> {
        // Add global scope
        {
            let mut context = self.context.borrow_mut();
            context.push_variable_scope();
            context.push_function_scope();
            context.push_type_scope();
        }
        self.gen_intrinsic_functions();
        self.prepare_intrinsic_types();
        for top in module.toplevels {
            self.gen_toplevel(top.value)?;
        }
        {
            let mut context = self.context.borrow_mut();
            context.pop_type_scope();
            context.pop_function_scope();
            context.pop_variable_scope();
        }
        Ok(self.llvm_module)
    }
}
