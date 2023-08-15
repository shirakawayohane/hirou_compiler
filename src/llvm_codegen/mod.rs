mod context;
mod error;
mod expression;
mod intrinsic;
mod result_value;
mod statement;
mod toplevel;

use std::cell::RefCell;
use std::rc::Rc;

pub use self::context::{
    Context, FunctionImpl, FunctionRegistration, Registration, Scope, ScopeKind,
    VariableRegistration,
};
use self::error::{CompileError, CompileErrorKind};
use self::result_value::Value;
use crate::ast::{
    Function, Module, ResolvedType, TypeRegistration, UnresolvedType,
    I32_TYPE_NAME, U32_TYPE_NAME, U64_TYPE_NAME, U8_TYPE_NAME, USIZE_TYPE_NAME, VOID_TYPE_NAME,
};
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
    llvm_module: LLVMModule<'a>,
    llvm_builder: LLVMBuilder<'a>,
    llvm_context: &'a LLVMContext,
    i8_type: IntType<'a>,
    i32_type: IntType<'a>,
    i64_type: IntType<'a>,
    pointer_size: PointerSize,
}

pub fn mangle_function_impl_name(ns: &str, name: &str, arg_tys: &[ResolvedType]) -> String {
    fn mangle_function_impl_name_impl(
        result: &mut String,
        ns: &str,
        name: &str,
        arg_tys: &[ResolvedType],
    ) {
        result.push_str(ns);
        result.push('.');
        result.push_str(name);
        if !arg_tys.is_empty() {
            result.push('(');
            for arg_ty in arg_tys {
                result.push_str(&mangle_resolved_ty(arg_ty));
            }
            result.push(')');
        }
    }
    let mut result = String::new();
    mangle_function_impl_name_impl(&mut result, ns, name, arg_tys);
    result
}

pub fn mangled_unresoled_ty(unresolved_ty: &UnresolvedType) -> String {
    format!("{}", unresolved_ty)
}

pub fn mangle_resolved_ty(resolved_ty: &ResolvedType) -> String {
    format!("{}", resolved_ty)
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
    pub fn gen_module(
        &mut self,
        ctx: Rc<RefCell<Context>>,
        module: Module,
    ) -> Result<LLVMModule<'a>, CompileError> {
        ctx.borrow().push_scope(Scope::new(ScopeKind::Global));

        self.gen_intrinsic_functions_on_llvm(ctx.clone());
        for top in module.toplevels {
            self.gen_toplevel(ctx.clone(), top.value)?;
        }

        ctx.borrow().pop_scope();

        Ok(self.llvm_module.clone())
    }
    pub fn resolve_type(
        &self,
        ctx: Rc<RefCell<Context>>,
        unresolved_ty: &UnresolvedType,
    ) -> Result<ResolvedType, CompileError> {
        match unresolved_ty {
            UnresolvedType::TypeRef(typeref) => {
                if let Some(_typeref_prefix) = &typeref.prefix {
                    todo!()
                } else {
                    Ok(match typeref.name.as_str() {
                        VOID_TYPE_NAME => ResolvedType::Void,
                        U8_TYPE_NAME => ResolvedType::U8,
                        I32_TYPE_NAME => ResolvedType::I32,
                        U32_TYPE_NAME => ResolvedType::U32,
                        U64_TYPE_NAME => ResolvedType::U64,
                        USIZE_TYPE_NAME => ResolvedType::USize,
                        _ => {
                            ctx.borrow().find_type(&typeref.name)?;
                            return Err(CompileError::from_error_kind(
                                CompileErrorKind::TypeNotFound {
                                    name: format!("{}", unresolved_ty),
                                },
                            ));
                        }
                    })
                }
            }
            UnresolvedType::Ptr(inner_ty) => {
                let resolved_inner_ty = self.resolve_type(ctx.clone(), inner_ty)?;
                Ok(ResolvedType::Ptr(
                    Box::new(resolved_inner_ty.clone()).clone(),
                ))
            }
        }
    }
    pub fn set_local_variable(
        &mut self,
        ctx: Rc<RefCell<Context>>,
        name: String,
        resolved_ty: ResolvedType,
        pointer: PointerValue<'a>,
    ) {
        let current_ns: String = ctx.borrow().get_current_ns();
        ctx.borrow().register_into_current_scope(
            &name,
            Registration::Variable(VariableRegistration {
                ns: current_ns,
                resolved_ty,
                value: pointer,
            }),
        );
    }
    pub fn set_type(&mut self, ctx: Rc<RefCell<Context>>, name: String, typedef: TypeRegistration) {
        ctx.borrow().register_into_current_scope(&name, Registration::Type(typedef));
    }
    pub fn register_function(&mut self, ctx: Rc<RefCell<Context>>, func: Function) {
        let current_ns = ctx.borrow().get_current_ns();
        let func_name = func.decl.name.clone();
        ctx.borrow().register_into_current_scope(
            &func_name,
            Registration::Function(FunctionRegistration {
                ns: current_ns,
                function: func,
            }),
        );
    }
}
