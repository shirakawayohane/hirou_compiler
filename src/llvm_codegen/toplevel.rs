use super::*;
use crate::ast::*;

impl LLVMCodegenerator<'_> {
    pub(super) fn gen_function(
        &self,
        decl: FunctionDecl,
        body: Vec<Statement>,
    ) -> Result<(), CompileError> {
        // TODO: int以外の型にも対応する
        let params = decl
            .params
            .iter()
            .map(|_| self.llvm_context.i32_type().into())
            .collect::<Vec<_>>();
        let fn_type = self.llvm_context.i32_type().fn_type(&params, true);
        let function = self.llvm_module.add_function(&decl.name, fn_type, None);
        let entry_basic_block = self.llvm_context.append_basic_block(function, "entry");
        self.llvm_builder.position_at_end(entry_basic_block);

        // パラメーターをFunctionBodyにallocし、Contextにも登録する
        self.context.borrow_mut().push_scope();
        // Set parameters in function body
        for (i, parameter) in function.get_param_iter().enumerate() {
            let (ty, name) = &decl.params[i];
            parameter.set_name(name.as_str());
            match ty {
                Type::I32 => {
                    let alloca = self.llvm_builder.build_alloca(parameter.get_type(), &name);
                    self.llvm_builder.build_store(alloca, parameter);
                    self.context.borrow_mut().set_variable(name.clone(), alloca);
                }
                Type::I64 => todo!(),
                Type::U8 => todo!(),
                Type::Ptr(_) => todo!(),
            }
        }

        for statement in body {
            self.gen_statement(statement)?;
        }

        self.context.borrow_mut().pop_scope();
        Ok(())
    }
}
