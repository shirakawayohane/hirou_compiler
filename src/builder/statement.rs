use inkwell::{
    builder::BuilderError,
    types::BasicType,
    values::{BasicValue, InstructionValue},
    AddressSpace,
};

use super::*;
use crate::resolved_ast::*;

impl LLVMCodeGenerator<'_> {
    pub(super) fn gen_variable_decl(&mut self, decl: &VariableDecl) -> Result<(), BuilderError> {
        let ty = self.type_to_basic_type_enum(&decl.value.ty).unwrap();
        let value = self.gen_expression(&decl.value)?.unwrap();
        if ty.is_struct_type() {
            let ptr = self.llvm_builder.build_alloca(ty, "")?;
            self.llvm_builder.build_memcpy(
                ptr,
                8,
                value.into_pointer_value(),
                8,
                ty.size_of().unwrap(),
            )?;
            self.add_variable(&decl.name, ptr);
        } else {
            let ptr = self.llvm_builder.build_alloca(ty, "")?;
            self.llvm_builder.build_store(ptr, value)?;
            self.add_variable(&decl.name, ptr);
        }
        // let ptr = self.llvm_builder.build_alloca(ty, "").unwrap();
        // self.add_variable(&decl.name, ptr);
        // let value = self.gen_expression(&decl.value)?.unwrap();
        // if value.is_pointer_value() {
        //     self.build_memcpy(
        //         value.into_pointer_value(),
        //         ptr,
        //         ty.size_of().unwrap().into(),
        //     )?;
        //     Ok(())
        // } else {
        //     self.llvm_builder.build_store(ptr, value)?;
        //     Ok(())
        // }
        Ok(())
    }
    pub(super) fn gen_return(&mut self, ret: &Return) -> Result<InstructionValue, BuilderError> {
        if let Some(expression) = &ret.expression {
            let value = self.gen_expression(expression)?.unwrap();
            let ptr = self.llvm_builder.build_alloca(value.get_type(), "")?;
            if value.is_struct_value() {
                dbg!("value is struct type");
                self.llvm_builder.build_call(
                    self.llvm_module.get_function("memcpy").unwrap(),
                    &[value
                        .get_type()
                        .size_of()
                        .unwrap()
                        .as_basic_value_enum()
                        .into()],
                    "memcpy",
                )?;
            } else {
                self.llvm_builder.build_store(ptr, value)?;
            }
            self.llvm_builder.build_return(Some(&value))
        } else {
            self.llvm_builder.build_return(None)
        }
    }
    pub(super) fn gen_effect(&self, effect: &Effect) -> Result<(), BuilderError> {
        self.gen_expression(&effect.expression)?;
        Ok(())
    }
    pub(super) fn gen_assignment(&self, assignment: &Assignment) -> Result<(), BuilderError> {
        let value = self.gen_expression(&assignment.expression)?.unwrap();
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
            let index = self.gen_expression(index_access)?.unwrap();
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
            if assignment.expression.ty.is_struct_type() {
                self.llvm_builder.build_memcpy(
                    ptr,
                    8,
                    value.into_pointer_value(),
                    8,
                    pointee_type.size_of().unwrap(),
                )?;
                return Ok(());
            }
        }
        self.llvm_builder.build_store(ptr, value)?;
        Ok(())
    }
    pub(super) fn gen_statement(
        &mut self,
        statement: &Statement,
    ) -> Result<Option<InstructionValue>, BuilderError> {
        match &statement {
            Statement::VariableDecl(decl) => {
                self.gen_variable_decl(decl)?;
                Ok(None)
            }
            Statement::Return(ret) => self.gen_return(ret).map(Some),
            Statement::Effect(effect) => {
                self.gen_effect(effect)?;
                Ok(None)
            }
            Statement::Assignment(assignment) => {
                self.gen_assignment(assignment)?;
                Ok(None)
            }
        }
    }
}
