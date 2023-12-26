mod error;
mod expression;
mod intrinsic;

use std::{cell::RefCell, collections::HashMap, ops::DerefMut, rc::Rc, thread::scope};

use crate::{
    ast::{self},
    resolved_ast::{self, ResolvedType},
};

use self::{
    error::{CompileError, FaitalError},
    expression::resolve_expression,
    intrinsic::{register_intrinsic_functions, register_intrinsic_types},
};

type Result<T, E = FaitalError> = std::result::Result<T, E>;

use crate::ast::*;

pub(crate) fn mangle_fn_name(
    name: &str,
    arg_types: &[&ResolvedType],
    ret: &ResolvedType,
) -> String {
    let mut mangled_name = name.to_owned();
    mangled_name.push_str("(");
    // for arg in arg_types {
    //     mangled_name.push_str(&arg.to_string());
    //     mangled_name.push_str(",");
    // }
    mangled_name.push_str(
        &arg_types
            .iter()
            .map(|arg| arg.to_string())
            .collect::<Vec<_>>()
            .join(","),
    );
    mangled_name.push_str(")");
    mangled_name.push_str("->");
    mangled_name.push_str(&ret.to_string());
    mangled_name
}

fn resolve_type<'a>(
    errors: &mut Vec<CompileError>,
    types: Rc<RefCell<HashMap<String, &ResolvedType>>>,
    ty: &ast::UnresolvedType,
) -> Result<ResolvedType> {
    match ty {
        UnresolvedType::TypeRef(typ_ref) => {
            let resolved_type = *types.borrow_mut().get(&typ_ref.name).unwrap_or_else(|| {
                errors.push(CompileError::from_error_kind(
                    error::CompileErrorKind::TypeNotFound {
                        name: typ_ref.name.clone(),
                    },
                ));
                &&ResolvedType::Unknown
            });
            // let resolved_type = *types.borrow().get(&typ_ref.name).unwrap();
            Ok(resolved_type.clone())
        }
        UnresolvedType::Ptr(inner_type) => {
            let inner_type: ResolvedType = resolve_type(errors, types.clone(), inner_type)?;
            Ok(ResolvedType::Ptr(Box::new(inner_type)))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scopes {
    variable_scopes: Vec<HashMap<String, ResolvedType>>,
}

impl<'a> Scopes {
    fn new() -> Self {
        Self {
            variable_scopes: Vec::new(),
        }
    }

    fn push_scope(&mut self) {
        self.variable_scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.variable_scopes.pop();
    }

    fn insert(&mut self, name: String, ty: ResolvedType) {
        self.variable_scopes.last_mut().unwrap().insert(name, ty);
    }

    fn get(&'a self, name: &str) -> Option<&ResolvedType> {
        for scope in self.variable_scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }
}

// ジェネリック関数の場合は事前に型を登録しておく必要がある
fn gen_function_impls_recursively<'a>(
    errors: &mut Vec<CompileError>,
    types: Rc<RefCell<HashMap<String, &'a ResolvedType>>>,
    scopes: Rc<RefCell<Scopes>>,
    function_by_name: &HashMap<String, ast::Function>,
    resolved_functions: &mut HashMap<String, resolved_ast::Function>,
    current_fn: &ast::Function,
) -> Result<(), FaitalError> {
    let mut resolved_args: Vec<resolved_ast::Argument> = Vec::new();
    for arg in &current_fn.decl.args {
        match arg {
            Argument::VarArgs => {
                resolved_args.push(resolved_ast::Argument::VarArgs);
            }
            Argument::Normal(arg_ty, arg_name) => {
                let arg_type = resolve_type(errors, types.clone(), &arg_ty)?;
                resolved_args.push(resolved_ast::Argument::Normal(arg_type, arg_name.clone()));
            }
        }
    }

    let result_type = resolve_type(errors, types.clone(), &current_fn.decl.return_type)?;

    let name = if current_fn.decl.generic_args.is_some() {
        let arg_types = resolved_args
            .iter()
            .map(|x| match x {
                resolved_ast::Argument::Normal(ty, _) => ty,
                _ => panic!("unexpected argument type"),
            })
            .collect::<Vec<_>>();
        mangle_fn_name(&current_fn.decl.name, &arg_types, &result_type)
    } else {
        current_fn.decl.name.clone()
    };

    if resolved_functions.contains_key(&name) {
        return Ok(());
    }

    let mut resolved_statements = Vec::new();
    for statement in &current_fn.body {
        match &statement.value {
            Statement::VariableDecl(decl) => {
                dbg!(decl);
                let annotation = Some(resolve_type(errors, types.clone(), &decl.ty)?);
                dbg!(annotation.clone());
                let resolved_expr = resolve_expression(
                    errors,
                    types.clone(),
                    scopes.clone(),
                    function_by_name,
                    resolved_functions,
                    &decl.value,
                    annotation,
                )?;
                scopes
                    .borrow_mut()
                    .insert(decl.name.clone(), resolved_expr.ty.clone());
                resolved_statements.push(resolved_ast::Statement::VariableDecl(
                    resolved_ast::VariableDecl {
                        name: decl.name.clone(),
                        value: resolved_expr,
                    },
                ));
            }
            Statement::Return(ret) => {
                if let Some(expr) = &ret.expression {
                    resolve_expression(
                        errors,
                        types.clone(),
                        scopes.clone(),
                        function_by_name,
                        resolved_functions,
                        expr,
                        None,
                    )?;
                }
            }
            Statement::Effect(effect) => {
                resolve_expression(
                    errors,
                    types.clone(),
                    scopes.clone(),
                    function_by_name,
                    resolved_functions,
                    &effect.expression,
                    None,
                )?;
            }
            Statement::Assignment(assignment) => {
                resolve_expression(
                    errors,
                    types.clone(),
                    scopes.clone(),
                    function_by_name,
                    resolved_functions,
                    &assignment.expression,
                    None,
                )?;
            }
        }
    }

    let resolved_function = resolved_ast::Function {
        decl: resolved_ast::FunctionDecl {
            name: name.clone(),
            args: resolved_args,
            return_type: result_type,
        },
        body: resolved_statements,
    };

    resolved_functions.insert(name, resolved_function);

    Ok(())
}

pub(crate) fn resolve_module(
    errors: &mut Vec<CompileError>,
    resolved_functions: &mut HashMap<String, resolved_ast::Function>,
    types: Rc<RefCell<HashMap<String, &ResolvedType>>>,
    module: &crate::ast::Module,
    is_build_only: bool,
) -> Result<crate::resolved_ast::Module, FaitalError> {
    let mut function_by_name = HashMap::new();
    let scopes = Rc::new(RefCell::new(Scopes::new()));
    scopes.borrow_mut().push_scope();
    // 組み込み関数の型を登録する
    register_intrinsic_functions(&mut function_by_name);
    register_intrinsic_types(
        scopes.borrow_mut().deref_mut(),
        types.borrow_mut().deref_mut(),
    );
    for toplevel in &module.toplevels {
        match &toplevel.value {
            TopLevel::Function(func) => {
                function_by_name.insert(func.decl.name.clone(), func.clone());
            }
        }
    }
    let main_fn = function_by_name
        .get("main")
        .ok_or_else(|| FaitalError("No main function found".into()))?;

    let resolved_toplevels = RefCell::new(Vec::new());

    // main関数から辿れる関数を全て解決する
    gen_function_impls_recursively(
        errors,
        types.clone(),
        scopes.clone(),
        &function_by_name,
        resolved_functions,
        main_fn,
    )?;

    for resolved_function in resolved_functions.values() {
        resolved_toplevels
            .borrow_mut()
            .push(resolved_ast::TopLevel::Function(resolved_function.clone()));
    }

    if !is_build_only {
        // 以下はmain関数から辿れない関数を解決する
        for toplevel in &module.toplevels {
            scopes.borrow_mut().push_scope();
            match &toplevel.value {
                TopLevel::Function(unresolved_function) => {
                    gen_function_impls_recursively(
                        errors,
                        types.clone(),
                        scopes.clone(),
                        &function_by_name,
                        resolved_functions,
                        unresolved_function,
                    )?;
                    for resolved_function in resolved_functions.values() {
                        resolved_toplevels
                            .borrow_mut()
                            .push(resolved_ast::TopLevel::Function(resolved_function.clone()));
                    }
                }
            }
            scopes.borrow_mut().pop_scope();
        }
    }

    scopes.borrow_mut().pop_scope();

    Ok(resolved_ast::Module {
        toplevels: resolved_toplevels.into_inner(),
    })
}
