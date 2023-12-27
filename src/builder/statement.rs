use inkwell::{types::BasicType, AddressSpace};

use super::*;
use crate::resolved_ast::*;

impl LLVMCodeGenerator<'_> {
    pub(super) fn gen_variable_decl(&mut self, decl: &VariableDecl) {
        let ty = self.type_to_basic_type_enum(&decl.value.ty).unwrap();
        let ptr = self.llvm_builder.build_alloca(ty, "").unwrap();
        self.add_variable(&decl.name, ptr);
        let value = self.gen_expression(&decl.value).unwrap();
        self.llvm_builder.build_store(ptr, value).unwrap();
    }
    pub(super) fn gen_return(&mut self, ret: &Return) {
        if let Some(expression) = &ret.expression {
            let value = &self.gen_expression(expression).unwrap();
            self.llvm_builder.build_return(Some(value)).unwrap()
        } else {
            self.llvm_builder.build_return(None).unwrap()
        };
    }
    pub(super) fn gen_effect(&self, effect: &Effect) {
        self.gen_expression(&effect.expression);
    }
    pub(super) fn gen_assignment(&self, assignment: &Assignment) {
        let value = self.gen_expression(&assignment.expression).unwrap();
        let pointee_type = value.get_type();
        let mut ptr = self.get_variable(&assignment.name);
        for _ in 0..assignment.deref_count {
            ptr = self
                .llvm_builder
                .build_load(pointee_type, ptr, "")
                .unwrap()
                .into_pointer_value();
        }
        if let Some(index_access) = &assignment.index_access {
            let index = self.gen_expression(index_access).unwrap();
            ptr = self
                .llvm_builder
                .build_load(pointee_type.ptr_type(AddressSpace::default()), ptr, "")
                .unwrap()
                .into_pointer_value();

            ptr = unsafe {
                self.llvm_builder
                    .build_in_bounds_gep(pointee_type, ptr, &[index.into_int_value()], "")
                    .unwrap()
            };
        }
        self.llvm_builder.build_store(ptr, value).unwrap();
    }
    pub(super) fn gen_statement(&mut self, statement: &Statement) {
        match &statement {
            Statement::VariableDecl(decl) => self.gen_variable_decl(decl),
            Statement::Return(ret) => self.gen_return(ret),
            Statement::Effect(effect) => self.gen_effect(effect),
            Statement::Assignment(assignment) => self.gen_assignment(assignment),
        }
    }
}
