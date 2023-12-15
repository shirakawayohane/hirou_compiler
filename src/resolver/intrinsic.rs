use std::collections::HashMap;

use nom::{
    error::{convert_error, VerboseError},
    Finish,
};

use crate::{
    ast::{self, Function},
    parser::parse_module,
};

const INTRINSIC_DECLS: &'static str = r#"
fn malloc<T>(size: usize, v: T) : *T {}
// fn printf<T>(format: *u8, v: T) : void {}
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
                function_by_name.insert(function_name, function);
            }
        }
    }
}
