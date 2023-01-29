use super::*;

use inkwell::AddressSpace;

const PRINTF_FUNCTION: &str = "printf";
const MALLOC_FUNCTION: &str = "__malloc";
const PRINTU8_FUNCTION: &str = "print-u8";
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
    }
    fn gen_malloc(&self) {
        let i64_type = self.llvm_context.i64_type();
        let i8_ptr_type = self
            .llvm_context
            .i8_type()
            .ptr_type(AddressSpace::default());
        let malloc_fn_type = i8_ptr_type.fn_type(&[i64_type.into()], false);
        self.llvm_module
            .add_function(MALLOC_FUNCTION, malloc_fn_type, None);
    }
    pub(super) fn gen_intrinsic_functions(&self) {
        self.gen_printf();
        self.gen_print_u8();
        self.gen_print_i32();
        self.gen_print_u64();
        self.gen_malloc();
    }
}
