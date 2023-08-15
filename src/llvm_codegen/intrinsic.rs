use super::*;
use crate::ast::*;

use inkwell::AddressSpace;
use once_cell::sync::Lazy;

const PRINTF_FUNCTION: &str = "printf";
const MALLOC_FUNCTION: &str = "__malloc";
const PRINTU8_FUNCTION: &str = "print-u8";
const PRINTU8_PTR_FUNCTION: &str = "print-u8-ptr";
const PRINTI32_FUNCTION: &str = "print-i32";
const PRINTU64_FUNCTION: &str = "print-u64";

pub const UNRESOLVED_VOID_TYPE: Lazy<UnresolvedType> = Lazy::new(|| {
    UnresolvedType::TypeRef(TypeRef {
        prefix: None,
        name: VOID_TYPE_NAME.to_string(),
        generic_args: None,
    })
});

pub const UNRESOLVED_U8_TYPE: Lazy<UnresolvedType> = Lazy::new(|| {
    UnresolvedType::TypeRef(TypeRef {
        prefix: None,
        name: U8_TYPE_NAME.to_string(),
        generic_args: None,
    })
});

pub const UNRESOLVED_U32_TYPE: Lazy<UnresolvedType> = Lazy::new(|| {
    UnresolvedType::TypeRef(TypeRef {
        prefix: None,
        name: U32_TYPE_NAME.to_string(),
        generic_args: None,
    })
});

pub const UNRESOLVED_I32_TYPE: Lazy<UnresolvedType> = Lazy::new(|| {
    UnresolvedType::TypeRef(TypeRef {
        prefix: None,
        name: I32_TYPE_NAME.to_string(),
        generic_args: None,
    })
});

pub const UNRESOLVED_I64_TYPE: Lazy<UnresolvedType> = Lazy::new(|| {
    UnresolvedType::TypeRef(TypeRef {
        prefix: None,
        name: I64_TYPE_NAME.to_string(),
        generic_args: None,
    })
});

pub const UNRESOLVED_U64_TYPE: Lazy<UnresolvedType> = Lazy::new(|| {
    UnresolvedType::TypeRef(TypeRef {
        prefix: None,
        name: U64_TYPE_NAME.to_string(),
        generic_args: None,
    })
});

pub const UNRESOLVED_USIZE_TYPE: Lazy<UnresolvedType> = Lazy::new(|| {
    UnresolvedType::TypeRef(TypeRef {
        prefix: None,
        name: USIZE_TYPE_NAME.to_string(),
        generic_args: None,
    })
});

impl LLVMCodegenerator<'_> {
    fn gen_printf(&self) {
        // printf
        let i32_type = self.llvm_context.i32_type();
        let i8_ptr_type = self
            .llvm_context
            .i8_type()
            .ptr_type(AddressSpace::default());
        let printf_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
        self.llvm_module
            .add_function(PRINTF_FUNCTION, printf_type, None);
    }
    fn gen_print_u8(&mut self, ctx: Rc<RefCell<Context>>) {
        // gen printi32 function
        let i8_type = self.i8_type;
        let void_type = self.llvm_context.void_type();
        let print_u8_type = void_type.fn_type(&[i8_type.into()], false);
        let print_u8_function =
            self.llvm_module
                .add_function("instrinsic_print_u8", print_u8_type, None);
        let entry_basic_block = self
            .llvm_context
            .append_basic_block(print_u8_function, "entry");
        self.llvm_builder.position_at_end(entry_basic_block);

        let digit_format_string_ptr = self
            .llvm_builder
            .build_global_string_ptr("%d\n", "digit_format_string");
        let argument = print_u8_function.get_first_param().unwrap();
        let printf_function = self.llvm_module.get_function(PRINTF_FUNCTION).unwrap();
        self.llvm_builder.build_call(
            printf_function,
            &[
                digit_format_string_ptr.as_pointer_value().into(),
                argument.into(),
            ],
            "call",
        );
        self.llvm_builder.build_return(None);

        self.register_function(
            ctx.clone(),
            Function {
                decl: FunctionDecl {
                    name: PRINTU8_FUNCTION.to_owned(),
                    generic_args: None,
                    params: vec![(
                        Located {
                            range: Range::default(),
                            value: UNRESOLVED_U8_TYPE.clone(),
                        },
                        "value".to_owned(),
                    )],
                    return_type: Located {
                        range: Range::default(),
                        value: UnresolvedType::TypeRef(TypeRef {
                            prefix: None,
                            name: "void".to_owned(),
                            generic_args: None,
                        }),
                    },
                },
                body: vec![Located {
                    range: Range::default(),
                    value: Statement::Effect {
                        expression: Located {
                            range: Range::default(),
                            value: Expression::CallExpr(CallExpr {
                                name: "instrinsic_print_u8".to_owned(),
                                args: vec![Located {
                                    range: Range::default(),
                                    value: Box::new(Expression::VariableRef {
                                        deref_count: 0,
                                        index_access: None,
                                        name: "value".to_owned(),
                                    }),
                                }],
                                generic_args: None,
                            }),
                        },
                    },
                }],
            },
        )
    }
    fn gen_malloc(&mut self, ctx: Rc<RefCell<Context>>) {
        let i64_type = self.llvm_context.i64_type();
        let i8_ptr_type = self
            .llvm_context
            .i8_type()
            .ptr_type(AddressSpace::default());
        let malloc_fn_type = i8_ptr_type.fn_type(
            &[match self.pointer_size {
                PointerSize::SixteenFour => i64_type.into(),
            }],
            false,
        );
        let builtin_malloc_function = self
            .llvm_module
            .add_function("malloc", malloc_fn_type, None);

        let wrapped_malloc_function =
            self.llvm_module
                .add_function(MALLOC_FUNCTION, malloc_fn_type, None);

        let entry_basic_block = self
            .llvm_context
            .append_basic_block(wrapped_malloc_function, "entry");
        self.llvm_builder.position_at_end(entry_basic_block);
        let argument = wrapped_malloc_function.get_first_param().unwrap();
        let pointer =
            self.llvm_builder
                .build_call(builtin_malloc_function, &[argument.into()], "call");
        self.llvm_builder
            .build_return(Some(&pointer.try_as_basic_value().left().unwrap()));

        self.register_function(
            ctx.clone(),
            Function {
                decl: FunctionDecl {
                    name: "__malloc".to_owned(),
                    generic_args: Some(vec![Located {
                        range: Range::default(),
                        value: GenericArgument {
                            name: "T".to_owned(),
                        },
                    }]),
                    params: vec![(
                        Located {
                            range: Range::default(),
                            value: UnresolvedType::TypeRef(TypeRef {
                                prefix: None,
                                name: "usize".to_owned(),
                                generic_args: None,
                            }),
                        },
                        "size".to_owned(),
                    )],
                    return_type: Located {
                        range: Range::default(),
                        value: UnresolvedType::Ptr(Box::new(UnresolvedType::TypeRef(TypeRef {
                            prefix: None,
                            name: "T".to_owned(),
                            generic_args: None,
                        }))),
                    },
                },
                body: vec![Located {
                    range: Range::default(),
                    value: Statement::Return {
                        expression: Some(Located {
                            range: Range::default(),
                            value: Expression::CallExpr(CallExpr {
                                name: MALLOC_FUNCTION.to_string(),
                                args: vec![Located {
                                    range: Range::default(),
                                    value: Box::new(Expression::VariableRef {
                                        deref_count: 0,
                                        index_access: None,
                                        name: "size".to_owned(),
                                    }),
                                }],
                                generic_args: None,
                            }),
                        }),
                    },
                }],
            },
        );
    }
    pub(super) fn gen_intrinsic_functions_on_llvm(&mut self, ctx: Rc<RefCell<Context>>) {
        self.gen_printf();
        self.gen_print_u8(ctx.clone());
        self.gen_malloc(ctx.clone());
    }
}
