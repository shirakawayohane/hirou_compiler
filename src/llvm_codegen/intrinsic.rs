use super::*;

use inkwell::AddressSpace;

const PRINTF_FUNCTION: &str = "printf";
const MALLOC_FUNCTION: &str = "malloc";
const PRINTU8_FUNCTION: &str = "print-u8";
const PRINTU8_PTR_FUNCTION: &str = "print-u8-ptr";
const PRINTI32_FUNCTION: &str = "print-i32";
const PRINTU64_FUNCTION: &str = "print-u64";

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
    fn gen_print_u8(&self) {
        // gen printi32 function
        let i8_type = self.i8_type;
        let void_type = self.llvm_context.void_type();
        let print_u8_type = void_type.fn_type(&[i8_type.into()], false);
        let print_u8_function =
            self.llvm_module
                .add_function(PRINTU8_FUNCTION, print_u8_type, None);
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

        self.context.borrow_mut().set_function(
            PRINTU8_FUNCTION.to_string(),
            Type::Void,
            vec![Type::U8],
            print_u8_function,
        );
    }
    fn gen_print_i32(&self) {
        // gen printi32 function
        let i32_type = self.llvm_context.i32_type();
        let void_type = self.llvm_context.void_type();
        let print_i32_type = void_type.fn_type(&[i32_type.into()], false);
        let print_i32_function =
            self.llvm_module
                .add_function(PRINTI32_FUNCTION, print_i32_type, None);
        let entry_basic_block = self
            .llvm_context
            .append_basic_block(print_i32_function, "entry");
        self.llvm_builder.position_at_end(entry_basic_block);

        let digit_format_string_ptr = self
            .llvm_builder
            .build_global_string_ptr("%d\n", "digit_format_string");
        let argument = print_i32_function.get_first_param().unwrap();
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

        self.context.borrow_mut().set_function(
            PRINTI32_FUNCTION.to_string(),
            Type::Void,
            vec![Type::I32],
            print_i32_function,
        );
    }
    fn gen_print_u64(&self) {
        // gen printi32 function
        let i64_type = self.llvm_context.i64_type();
        let void_type = self.llvm_context.void_type();
        let print_u64_type = void_type.fn_type(&[i64_type.into()], false);
        let print_u64_function =
            self.llvm_module
                .add_function(PRINTU64_FUNCTION, print_u64_type, None);
        let entry_basic_block = self
            .llvm_context
            .append_basic_block(print_u64_function, "entry");
        self.llvm_builder.position_at_end(entry_basic_block);

        let digit_format_string_ptr = self
            .llvm_builder
            .build_global_string_ptr("%zu\n", "digit_format_string");
        let argument = print_u64_function.get_first_param().unwrap();
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

        self.context.borrow_mut().set_function(
            PRINTU64_FUNCTION.to_string(),
            Type::Void,
            vec![Type::U64],
            print_u64_function,
        )
    }
    fn gen_print_u8_ptr(&self) {
        // gen printi32 function
        let ptr_type = self
            .llvm_context
            .i8_type()
            .ptr_type(AddressSpace::default());
        let void_type = self.llvm_context.void_type();
        let print_ptr_type = void_type.fn_type(&[ptr_type.into()], false);
        let print_u8_ptr_function =
            self.llvm_module
                .add_function(PRINTU8_PTR_FUNCTION, print_ptr_type, None);
        let entry_basic_block = self
            .llvm_context
            .append_basic_block(print_u8_ptr_function, "entry");
        self.llvm_builder.position_at_end(entry_basic_block);

        let digit_format_string_ptr = self
            .llvm_builder
            .build_global_string_ptr("%zu\n", "digit_format_string");
        let argument = print_u8_ptr_function.get_first_param().unwrap();
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

        self.context.borrow_mut().set_function(
            PRINTU8_PTR_FUNCTION.to_string(),
            Type::Void,
            vec![Type::U8],
            print_u8_ptr_function,
        );
    }
    fn gen_malloc(&self) {
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
        let malloc_function = self
            .llvm_module
            .add_function(MALLOC_FUNCTION, malloc_fn_type, None);

        self.context.borrow_mut().set_function(
            "__malloc".to_string(),
            Type::Void,
            vec![Type::I32],
            malloc_function,
        );

        self.context.borrow_mut().set_function(
            MALLOC_FUNCTION.to_string(),
            Type::Ptr(Box::new(Type::U8)),
            vec![Type::USize],
            malloc_function,
        );
    }
    pub(super) fn gen_intrinsic_functions(&self) {
        self.gen_printf();
        self.gen_print_u8();
        self.gen_print_i32();
        self.gen_print_u64();
        self.gen_print_u8_ptr();
        self.gen_malloc();
    }
}
