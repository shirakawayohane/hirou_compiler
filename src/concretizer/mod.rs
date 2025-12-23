mod toplevel;

use crate::{
    common::target::PointerSizedIntWidth,
    concrete_ast::ConcreteModule,
    resolved_ast::ResolvedModule,
};

pub struct ConcretizerContext {
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
    let context = ConcretizerContext { ptr_sized_int_type };

    let mut toplevels = Vec::new();

    for toplevel in resolved_module.toplevels {
        if let Some(concrete_toplevels) = toplevel::concretize_toplevel(&context, &toplevel) {
            toplevels.extend(concrete_toplevels);
        }
    }

    ConcreteModule { toplevels }
}
