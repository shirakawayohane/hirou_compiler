use std::collections::HashMap;

use nom::{
    error::{convert_error, VerboseError},
    Finish,
};

use crate::{
    ast::{self, Function},
    parser::parse_module,
    resolved_ast::ResolvedType,
};

use super::Scopes;

const INTRINSIC_DECLS: &'static str = r#"
fn malloc<T>(size: usize, v: T) : *T {}
fn free<T>(ptr: *T) : void {}
fn memcpy<T>(dst: *T, src: *T, size: usize) : void {}
fn memset<T>(dst: *T, value: T, size: usize) : void {}
fn strlen(s: *u8) : usize {}
fn strcmp(s1: *u8, s2: *u8) : i32 {}
fn strcpy(dst: *u8, src: *u8) : *u8 {}
fn strcat(dst: *u8, src: *u8) : *u8 {}
fn printf(s: *u8, ...) : i32 {}
fn print-i32(s: *u8, n: i32): void {
    (printf 1, n)
}
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
    dbg!(&intrinsic_module);
    for toplevel in intrinsic_module.toplevels {
        match toplevel.value {
            ast::TopLevel::Function(function) => {
                let function_name = function.decl.name.clone();
                function_by_name.insert(function_name, function);
            }
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
const I32_TYPE: ResolvedType = ResolvedType::I32;
const I64_TYPE: ResolvedType = ResolvedType::I64;
const U32_TYPE: ResolvedType = ResolvedType::U32;
const U64_TYPE: ResolvedType = ResolvedType::U64;
const USIZE_TYPE: ResolvedType = ResolvedType::USize;
const U8_TYPE: ResolvedType = ResolvedType::U8;
const VOID_TYPE: ResolvedType = ResolvedType::Void;

pub(super) fn register_intrinsic_types(
    scopes: &mut Scopes,
    types: &mut HashMap<String, &ResolvedType>,
) {
    types.insert("i32".into(), &I32_TYPE);
    types.insert("i64".into(), &I64_TYPE);
    types.insert("u32".into(), &U32_TYPE);
    types.insert("u64".into(), &U64_TYPE);
    types.insert("usize".into(), &USIZE_TYPE);
    types.insert("u8".into(), &U8_TYPE);
    types.insert("void".into(), &VOID_TYPE);

    scopes.insert("i32".into(), I32_TYPE);
    scopes.insert("i64".into(), I64_TYPE);
    scopes.insert("u32".into(), U32_TYPE);
    scopes.insert("u64".into(), U64_TYPE);
    scopes.insert("usize".into(), USIZE_TYPE);
    scopes.insert("u8".into(), U8_TYPE);
    scopes.insert("void".into(), VOID_TYPE);
}
