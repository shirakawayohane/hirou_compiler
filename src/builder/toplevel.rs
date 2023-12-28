use inkwell::{
    builder::BuilderError,
    types::{BasicType, BasicTypeEnum},
    values::FunctionValue,
};

use super::*;
use crate::resolved_ast::*;

impl<'a> LLVMCodeGenerator<'a> {
    pub(super) fn gen_or_get_function(&self, function: &Function) -> FunctionValue {
        if let Some(ret) = self.llvm_module.get_function(&function.decl.name) {
            return ret;
        }

        let param_types = function
            .decl
            .args
            .iter()
            .filter_map(|x| match x {
                Argument::VarArgs => None,
                Argument::Normal(ty, _) => self.type_to_basic_metadata_type_enum(ty),
            })
            .collect::<Vec<_>>();

        let has_var_args = function
            .decl
            .args
            .iter()
            .any(|x| matches!(x, Argument::VarArgs));

        let function = self.llvm_module.add_function(
            &function.decl.name,
            if let Some(return_type) = self.type_to_basic_type_enum(&function.decl.return_type) {
                let basic_type = BasicTypeEnum::try_from(return_type).unwrap();
                basic_type.fn_type(&param_types, has_var_args)
            } else {
                self.llvm_context.void_type().fn_type(&param_types, false)
            },
            None,
        );

        function
    }

    pub(super) fn gen_function_body(&mut self, function: &'a Function) -> Result<(), BuilderError> {
        if function.body.len() == 0 {
            return Ok(());
        }

        let function_value = self.llvm_module.get_function(&function.decl.name).unwrap();
        let entry_basic_block = self
            .llvm_context
            .append_basic_block(function_value, "entry");

        let scope = Scope::new(ScopeKind::Function);
        self.push_scope(scope);
        {
            self.llvm_builder.position_at_end(entry_basic_block);

            // Set parameters in function body
            // Generate function body
            for (i, (_ty, name)) in function
                .decl
                .args
                .iter()
                .map(|x| match x {
                    Argument::VarArgs => unimplemented!(),
                    Argument::Normal(ty, name) => (ty, name),
                })
                .enumerate()
            {
                let parameter = function_value.get_nth_param(i as u32).unwrap();
                parameter.set_name(name.as_str());
                let allocated_pointer = self
                    .llvm_builder
                    .build_alloca(parameter.get_type(), &name)
                    .unwrap();
                self.llvm_builder
                    .build_store(allocated_pointer, parameter)
                    .unwrap();
                self.add_variable(name, allocated_pointer);
            }

            // Generate function body
            for statement in &function.body {
                self.gen_statement(&statement)?;
            }
        }
        self.pop_scope();
        Ok(())
    }

    pub(super) fn gen_toplevel(&mut self, top: &'a TopLevel) {
        match top {
            TopLevel::Function(func) => self.gen_or_get_function(func),
        };
    }
}
