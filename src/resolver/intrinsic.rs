use std::collections::HashMap;

use nom::{
    error::{convert_error, VerboseError},
    Finish,
};

use crate::{
    ast::{self, Function, FunctionDecl},
    parser::parse_module,
    resolved_ast::ResolvedType,
};

use super::TypeScopes;

const INTRINSIC_DECLS: &str = r#"
fn malloc(size: usize) : *void {}
fn free(ptr: *void) : *void {}
fn memcpy(dst: *void, src: *void, size: usize) : void {}
fn memset(dst: *void, value: *void, size: usize) : void {}
fn strlen(s: *u8) : usize {}
fn strcmp(s1: *u8, s2: *u8) : i32 {}
fn strcpy(dst: *u8, src: *u8) : *u8 {}
fn strcat(dst: *u8, src: *u8) : *u8 {}
fn printf(s: *u8, ...) : i32 {}
"#;

// 組み込み関数の定義を追加する
pub(super) fn register_intrinsic_functions(function_by_name: &mut HashMap<String, Function>) {
    let result = parse_module(INTRINSIC_DECLS.into()).finish();
    if let Err(err) = result {
        // using workaround to convert Span -> &str
        // ref: https://github.com/fflorent/nom_locate/issues/36#issuecomment-1013469728
        let errors = err
            .errors
            .into_iter()
            .map(|(input, error)| (*input.fragment(), error))
            .collect();

        let error_message = convert_error(INTRINSIC_DECLS, VerboseError { errors });
        println!("{}", error_message);
        return;
    }
    let (_, intrinsic_module) = result.unwrap();

    for toplevel in intrinsic_module.toplevels {
        match toplevel.value {
            ast::TopLevel::Function(function) => {
                let function_name = function.decl.name.clone();
                function_by_name.insert(
                    function_name,
                    Function {
                        decl: FunctionDecl {
                            is_intrinsic: true,
                            ..function.decl
                        },
                        body: function.body,
                    },
                );
            }
            ast::TopLevel::TypeDef(_) => {}
            ast::TopLevel::Implemantation(_) => unreachable!(),
            ast::TopLevel::Interface(_) => unreachable!(),
        }
    }
}

/*

I32,
I64,
U32,
U64,
USize,
U8,
Ptr(Box<ResolvedType>),
Void,
Unknown, */

pub(super) fn register_intrinsic_types(types: &mut TypeScopes) {
    types.add("i32".into(), ResolvedType::I32);
    types.add("i64".into(), ResolvedType::I64);
    types.add("u32".into(), ResolvedType::U32);
    types.add("u64".into(), ResolvedType::U64);
    types.add("usize".into(), ResolvedType::USize);
    types.add("u8".into(), ResolvedType::U8);
    types.add("bool".into(), ResolvedType::Bool);
    types.add("void".into(), ResolvedType::Void);
}
