use std::{fs::read_to_string, path::Path};
mod ast;
mod llvm_codegen;
mod parser;
mod util;
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

    let llvm_context: LLVMContext = LLVMContext::create();
    let mut llvm_codegenerator = llvm_codegen::LLVMCodegenerator::new(&llvm_context);
    match llvm_codegenerator.gen_module(module) {
        Ok(module) => {
            module.print_to_file(Path::new("out.ll")).unwrap();
            let execution_engine = &module
                .create_jit_execution_engine(inkwell::OptimizationLevel::None)
                .unwrap();
            unsafe {
                execution_engine
                    .get_function::<unsafe extern "C" fn()>("main")
                    .unwrap()
                    .call();
            }
        }
        Err(err) => {
            dbg!(err);
        }
    };
}
