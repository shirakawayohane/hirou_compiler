use inkwell::{
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum},
    AddressSpace,
};

use crate::resolved_ast::{ResolvedStructType, ResolvedType};

use super::LLVMCodeGenerator;

impl<'a> LLVMCodeGenerator<'a> {
    pub fn type_to_basic_type_enum(&self, ty: &ResolvedType) -> Option<BasicTypeEnum<'a>> {
        self.type_to_basic_metadata_type_enum(ty)
            .map(|x| x.try_into().unwrap())
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
            ResolvedType::Bool => BasicMetadataTypeEnum::IntType(self.llvm_context.bool_type()),
            ResolvedType::Void => return None,
            ResolvedType::Unknown => unimplemented!(),
            ResolvedType::Struct(ResolvedStructType {
                name,
                fields,
                generic_args: _,
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
