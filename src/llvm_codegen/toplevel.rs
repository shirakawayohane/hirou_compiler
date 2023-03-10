use super::error::{CompileError, ContextType};
use super::*;
use crate::{ast::*, error_context};

impl LLVMCodegenerator<'_> {
    pub(super) fn gen_function_impl(
        &mut self,
        func: &Function,
        generic_args: &[ResolvedType],
        arg_types: &[ResolvedType],
    ) -> Result<(), CompileError> {
        // TODO: int以外の型にも対応する
        let params = func
            .decl
            .params
            .iter()
            .map(|_| self.llvm_context.i32_type().into())
            .collect::<Vec<_>>();
        let fn_type = self.llvm_context.i32_type().fn_type(&params, true);
        let function_value = self
            .llvm_module
            .add_function(&func.decl.name, fn_type, None);
        let entry_basic_block = self
            .llvm_context
            .append_basic_block(function_value, "entry");
        self.llvm_builder.position_at_end(entry_basic_block);

        // パラメーターをFunctionBodyにallocし、Contextにも登録する
        self.push_scope(ScopeKind::Function);
        {
            // Set parameters in function body
            for (i, (loc_ty, name)) in func.decl.params.into_iter().enumerate() {
                let resolved_ty = self.resolve_type(&loc_ty.value)?;
                let parameter = function_value.get_nth_param(i as u32).unwrap();
                parameter.set_name(name.as_str());
                if let ResolvedType::Void = &resolved_ty {
                    continue;
                } else {
                    let allocated_pointer =
                        self.llvm_builder.build_alloca(parameter.get_type(), &name);
                    self.llvm_builder.build_store(allocated_pointer, parameter);
                    self.set_variable(name.clone(), loc_ty.value, allocated_pointer);
                }
            }
        }

        for statement in func.body {
            self.gen_statement(statement.value)?;
        }

        self.pop_scope();
        Ok(())
    }
    pub(super) fn gen_toplevel(&mut self, top: TopLevel) -> Result<(), CompileError> {
        match top {
            TopLevel::Function(func) => {
                if func.decl.generic_args.is_some() {
                    self.register_function(func);
                    Ok(())
                } else {
                    self.register_function(func);
                    Ok(())
                }
            }
        }
    }
}
