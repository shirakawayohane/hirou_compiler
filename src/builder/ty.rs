use inkwell::{
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum},
    AddressSpace,
};

use crate::resolved_ast::ResolvedType;

use super::LLVMCodeGenerator;

impl<'a> LLVMCodeGenerator<'a> {
    pub fn type_to_basic_type_enum(&self, ty: &ResolvedType) -> Option<BasicTypeEnum<'a>> {
        Some(match ty {
            ResolvedType::I32 => BasicTypeEnum::IntType(self.llvm_context.i32_type()),
            ResolvedType::U8 => BasicTypeEnum::IntType(self.llvm_context.i8_type()),
            ResolvedType::U32 => BasicTypeEnum::IntType(self.llvm_context.i32_type()),
            ResolvedType::U64 => BasicTypeEnum::IntType(self.llvm_context.i64_type()),
            ResolvedType::I64 => BasicTypeEnum::IntType(self.llvm_context.i64_type()),
            ResolvedType::USize => BasicTypeEnum::IntType(self.llvm_context.i64_type()),
            ResolvedType::Ptr(inner) => {
                BasicTypeEnum::PointerType(if let Some(t) = self.type_to_basic_type_enum(inner) {
                    t.ptr_type(AddressSpace::default())
                } else {
                    // Void Pointer Type
                    self.llvm_context
                        .i8_type()
                        .ptr_type(AddressSpace::default())
                })
            }
            ResolvedType::Void => return None,
            ResolvedType::Unknown => unreachable!(),
        })
    }
    pub fn type_to_basic_metadata_type_enum(
        &self,
        ty: &ResolvedType,
    ) -> Option<BasicMetadataTypeEnum<'a>> {
        Some(match ty {
            ResolvedType::I32 => BasicMetadataTypeEnum::IntType(self.llvm_context.i32_type()),
            ResolvedType::U8 => BasicMetadataTypeEnum::IntType(self.llvm_context.i8_type()),
            ResolvedType::U32 => BasicMetadataTypeEnum::IntType(self.llvm_context.i32_type()),
            ResolvedType::U64 => BasicMetadataTypeEnum::IntType(self.llvm_context.i64_type()),
            ResolvedType::I64 => BasicMetadataTypeEnum::IntType(self.llvm_context.i64_type()),
            ResolvedType::USize => BasicMetadataTypeEnum::IntType(self.llvm_context.i64_type()),
            ResolvedType::Ptr(inner) => BasicMetadataTypeEnum::PointerType(
                if let Some(t) = self.type_to_basic_type_enum(inner) {
                    t.ptr_type(AddressSpace::default())
                } else {
                    // Void Pointer Type
                    self.llvm_context
                        .i8_type()
                        .ptr_type(AddressSpace::default())
                },
            ),
            ResolvedType::Void => return None,
            ResolvedType::Unknown => unimplemented!(),
        })
    }
}
