mod toplevel;

use std::collections::HashMap;

use crate::{
    common::target::PointerSizedIntWidth,
    concrete_ast::{ConcreteModule, TopLevel},
    resolved_ast::{self, ResolvedModule},
};

pub struct ConcretizerContext {
    pub function_by_name: HashMap<String, resolved_ast::Function>,
    pub interface_by_name: HashMap<String, resolved_ast::Interface>,
    pub impls_by_name: HashMap<String, Vec<resolved_ast::Implementation>>,
    pub ptr_sized_int_type: PointerSizedIntWidth,
}

impl ConcretizerContext {
    pub fn is_64_bit(&self) -> bool {
        self.ptr_sized_int_type == PointerSizedIntWidth::SixtyFour
    }
}

pub fn concretize_module(
    resolved_module: ResolvedModule,
    ptr_sized_int_type: PointerSizedIntWidth,
) -> ConcreteModule {
    let mut function_by_name = HashMap::new();
    let mut interface_by_name = HashMap::new();
    let mut impls_by_name: HashMap<String, Vec<resolved_ast::Implementation>> = HashMap::new();

    for toplevel in &resolved_module.toplevels {
        match toplevel {
            resolved_ast::TopLevel::Function(func) => {
                function_by_name.insert(func.decl.name.clone(), func.clone());
            }
            resolved_ast::TopLevel::Implemantation(imp) => {
                let entry = impls_by_name.entry(imp.decl.name.clone()).or_default();
                entry.push(imp.clone());
            }
            resolved_ast::TopLevel::Interface(interface) => {
                interface_by_name.insert(interface.name.clone(), interface.clone());
            }
        }
    }

    let context = ConcretizerContext {
        function_by_name,
        interface_by_name,
        impls_by_name,
        ptr_sized_int_type,
    };

    let mut toplevels = Vec::new();

    for toplevel in resolved_module.toplevels {
        if let Some(concrete_toplevels) = toplevel::concretize_toplevel(&context, &toplevel) {
            toplevels.extend(concrete_toplevels);
        }
    }

    ConcreteModule { toplevels }
}
