use inkwell::{
    builder::BuilderError,
    types::{AnyType, BasicMetadataTypeEnum, BasicType},
    values::FunctionValue,
    AddressSpace,
};

use super::*;
use crate::concrete_ast::*;

impl<'a> LLVMCodeGenerator<'a> {
    pub(super) fn gen_or_get_function(&self, function: &Function) -> FunctionValue {
        if let Some(ret) = self.llvm_module.get_function(&function.decl.name) {
            return ret;
        }

        let returns_struct = match function.decl.return_type {
            ConcreteType::StructLike(_) => true,
            _ => false,
        };

        let mut has_var_args = false;
        let mut param_types: Vec<BasicMetadataTypeEnum> = Vec::new();
        // structを返す関数の場合は第一引数にポインタを追加する
        if returns_struct {
            let param = self
                .llvm_context
                .i8_type()
                .ptr_type(AddressSpace::default());
            param_types.push(param.into());
        }

        for arg in &function.decl.args {
            match arg {
                Argument::VarArgs => {
                    has_var_args = true;
                }
                Argument::Normal(ty, _) => {
                    if let Some(ty) = self.type_to_basic_metadata_type_enum(ty) {
                        param_types.push(ty);
                    }
                }
            }
        }

        let return_ty = self.type_to_basic_type_enum(&function.decl.return_type);
        let function = self.llvm_module.add_function(
            &function.decl.name,
            if let Some(return_ty) = return_ty {
                if returns_struct {
                    self.llvm_context
                        .void_type()
                        .fn_type(&param_types, has_var_args)
                } else {
                    return_ty.fn_type(&param_types, has_var_args)
                }
            } else {
                self.llvm_context
                    .void_type()
                    .fn_type(&param_types, has_var_args)
            },
            None,
        );

        if returns_struct {
            // noaliasをつける
            function.add_attribute(
                inkwell::attributes::AttributeLoc::Param(0),
                self.llvm_context.create_enum_attribute(
                    inkwell::attributes::Attribute::get_named_enum_kind_id("noalias"),
                    0,
                ),
            );
            // sretをつける
            function.add_attribute(
                inkwell::attributes::AttributeLoc::Param(0),
                self.llvm_context.create_type_attribute(
                    inkwell::attributes::Attribute::get_named_enum_kind_id("sret"),
                    return_ty.unwrap().as_any_type_enum(),
                ),
            );
        }

        function
    }

    pub(super) fn gen_function_body(&mut self, function: &'a Function) -> Result<(), BuilderError> {
        if function.body.is_empty() {
            return Ok(());
        }
        let returns_struct = match function.decl.return_type {
            ConcreteType::StructLike(_) => true,
            _ => false,
        };
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

            if returns_struct {
                let parameter = function_value.get_first_param().unwrap();
                parameter.set_name("sret_ptr");
            }

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
                let i = if returns_struct { i + 1 } else { i };
                let parameter = function_value.get_nth_param(i as u32).unwrap();
                parameter.set_name(name.as_str());
                let allocated_pointer = self
                    .llvm_builder
                    .build_alloca(parameter.get_type(), name)
                    .unwrap();
                self.llvm_builder
                    .build_store(allocated_pointer, parameter)
                    .unwrap();
                self.add_variable(name, allocated_pointer);
            }

            // Generate function body
            for (i, statement) in function.body.iter().enumerate() {
                if i == function.body.len() - 1 {
                    // 構造体を返す場合、最後のreturn文はreturn voidにする
                    if returns_struct {
                        match statement {
                            Statement::Return(return_stmt) => {
                                let value = self
                                    .gen_expression(return_stmt.expression.as_ref().unwrap())?
                                    .unwrap();
                                let first_param_ptr = function_value
                                    .get_first_param()
                                    .unwrap()
                                    .into_pointer_value();
                                let struct_ptr = value.into_pointer_value();
                                let struct_ty = self
                                    .type_to_basic_type_enum(
                                        &return_stmt.expression.as_ref().unwrap().ty,
                                    )
                                    .unwrap()
                                    .into_struct_type();
                                self.llvm_builder.build_memcpy(
                                    first_param_ptr,
                                    8,
                                    struct_ptr,
                                    8,
                                    struct_ty.size_of().unwrap(),
                                )?;
                                self.llvm_builder.build_return(None)?;
                                continue;
                            }
                            _ => {}
                        }
                    }
                }
                self.gen_statement(statement)?;
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
