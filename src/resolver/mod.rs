mod error;
mod expression;
mod intrinsic;
mod statement;
mod ty;

use std::{cell::RefCell, collections::HashMap, ops::DerefMut, rc::Rc};

use crate::{
    ast::{self},
    resolved_ast::{self, ResolvedType},
    resolver::ty::resolve_type,
};

use self::{
    error::{CompileError, FaitalError},
    intrinsic::{register_intrinsic_functions, register_intrinsic_types},
    statement::resolve_statement,
};

pub(crate) type Result<T, E = FaitalError> = std::result::Result<T, E>;

use crate::ast::*;

pub(crate) fn mangle_fn_name(
    name: &str,
    arg_types: &[&ResolvedType],
    ret: &ResolvedType,
) -> String {
    let mut mangled_name = name.to_owned();
    mangled_name.push_str("(");
    // for arg in arg_type_scopes {
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

#[derive(Debug, Clone)]
pub struct VariableScopes {
    scopes: Vec<HashMap<String, ResolvedType>>,
}

impl<'a> VariableScopes {
    fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    fn push_new(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn push(&mut self, scope: HashMap<String, ResolvedType>) {
        self.scopes.push(scope);
    }

    fn pop(&mut self) -> HashMap<String, ResolvedType> {
        self.scopes.pop().unwrap()
    }

    fn add(&mut self, name: String, ty: ResolvedType) {
        self.scopes.last_mut().unwrap().insert(name, ty);
    }

    fn get(&'a self, name: &str) -> Option<&ResolvedType> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }

    fn len(&self) -> usize {
        self.scopes.len()
    }
}

#[derive(Debug, Clone)]
pub struct TypeScopes {
    scopes: Vec<HashMap<String, ResolvedType>>,
}

impl<'a> TypeScopes {
    pub fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    fn push_new(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn push(&mut self, scope: HashMap<String, ResolvedType>) {
        self.scopes.push(scope);
    }

    fn pop(&mut self) -> HashMap<String, ResolvedType> {
        self.scopes.pop().unwrap()
    }

    fn add(&mut self, name: String, ty: ResolvedType) {
        self.scopes.last_mut().unwrap().insert(name, ty);
    }

    fn get(&'a self, name: &str) -> Option<&ResolvedType> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }

    fn len(&self) -> usize {
        self.scopes.len()
    }
}

#[macro_export]
macro_rules! in_global_scope {
    ($scopes: expr, $block: block) => {
        let mut stashed_scopes = Vec::new();
        while $scopes.borrow().len() > 1 {
            stashed_scopes.push($scopes.borrow_mut().pop());
        }
        $block
        while stashed_scopes.len() > 0 {
            $scopes.borrow_mut().push(stashed_scopes.pop().unwrap());
        }
    };
}

#[macro_export]
macro_rules! in_new_scope {
    ($scopes:expr, $block:block) => {{
        $scopes.borrow_mut().push_new();
        let result = $block;
        $scopes.borrow_mut().pop();
        result
    }};
}

// ジェネリック関数の場合は事前に型を登録しておく必要がある
fn resolve_function<'a>(
    errors: &mut Vec<CompileError>,
    type_scopes: Rc<RefCell<TypeScopes>>,
    scopes: Rc<RefCell<VariableScopes>>,
    type_defs: &HashMap<String, ast::TypeDef>,
    function_by_name: &HashMap<String, ast::Function>,
    resolved_functions: &mut HashMap<String, resolved_ast::Function>,
    current_fn: &ast::Function,
) -> Result<(), FaitalError> {
    let result_type = resolve_type(
        errors,
        type_scopes.borrow_mut().deref_mut(),
        type_defs,
        &current_fn.decl.return_type,
    )?;
    if !current_fn.decl.intrinsic && current_fn.body.len() == 0 {
        if result_type != ResolvedType::Void {
            errors.push(CompileError::from_error_kind(
                error::CompileErrorKind::ReturnTypeMismatch {
                    expected: result_type.to_string(),
                    actual: ResolvedType::Void.to_string(),
                },
            ));
        }
        return Ok(());
    }
    in_new_scope!(scopes, {
        let mut resolved_args: Vec<resolved_ast::Argument> = Vec::new();
        for arg in &current_fn.decl.args {
            match arg {
                Argument::VarArgs => {
                    resolved_args.push(resolved_ast::Argument::VarArgs);
                }
                Argument::Normal(arg_ty, arg_name) => {
                    let arg_type = resolve_type(
                        errors,
                        type_scopes.borrow_mut().deref_mut(),
                        type_defs,
                        &arg_ty,
                    )?;
                    scopes.borrow_mut().add(arg_name.clone(), arg_type.clone());
                    resolved_args.push(resolved_ast::Argument::Normal(arg_type, arg_name.clone()));
                }
            }
        }

        let name = if current_fn.decl.generic_args.is_some() {
            let arg_type_scopes = resolved_args
                .iter()
                .map(|x| match x {
                    resolved_ast::Argument::Normal(ty, _) => ty,
                    _ => panic!("unexpected argument type"),
                })
                .collect::<Vec<_>>();
            mangle_fn_name(&current_fn.decl.name, &arg_type_scopes, &result_type)
        } else {
            current_fn.decl.name.clone()
        };

        if resolved_functions.contains_key(&name) {
            return Ok(());
        }

        let mut resolved_statements = Vec::new();
        for statement in &current_fn.body {
            resolved_statements.push(resolve_statement(
                errors,
                type_scopes.clone(),
                scopes.clone(),
                type_defs,
                function_by_name,
                resolved_functions,
                statement,
            )?);
        }
        // 必ずReturnするための特別な処理
        if !current_fn.decl.intrinsic {
            if resolved_statements.len() == 0 {
                resolved_statements.push(resolved_ast::Statement::Return(resolved_ast::Return {
                    expression: None,
                }));
            } else {
                let last_stmt = resolved_statements.pop().unwrap();
                match last_stmt {
                    resolved_ast::Statement::Return(_) => {
                        resolved_statements.push(last_stmt);
                    }
                    resolved_ast::Statement::Effect(effect) => {
                        resolved_statements.push(resolved_ast::Statement::Return(
                            resolved_ast::Return {
                                expression: if result_type == ResolvedType::Void {
                                    None
                                } else {
                                    Some(effect.expression.clone())
                                },
                            },
                        ));
                    }
                    _ => {
                        resolved_statements.push(last_stmt);
                        resolved_statements.push(resolved_ast::Statement::Return(
                            resolved_ast::Return { expression: None },
                        ));
                    }
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
    });
    Ok(())
}

pub(crate) fn resolve_module(
    errors: &mut Vec<CompileError>,
    resolved_functions: &mut HashMap<String, resolved_ast::Function>,
    module: &crate::ast::Module,
    is_build_only: bool,
) -> Result<crate::resolved_ast::Module, FaitalError> {
    let mut function_by_name = HashMap::new();
    let mut type_defs = HashMap::new();
    let scopes = Rc::new(RefCell::new(VariableScopes::new()));
    let type_scopes = Rc::new(RefCell::new(TypeScopes::new()));
    scopes.borrow_mut().push_new();
    type_scopes.borrow_mut().push_new();
    // 組み込み関数の型を登録する
    register_intrinsic_functions(&mut function_by_name);
    register_intrinsic_types(type_scopes.borrow_mut().deref_mut());

    for toplevel in &module.toplevels {
        match &toplevel.value {
            // 関数を名前で引けるようにしておく
            TopLevel::Function(func) => {
                function_by_name.insert(func.decl.name.clone(), func.clone());
            }
            // 型定義を名前で引けるようにしておく
            TopLevel::TypeDef(typedef) => {
                type_defs.insert(typedef.name.clone(), typedef.clone());
            }
        }
    }
    let main_fn = function_by_name
        .get("main")
        .ok_or_else(|| FaitalError("No main function found".into()))?;

    let resolved_toplevels = RefCell::new(Vec::new());

    // main関数から辿れる関数を全て解決する
    resolve_function(
        errors,
        type_scopes.clone(),
        scopes.clone(),
        &type_defs,
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
            match &toplevel.value {
                TopLevel::Function(unresolved_function) => {
                    if unresolved_function.decl.generic_args.is_some() {
                        // ジェネリック関数はmain関数から辿れる関数の中で解決される
                        // TODO: この部分で出来ない解析は別の場所で行う
                        continue;
                    }
                    resolve_function(
                        errors,
                        type_scopes.clone(),
                        scopes.clone(),
                        &type_defs,
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
                TopLevel::TypeDef(_) => {}
            }
        }
    }

    Ok(resolved_ast::Module {
        toplevels: resolved_toplevels.into_inner(),
    })
}
