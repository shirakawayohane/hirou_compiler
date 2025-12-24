use std::collections::HashMap;

use nom::{
    error::{convert_error, VerboseError},
    Finish,
};

use crate::{
    ast::{self, Function, Implementation, Interface, TypeDef},
    parser::parse_module,
};

/// Standard library definitions
/// These are parsed and registered before user code.
const STDLIB_DEFINITIONS: &str = r#"
// Vec<T> - Dynamic array type
struct Vec<T> {
    capacity: i32,
    size: i32,
    buf: *T,
}

// Vec operations as namespaced functions
// These will be accessible as Vec::new, Vec::get, etc.
fn Vec::new<T>(): Vec<T> {
    (:= alloc_size (sizeof T))
    (:= buf: *T (malloc alloc_size))
    Vec<T> {
        capacity: 1,
        size: 0,
        buf: buf
    }
}

fn Vec::get<T>(vec: Vec<T>, index: i32): T {
    (:= buf vec.buf)
    (:= idx : usize index)
    buf[idx]
}

fn Vec::len<T>(vec: Vec<T>): i32 {
    vec.size
}

fn Vec::capacity<T>(vec: Vec<T>): i32 {
    vec.capacity
}

fn Vec::is_empty<T>(vec: Vec<T>): bool {
    (= vec.size 0)
}

fn Vec::set<T>(vec: Vec<T>, index: i32, value: T): void {
    (:= buf vec.buf)
    (:= idx : usize index)
    (:=< buf[idx] value)
}

fn Vec::first<T>(vec: Vec<T>): T {
    (:= buf vec.buf)
    buf[0]
}

fn Vec::last<T>(vec: Vec<T>): T {
    (:= buf vec.buf)
    (:= last_idx_i32 (- vec.size 1))
    (:= last_idx : usize last_idx_i32)
    buf[last_idx]
}

fn Vec::free<T>(vec: Vec<T>): void {
    (free vec.buf)
}

fn Vec::alloc_and_copy<T>(old_buf: *T, old_size: i32, new_capacity: i32): *T {
    (:= elem_size (sizeof T))
    (:= alloc_size (* new_capacity elem_size))
    (:= new_buf : *T (malloc alloc_size))
    (:= copy_size (* old_size elem_size))
    (:= copy_size_usize : usize copy_size)
    (memcpy new_buf old_buf copy_size_usize)
    (free old_buf)
    new_buf
}

fn Vec::push<T>(vec: Vec<T>, value: T): Vec<T> {
    (:= old_capacity vec.capacity
        old_size vec.size
        old_buf vec.buf
        needs_grow (= old_size old_capacity)
        new_capacity (if needs_grow (* old_capacity 2) old_capacity)
        new_buf : *T (if needs_grow
            (Vec::alloc_and_copy<T> old_buf old_size new_capacity)
            old_buf
        )
        size_idx : usize old_size)
    (:=< new_buf[size_idx] value)
    (:= new_size (+ old_size 1)
        result Vec<T> {
            capacity: new_capacity,
            size: new_size,
            buf: new_buf
        })
    result
}

fn Vec::pop<T>(vec: Vec<T>): Vec<T> {
    Vec<T> {
        capacity: vec.capacity,
        size: (- vec.size 1),
        buf: vec.buf
    }
}
"#;

pub struct StdlibRegistration {
    pub type_defs: Vec<TypeDef>,
    pub interfaces: Vec<Interface>,
    pub implementations: Vec<Implementation>,
    pub functions: Vec<Function>,
}

pub fn parse_stdlib() -> Result<StdlibRegistration, String> {
    let result = parse_module(STDLIB_DEFINITIONS.into()).finish();
    if let Err(err) = result {
        let errors = err
            .errors
            .into_iter()
            .map(|(input, error)| (*input.fragment(), error))
            .collect();
        let error_message = convert_error(STDLIB_DEFINITIONS, VerboseError { errors });
        return Err(error_message);
    }

    let (_, module) = result.unwrap();

    let mut type_defs = Vec::new();
    let mut interfaces = Vec::new();
    let mut implementations = Vec::new();
    let mut functions = Vec::new();

    for toplevel in module.toplevels {
        match toplevel.value {
            ast::TopLevel::TypeDef(typedef) => {
                type_defs.push(typedef);
            }
            ast::TopLevel::Interface(interface) => {
                interfaces.push(interface);
            }
            ast::TopLevel::Implemantation(implementation) => {
                implementations.push(implementation);
            }
            ast::TopLevel::Function(function) => {
                functions.push(function);
            }
            ast::TopLevel::Use(_) => {
                // Use statements in stdlib are ignored - they're just for documentation
            }
        }
    }

    Ok(StdlibRegistration {
        type_defs,
        interfaces,
        implementations,
        functions,
    })
}

/// Register stdlib definitions into the resolver context
pub fn register_stdlib(
    type_defs: &mut HashMap<String, TypeDef>,
    interface_by_name: &mut HashMap<String, Interface>,
    impls_by_name: &mut HashMap<String, Vec<Implementation>>,
    function_by_name: &mut HashMap<String, Function>,
) {
    match parse_stdlib() {
        Ok(stdlib) => {
            for typedef in stdlib.type_defs {
                type_defs.insert(typedef.name.clone(), typedef);
            }
            for interface in stdlib.interfaces {
                interface_by_name.insert(interface.name.clone(), interface);
            }
            for implementation in stdlib.implementations {
                let impl_name = implementation.decl.name.clone();
                impls_by_name
                    .entry(impl_name)
                    .or_insert_with(Vec::new)
                    .push(implementation);
            }
            for function in stdlib.functions {
                function_by_name.insert(function.decl.name.clone(), function);
            }
        }
        Err(error_message) => {
            eprintln!("Failed to parse stdlib:\n{}", error_message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_stdlib() {
        let result = parse_stdlib();
        assert!(result.is_ok(), "Failed to parse stdlib: {:?}", result.err());
        let stdlib = result.unwrap();
        assert!(stdlib.type_defs.len() > 0, "No type definitions found");
        assert!(stdlib.functions.len() > 0, "No functions found");

        // Check Vec is defined
        assert!(stdlib.type_defs.iter().any(|t| t.name == "Vec"), "Vec type not found");

        // Check functions are defined
        let fn_names: Vec<&str> = stdlib.functions.iter().map(|f| f.decl.name.as_str()).collect();
        assert!(fn_names.contains(&"Vec::new"), "Vec::new function not found");
        assert!(fn_names.contains(&"Vec::get"), "Vec::get function not found");
        assert!(fn_names.contains(&"Vec::len"), "Vec::len function not found");
        assert!(fn_names.contains(&"Vec::push"), "Vec::push function not found");
        assert!(fn_names.contains(&"Vec::pop"), "Vec::pop function not found");
    }
}
