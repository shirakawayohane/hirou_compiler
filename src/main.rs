use std::fs::read_to_string;
mod ast;
mod codegen;
mod location;
mod parser;
use clap::{command, Parser};
use inkwell::context::Context as LLVMContext;
use nom::{
    error::{convert_error, VerboseError},
    Finish,
};

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(index = 1)]
    target: String,
    #[clap(short, long)]
    output: Option<String>,
}

fn main() {
    let args = Args::parse();
    let input = read_to_string(args.target).unwrap();
    let input = input.as_str().into();
    let module = match parser::parse_module(input).finish() {
        Ok((_, module)) => module,
        Err(err) => {
            // using workaround to convert Span -> &str
            // ref: https://github.com/fflorent/nom_locate/issues/36#issuecomment-1013469728
            let errors = err
                .errors
                .into_iter()
                .map(|(input, error)| (*input.fragment(), error))
                .collect();

            let error_message = convert_error(*input, VerboseError { errors });
            println!("{}", error_message);
            return;
        }
    };
    let llvm_context = LLVMContext::create();
    let llvm_codegenerator = codegen::LLVMCodegenerator::new(&llvm_context);
    if let Err(compile_error) = llvm_codegenerator.gen_module(module) {
        dbg!(compile_error);
    }
    if let Some(output) = args.output {
        llvm_codegenerator
            .llvm_module
            .print_to_file(output)
            .unwrap();
    }
    let execution_engine = llvm_codegenerator
        .llvm_module
        .create_jit_execution_engine(inkwell::OptimizationLevel::None)
        .unwrap();
    unsafe {
        execution_engine
            .get_function::<unsafe extern "C" fn()>("main")
            .unwrap()
            .call();
    }
}
