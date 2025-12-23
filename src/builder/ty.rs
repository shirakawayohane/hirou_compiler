use inkwell::{
    types::{BasicMetadataTypeEnum, BasicTypeEnum},
    AddressSpace,
};

use crate::concrete_ast::{ConcreteStructType, ConcreteType};

use super::LLVMCodeGenerator;

impl<'a> LLVMCodeGenerator<'a> {
    pub fn type_to_basic_type_enum(&self, ty: &ConcreteType) -> Option<BasicTypeEnum<'a>> {
        self.type_to_basic_metadata_type_enum(ty)
            .map(|x| x.try_into().unwrap())
    }
    pub fn type_to_basic_metadata_type_enum(
        &self,
        ty: &ConcreteType,
    ) -> Option<BasicMetadataTypeEnum<'a>> {
        Some(match ty {
            ConcreteType::I32 => BasicMetadataTypeEnum::IntType(self.llvm_context.i32_type()),
            ConcreteType::U8 => BasicMetadataTypeEnum::IntType(self.llvm_context.i8_type()),
            ConcreteType::U32 => BasicMetadataTypeEnum::IntType(self.llvm_context.i32_type()),
            ConcreteType::U64 => BasicMetadataTypeEnum::IntType(self.llvm_context.i64_type()),
            ConcreteType::I64 => BasicMetadataTypeEnum::IntType(self.llvm_context.i64_type()),
            ConcreteType::Ptr(_inner) => BasicMetadataTypeEnum::PointerType(
                // LLVM 15+ uses opaque pointers, no distinction between pointer types
                self.llvm_context.ptr_type(AddressSpace::default()),
            ),
            ConcreteType::Bool => BasicMetadataTypeEnum::IntType(self.llvm_context.bool_type()),
            ConcreteType::Void => return None,
            ConcreteType::StructLike(ConcreteStructType {
                name,
                fields,
                non_generic_name: _,
            }) => {
                if let Some(t) = self.llvm_context.get_struct_type(name) {
                    return Some(t.into());
                }
                let struct_type = self.llvm_context.opaque_struct_type(name);
                let mut field_types = Vec::new();
                for (_field_name, field_ty) in fields {
                    if let Some(t) = self.type_to_basic_type_enum(field_ty) {
                        field_types.push(t);
                    } else {
                        return None;
                    }
                }
                struct_type.set_body(&field_types, false);
                struct_type.into()
            }
        })
    }
}
