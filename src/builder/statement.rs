use super::*;
use crate::resolved_ast::*;

impl LLVMCodeGenerator<'_> {
    pub(super) fn gen_variable_decl(&mut self, decl: &VariableDecl) {
        let ty = self.type_to_basic_type_enum(&decl.value.ty).unwrap();
        let ptr = self.llvm_builder.build_alloca(ty, &decl.name);

        self.add_variable(&decl.name, ptr.clone());

        let value = self.gen_expression(&decl.value).unwrap();
        self.llvm_builder.build_store(ptr, value);
    }
    pub(super) fn gen_return(&mut self, ret: &Return) {
        if let Some(expression) = &ret.expression {
            let value = &self.gen_expression(expression).unwrap();
            self.llvm_builder.build_return(Some(value));
        } else {
            self.llvm_builder.build_return(None);
        };
    }
    pub(super) fn gen_effect(&self, effect: &Effect) {
        self.gen_expression(&effect.expression);
    }
    pub(super) fn gen_assignment(&self, assignment: &Assignment) {
        let value = self.gen_expression(&assignment.expression).unwrap();
        let ptr = self.get_variable(&assignment.name);
        self.build_store(ptr, value);
    }
    pub(super) fn gen_statement(&mut self, statement: &Statement) {
        match &statement {
            Statement::VariableDecl(decl) => self.gen_variable_decl(decl),
            Statement::Return(ret) => self.gen_return(ret),
            Statement::Effect(effect) => self.gen_effect(effect),
            Statement::Assignmetn(assignment) => self.gen_assignment(assignment),
        }
    }
}
