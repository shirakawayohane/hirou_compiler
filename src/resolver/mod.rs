mod error;
mod intrinsic;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    ast::{self},
    resolved_ast::{self, ResolvedExpression, ResolvedType, VariableRefExpr},
};

use self::{
    error::{CompileError, FaitalError},
    intrinsic::register_intrinsic_functions,
};

type Result<T, E = FaitalError> = std::result::Result<T, E>;

use crate::ast::*;

fn mangle_fn_name(name: &str, arg_types: &[&ResolvedType], ret: &ResolvedType) -> String {
    let mut mangled_name = name.to_owned();
    mangled_name.push_str("(");
    for arg in arg_types {
        mangled_name.push_str(&arg.to_string());
        mangled_name.push_str(",");
    }
    mangled_name.push_str(")");
    mangled_name.push_str("->");
    mangled_name.push_str(&ret.to_string());
    mangled_name
}

fn resolve_type<'a>(
    errors: &mut Vec<CompileError>,
    types: Rc<RefCell<HashMap<String, &'a ResolvedType>>>,
    ty: &ast::UnresolvedType,
) -> Result<ResolvedType> {
    match ty {
        UnresolvedType::TypeRef(typ_ref) => {
            let resolved_type = *types.borrow().get(&typ_ref.name).unwrap_or({
                errors.push(CompileError::from_error_kind(
                    error::CompileErrorKind::TypeNotFound {
                        name: typ_ref.name.clone(),
                    },
                ));
                &&ResolvedType::Unknown
            });
            Ok(resolved_type.clone())
        }
        UnresolvedType::Ptr(inner_type) => {
            let inner_type: ResolvedType = resolve_type(errors, types.clone(), inner_type)?;
            Ok(ResolvedType::Ptr(Box::new(inner_type)))
        }
    }
}

struct Scopes {
    scopes: Vec<HashMap<String, ResolvedType>>,
}

impl<'a> Scopes {
    fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn insert(&mut self, name: String, ty: ResolvedType) {
        self.scopes.last_mut().unwrap().insert(name, ty);
    }

    fn get(&'a self, name: &str) -> Option<&'a ResolvedType> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }
}

fn gen_function_impls_recursively<'a>(
    errors: &mut Vec<CompileError>,
    types: Rc<RefCell<HashMap<String, &'a ResolvedType>>>,
    scopes: Rc<RefCell<Scopes>>,
    function_by_name: &HashMap<String, ast::Function>,
    resolved_functions: Rc<RefCell<Vec<resolved_ast::Function>>>,
    current_fn: &ast::Function,
) -> Result<(), FaitalError> {
    fn gen_function_impls_from_expression_recursively<'a>(
        errors: &mut Vec<CompileError>,
        types: Rc<RefCell<HashMap<String, &'a ResolvedType>>>,
        scopes: Rc<RefCell<Scopes>>,
        function_by_name: &HashMap<String, ast::Function>,
        resolved_functions: Rc<RefCell<Vec<resolved_ast::Function>>>,
        expr: &ast::Expression,
        annotation: Option<ResolvedType>,
    ) -> Result<resolved_ast::ResolvedExpression, FaitalError> {
        match expr {
            Expression::VariableRef(variable_ref) => {
                let expr_kind = resolved_ast::ExpressionKind::VariableRef(VariableRefExpr {
                    name: variable_ref.name.clone(),
                });

                if let Some(ty) = scopes.borrow().get(&variable_ref.name) {
                    let resolved_type = if let Some(annotation) = annotation {
                        annotation
                    } else {
                        ty.clone()
                    };

                    return Ok(ResolvedExpression {
                        ty: resolved_type,
                        kind: expr_kind,
                    });
                } else {
                    return Ok(ResolvedExpression {
                        ty: ResolvedType::I32,
                        kind: expr_kind,
                    });
                }
            }
            Expression::NumberLiteral(number_literal) => {
                let kind =
                    resolved_ast::ExpressionKind::NumberLiteral(resolved_ast::NumberLiteral {
                        value: number_literal.value.clone(),
                        annotation: None,
                    });
                let ty = if let Some(annotation) = annotation {
                    annotation
                } else {
                    if number_literal.value.parse::<i32>().is_ok() {
                        ResolvedType::I32
                    } else if number_literal.value.parse::<i64>().is_ok() {
                        ResolvedType::I64
                    } else if number_literal.value.parse::<u64>().is_ok() {
                        ResolvedType::U64
                    } else {
                        unreachable!()
                    }
                };

                return Ok(ResolvedExpression { ty, kind });
            }
            Expression::BinaryExpr(bin_expr) => {
                let lhs = gen_function_impls_from_expression_recursively(
                    errors,
                    types.clone(),
                    scopes.clone(),
                    function_by_name,
                    resolved_functions.clone(),
                    &bin_expr.lhs,
                    None,
                )?;
                let rhs = gen_function_impls_from_expression_recursively(
                    errors,
                    types.clone(),
                    scopes.clone(),
                    function_by_name,
                    resolved_functions.clone(),
                    &bin_expr.rhs,
                    None,
                )?;
                return Ok(resolved_ast::ResolvedExpression {
                    kind: resolved_ast::ExpressionKind::BinaryExpr(resolved_ast::BinaryExpr {
                        op: bin_expr.op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }),
                    ty: ResolvedType::I32,
                });
            }
            Expression::Call(call_expr) => {
                let callee = function_by_name
                    .get(&call_expr.name)
                    .ok_or_else(|| FaitalError(format!("No function named {}", call_expr.name)))?;
                gen_function_impls_recursively(
                    errors,
                    types.clone(),
                    scopes.clone(),
                    function_by_name,
                    resolved_functions.clone(),
                    callee,
                )?;

                let resolved_return_ty =
                    resolve_type(errors, types.clone(), &callee.decl.return_type.value)?;
                let mut resolved_args = Vec::new();
                for arg in &call_expr.args {
                    resolved_args.push(gen_function_impls_from_expression_recursively(
                        errors,
                        types.clone(),
                        scopes.clone(),
                        function_by_name,
                        resolved_functions.clone(),
                        arg,
                        annotation.clone(),
                    )?);
                }
                return Ok(resolved_ast::ResolvedExpression {
                    kind: resolved_ast::ExpressionKind::CallExpr(resolved_ast::CallExpr {
                        name: call_expr.name.clone(),
                        args: resolved_args,
                    }),
                    ty: resolved_return_ty,
                });
            }
            Expression::DerefExpr(_) => todo!(),
            Expression::IndexAccess(_) => todo!(),
            Expression::StringLiteral(str_literal) => {
                return Ok(resolved_ast::ResolvedExpression {
                    kind: resolved_ast::ExpressionKind::StringLiteral(
                        resolved_ast::StringLiteral {
                            value: str_literal.value.clone(),
                        },
                    ),
                    ty: ResolvedType::Ptr(Box::new(ResolvedType::Ptr(Box::new(ResolvedType::U8)))),
                });
            }
        };
    }

    let mut resolved_args: Vec<(ResolvedType, String)> = Vec::new();
    for i in 0..current_fn.decl.args.len() {
        let (arg_ty, arg_name) = &current_fn.decl.args[i];
        let arg_type = resolve_type(errors, types.clone(), &arg_ty)?;
        resolved_args.push((arg_type, arg_name.clone()));
    }

    let result_type = resolve_type(errors, types.clone(), &current_fn.decl.return_type)?;

    let name = if current_fn.decl.generic_args.is_some() {
        let arg_types = resolved_args.iter().map(|x| &x.0).collect::<Vec<_>>();
        mangle_fn_name(&current_fn.decl.name, &arg_types, &result_type)
    } else {
        current_fn.decl.name.clone()
    };

    let mut resolved_statements = Vec::new();
    for statement in &current_fn.body {
        match &statement.value {
            Statement::VariableDecl(decl) => {
                let annotation = Some(resolve_type(errors, types.clone(), &decl.ty)?);
                let resolved_expr = gen_function_impls_from_expression_recursively(
                    errors,
                    types.clone(),
                    scopes.clone(),
                    function_by_name,
                    resolved_functions.clone(),
                    &decl.value,
                    annotation,
                )?;
                resolved_statements.push(resolved_ast::Statement::VariableDecl(
                    resolved_ast::VariableDecl {
                        name: decl.name.clone(),
                        value: resolved_expr,
                    },
                ));
            }
            Statement::Return(ret) => {
                if let Some(expr) = &ret.expression {
                    gen_function_impls_from_expression_recursively(
                        errors,
                        types.clone(),
                        scopes.clone(),
                        function_by_name,
                        resolved_functions.clone(),
                        expr,
                        None,
                    )?;
                }
            }
            Statement::Effect(effect) => {
                gen_function_impls_from_expression_recursively(
                    errors,
                    types.clone(),
                    scopes.clone(),
                    function_by_name,
                    resolved_functions.clone(),
                    &effect.expression,
                    None,
                )?;
            }
            Statement::Assignment(assignment) => {
                gen_function_impls_from_expression_recursively(
                    errors,
                    types.clone(),
                    scopes.clone(),
                    function_by_name,
                    resolved_functions.clone(),
                    &assignment.expression,
                    None,
                )?;
            }
        }
    }

    resolved_functions
        .borrow_mut()
        .push(resolved_ast::Function {
            decl: resolved_ast::FunctionDecl {
                name,
                args: resolved_args,
                return_type: result_type,
            },
            body: resolved_statements,
        });

    Ok(())
}

pub(crate) fn resolve_module<'a>(
    errors: &mut Vec<CompileError>,
    types: Rc<RefCell<HashMap<String, &'a ResolvedType>>>,
    module: &'a crate::ast::Module,
) -> Result<crate::resolved_ast::Module, FaitalError> {
    let mut function_by_name = HashMap::new();
    // 組み込み関数の型を登録する
    register_intrinsic_functions(&mut function_by_name);
    dbg!(&function_by_name);
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
    let resolved_functions = Rc::new(RefCell::new(Vec::new()));
    let scopes = Rc::new(RefCell::new(Scopes::new()));

    // main関数から辿れる関数を全て解決する
    gen_function_impls_recursively(
        errors,
        types.clone(),
        scopes.clone(),
        &function_by_name,
        resolved_functions.clone(),
        main_fn,
    )?;
    for resolved_function in resolved_functions.borrow().iter() {
        resolved_toplevels
            .borrow_mut()
            .push(resolved_ast::TopLevel::Function(resolved_function.clone()));
    }

    // 以下はmain関数から辿れない関数を解決する
    for toplevel in &module.toplevels {
        scopes.borrow_mut().push_scope();
        match &toplevel.value {
            TopLevel::Function(unresolved_function) => {
                for (arg_ty, arg_name) in &unresolved_function.decl.args {
                    let arg_type = resolve_type(errors, types.clone(), &arg_ty)?;
                    scopes
                        .borrow_mut()
                        .insert(arg_name.clone(), arg_type.clone());
                }
                gen_function_impls_recursively(
                    errors,
                    types.clone(),
                    scopes.clone(),
                    &function_by_name,
                    resolved_functions.clone(),
                    unresolved_function,
                )?;
                for resolved_function in resolved_functions.borrow().iter() {
                    resolved_toplevels
                        .borrow_mut()
                        .push(resolved_ast::TopLevel::Function(resolved_function.clone()));
                }
                resolved_functions.borrow_mut().clear();
            }
        }
        scopes.borrow_mut().pop_scope();
    }

    Ok(resolved_ast::Module {
        toplevels: resolved_toplevels.into_inner(),
    })
}
