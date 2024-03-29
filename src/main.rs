use std::{fs::read_to_string, path::Path};
mod ast;
mod builder;
mod common;
mod concrete_ast;
mod concretizer;
mod parser;
mod resolved_ast;
mod resolver;

use clap::{command, Parser};
use common::target::{PointerSizedIntWidth, TargetPlatform};
use inkwell::{context::Context as LLVMContext, OptimizationLevel};
use nom::{
    error::{convert_error, VerboseError},
    Finish,
};
use resolver::ResolverContext;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(index = 1)]
    target: String,
    #[clap(short, long)]
    output: Option<String>,
    #[clap(long)]
    parse: bool,
}

fn main() {
    let args = Args::parse();
    let path = Path::new(&args.target);
    let input = read_to_string(path).unwrap();
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
    if args.parse {
        dbg!(module);
        return;
    }

    let llvm_context: LLVMContext = LLVMContext::create();
    let target_platform = TargetPlatform::DarwinArm64;
    let resolver_context = ResolverContext::new(PointerSizedIntWidth::from(target_platform));
    let resolved_module = match resolver::resolve_module(&resolver_context, &module, true) {
        Ok(module) => module,
        Err(err) => {
            dbg!(err);
            return;
        }
    };
    if !resolver_context.errors.borrow().is_empty() {
        let absolute_path = path.canonicalize().unwrap();
        let current_dir = std::env::current_dir().unwrap();
        let relative_path = absolute_path.strip_prefix(current_dir).unwrap();
        let mut stdout = std::io::stdout();
        for error in resolver_context.errors.borrow().iter() {
            error
                .fmt_with_source(
                    &mut stdout,
                    relative_path.to_str().unwrap(),
                    input.fragment(),
                )
                .unwrap();
        }
        return;
    }
    let concretizer_context =
        concretizer::ConcretizerContext::from_resolved_module(&resolver_context, resolved_module);
    let mut concrete_module = concretizer::concretize_module(&concretizer_context);
    let mut llvm_codegenerator = builder::LLVMCodeGenerator::new(
        &llvm_context,
        target_platform,
        OptimizationLevel::None,
        &concrete_module,
    );
    llvm_codegenerator.gen_module(&concrete_module);
    let module = llvm_codegenerator.get_module();

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
