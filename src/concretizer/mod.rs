use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    common::target::PointerSizedIntWidth,
    concrete_ast::ConcreteModule,
    resolved_ast::{self, ResolvedModule},
    resolver::ResolverContext,
};

pub struct ConcretizerContext {
    pub resolved_types: Rc<RefCell<HashMap<String, resolved_ast::ResolvedType>>>,
    pub function_by_name: Rc<RefCell<HashMap<String, resolved_ast::Function>>>,
    pub interface_by_name: Rc<RefCell<HashMap<String, resolved_ast::Interface>>>,
    pub impls_by_name: Rc<RefCell<HashMap<String, Vec<resolved_ast::Implementation>>>>,
    pub ptr_sized_int_type: PointerSizedIntWidth,
}

impl ConcretizerContext {
    pub fn from_resolved_module(
        context: &ResolverContext,
        resolved_module: ResolvedModule,
    ) -> Self {
        let ret = Self {
            resolved_types: Default::default(),
            function_by_name: Default::default(),
            interface_by_name: Default::default(),
            impls_by_name: Default::default(),
            ptr_sized_int_type: context.ptr_sized_int_type,
        };
        for toplevel in resolved_module.toplevels {
            match toplevel {
                resolved_ast::TopLevel::Function(func) => {
                    let name = func.decl.name.clone();
                    ret.function_by_name.borrow_mut().insert(name, func.clone());
                }
                resolved_ast::TopLevel::Implemantation(imp) => {
                    let mut impls = ret.impls_by_name.borrow_mut();
                    let name = imp.decl.name.clone();
                    let entry = impls.entry(name).or_insert_with(Vec::new);
                    entry.push(imp);
                }
                resolved_ast::TopLevel::Interface(interface) => {
                    let name = interface.name.clone();
                    ret.interface_by_name
                        .borrow_mut()
                        .insert(name, interface.clone());
                }
            }
        }
        ret
    }
}

pub fn concretize_module(context: &ConcretizerContext) -> ConcreteModule {
    todo!()
}
