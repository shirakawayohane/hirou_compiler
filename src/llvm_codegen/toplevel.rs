use super::error::CompileError;
use super::*;
use crate::ast::*;

impl LLVMCodegenerator<'_> {
    pub(super) fn gen_function_impl(
        &mut self,
        ctx: Rc<RefCell<Context>>,
        fn_reg: &FunctionRegistration,
        generic_args: Vec<ResolvedType>,
        arg_types: Vec<ResolvedType>,
    ) -> Result<&FunctionImpl, CompileError> {
        let fn_name = &fn_reg.function.decl.name;
        // TODO: int以外の型にも対応する
        let params = fn_reg
            .function
            .decl
            .params
            .iter()
            .map(|_| self.llvm_context.i32_type().into())
            .collect::<Vec<_>>();
        let fn_type = self.llvm_context.i32_type().fn_type(&params, true);
        let function_value = self.llvm_module.add_function(&fn_name, fn_type, None);
        let entry_basic_block = self
            .llvm_context
            .append_basic_block(function_value, "entry");
        self.llvm_builder.position_at_end(entry_basic_block);

        {
            // パラメーターをFunctionBodyにallocし、Contextにも登録する
            ctx.borrow().push_scope(Scope::new(ScopeKind::Function));
            if let Some(fn_reg_generic_args) = &fn_reg.function.decl.generic_args {
                if generic_args.len() > fn_reg_generic_args.len() {
                    return Err(CompileError::from_error_kind(
                        CompileErrorKind::TooManyGenericArgs {
                            fn_name: fn_reg.function.decl.name.clone(),
                            expected: fn_reg_generic_args.len() as u32,
                            actual: generic_args.len() as u32,
                        },
                    ));
                }
                if generic_args.len() < fn_reg_generic_args.len() {
                    return Err(CompileError::from_error_kind(
                        CompileErrorKind::TooFewGenericArgs {
                            fn_name: fn_reg.function.decl.name.clone(),
                            expected: fn_reg_generic_args.len() as u32,
                            actual: generic_args.len() as u32,
                        },
                    ));
                }
                for (i, resolved_generic_arg_ty) in generic_args.iter().enumerate() {
                    let reg_generic_arg = fn_reg_generic_args.get(i).unwrap().value.clone();
                    ctx.borrow().register_into_current_scope(
                        &reg_generic_arg.name,
                        Registration::Type(TypeRegistration {
                            ns: "".to_string(),
                            name: reg_generic_arg.name,
                            resolved_ty: resolved_generic_arg_ty.clone(),
                        }),
                    )
                }
            }
            // Set parameters in function body
            for (i, (loc_ty, name)) in fn_reg.function.decl.params.iter().enumerate() {
                let resolved_ty = self.resolve_type(ctx.clone(), &loc_ty.value)?;
                let parameter = function_value.get_nth_param(i as u32).unwrap();
                parameter.set_name(name.as_str());
                if let ResolvedType::Void = resolved_ty {
                    continue;
                } else {
                    let allocated_pointer =
                        self.llvm_builder.build_alloca(parameter.get_type(), &name);
                    self.llvm_builder.build_store(allocated_pointer, parameter);
                    ctx.borrow().register_into_current_scope(
                        name,
                        Registration::Variable(VariableRegistration {
                            ns: fn_reg.ns.clone(),
                            resolved_ty,
                            value: allocated_pointer,
                        }),
                    );
                }
            }

            ctx.borrow().pop_scope();
        }

        for statement in &fn_reg.function.body {
            self.gen_statement(ctx.clone(), &statement.value)?;
        }

        let return_type =
            self.resolve_type(ctx.clone(), &fn_reg.function.decl.return_type.value)?;
        let key = mangle_function_impl_name(&fn_reg.ns, &fn_name, &arg_types);
        ctx.borrow().function_impls.insert(
            key,
            FunctionImpl {
                function_value,
                return_type,
                arg_types,
            },
        );
        Ok(ctx.borrow().function_impls.get(&key).unwrap())
    }
    pub(super) fn gen_toplevel(
        &mut self,
        ctx: Rc<RefCell<Context>>,
        top: TopLevel,
    ) -> Result<(), CompileError> {
        match top {
            TopLevel::Function(func) => {
                self.register_function(ctx.clone(), func);
                Ok(())
            }
        }
    }
}
