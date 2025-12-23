mod error;
mod expression;
mod generics;
mod intrinsic;
mod statement;
mod ty;

use std::{cell::RefCell, collections::HashMap, ops::DerefMut, rc::Rc};

use crate::{
    ast,
    common::target::PointerSizedIntWidth,
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

pub struct ResolverContext {
    pub errors: Rc<RefCell<Vec<CompileError>>>,
    pub types: Rc<RefCell<TypeScopes>>,
    pub scopes: Rc<RefCell<VariableScopes>>,
    pub type_defs: Rc<RefCell<HashMap<String, ast::TypeDef>>>,
    pub function_by_name: Rc<RefCell<HashMap<String, ast::Function>>>,
    pub interface_by_name: Rc<RefCell<HashMap<String, ast::Interface>>>,
    pub impls_by_name: Rc<RefCell<HashMap<String, Vec<Implementation>>>>,
    pub resolved_functions: Rc<RefCell<HashMap<String, resolved_ast::Function>>>,
    pub ptr_sized_int_type: PointerSizedIntWidth,
}

impl ResolverContext {
    pub fn new(ptr_sized_int_type: PointerSizedIntWidth) -> Self {
        Self {
            errors: Default::default(),
            types: Rc::new(RefCell::new(TypeScopes::new())),
            scopes: Rc::new(RefCell::new(VariableScopes::new())),
            type_defs: Default::default(),
            function_by_name: Default::default(),
            resolved_functions: Default::default(),
            ptr_sized_int_type,
            interface_by_name: Default::default(),
            impls_by_name: Default::default(),
        }
    }
    pub fn is_64_bit(&self) -> bool {
        self.ptr_sized_int_type == PointerSizedIntWidth::SixtyFour
    }
}

pub(crate) fn mangle_fn_name(
    name: &str,
    arg_types: &[&ResolvedType],
    ret: &ResolvedType,
) -> String {
    let mut mangled_name = name.to_owned();
    mangled_name.push('(');
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
    mangled_name.push(')');
    mangled_name.push_str("->");
    mangled_name.push_str(&ret.to_string());
    mangled_name
}

#[derive(Debug, Clone)]
pub struct VariableScopes {
    scopes: Vec<HashMap<String, ResolvedType>>,
}

impl VariableScopes {
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

    fn get(&self, name: &str) -> Option<&ResolvedType> {
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

impl TypeScopes {
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

    pub fn add(&mut self, name: String, ty: ResolvedType) {
        self.scopes.last_mut().unwrap().insert(name, ty);
    }

    fn get(&self, name: &str) -> Option<&ResolvedType> {
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

// Don't use `return` in this macro
#[macro_export]
macro_rules! in_global_scope {
    ($scopes: expr, $block: block) => {{
        let mut stashed_scopes = Vec::new();
        while $scopes.borrow().len() > 1 {
            stashed_scopes.push($scopes.borrow_mut().pop());
        }
        let result = $block;
        while stashed_scopes.len() > 0 {
            $scopes.borrow_mut().push(stashed_scopes.pop().unwrap());
        }
        result
    }};
}

// Don't use `return` in this macro
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
fn resolve_function(
    context: &ResolverContext,
    current_fn: &ast::Function,
) -> Result<(), FaitalError> {
    // Register generic type parameters first (only if not already registered with concrete types)
    in_new_scope!(context.types, {
        if let Some(generic_args) = &current_fn.decl.generic_args {
            for generic_arg in generic_args {
                // Check if a concrete type is already registered (from call site)
                if context.types.borrow().get(&generic_arg.name).is_none() {
                    let restrictions = generic_arg
                        .restrictions
                        .iter()
                        .map(|r| match r {
                            ast::Restriction::Interface(name) => {
                                resolved_ast::Restriction::Interface(
                                    resolved_ast::InterfaceRestriction { name: name.clone() },
                                )
                            }
                        })
                        .collect();
                    context.types.borrow_mut().add(
                        generic_arg.name.clone(),
                        ResolvedType::Generics(resolved_ast::ResolvedGenericType {
                            name: generic_arg.name.clone(),
                            restrictions,
                        }),
                    );
                }
            }
        }

        let result_type = resolve_type(context, &current_fn.decl.return_type)?;
        in_new_scope!(context.scopes, {
            let mut resolved_args: Vec<resolved_ast::Argument> = Vec::new();
            for arg in &current_fn.decl.args {
                match arg {
                    Argument::VarArgs => {
                        resolved_args.push(resolved_ast::Argument::VarArgs);
                    }
                    Argument::SelfArg => {
                        // SelfArg is only valid in interface implementations
                        unreachable!("SelfArg is not allowed in regular functions")
                    }
                    Argument::Normal(arg_ty, arg_name) => {
                        let arg_type = resolve_type(context, arg_ty)?;
                        context
                            .scopes
                            .borrow_mut()
                            .add(arg_name.clone(), arg_type.clone());
                        resolved_args
                            .push(resolved_ast::Argument::Normal(arg_type, arg_name.clone()));
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

        if context.resolved_functions.borrow().contains_key(&name) {
            return Ok(());
        }

        let mut resolved_statements = Vec::new();
        for statement in &current_fn.body {
            resolved_statements.push(resolve_statement(context, statement)?);
        }
        // 必ずReturnするための特別な処理
        if !current_fn.decl.is_intrinsic {
            if resolved_statements.is_empty() {
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
                        if result_type == ResolvedType::Void {
                            resolved_statements.push(resolved_ast::Statement::Effect(
                                resolved_ast::Effect {
                                    expression: effect.expression.clone(),
                                },
                            ));
                            resolved_statements.push(resolved_ast::Statement::Return(
                                resolved_ast::Return { expression: None },
                            ));
                        } else {
                            resolved_statements.push(resolved_ast::Statement::Return(
                                resolved_ast::Return {
                                    expression: Some(effect.expression.clone()),
                                },
                            ));
                        }
                    }
                }
            }
        }

        if ResolvedType::Void != result_type && !current_fn.decl.is_intrinsic {
            let return_stmt = resolved_statements.last().unwrap();
            let actual_return_ty = match return_stmt {
                resolved_ast::Statement::Return(ret) => &ret.expression.as_ref().unwrap().ty,
                _ => unreachable!(),
            };
            if !result_type.can_insert(actual_return_ty) {
                context.errors.borrow_mut().push(CompileError::new(
                    current_fn.body.last().unwrap().range,
                    crate::resolver::error::CompileErrorKind::TypeMismatch {
                        expected: result_type.clone(),
                        actual: actual_return_ty.clone(),
                    },
                ));
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

        context
            .resolved_functions
            .borrow_mut()
            .insert(name, resolved_function);
        });
    });
    Ok(())
}

// Resolve an implementation as a function
pub(crate) fn resolve_implementation(
    context: &ResolverContext,
    implementation: &ast::Implementation,
    fn_name: &str,
) -> Result<(), FaitalError> {
    // Check if already resolved
    if context.resolved_functions.borrow().contains_key(fn_name) {
        return Ok(());
    }

    // Resolve the target type (e.g., i32)
    let target_ty = resolve_type(context, &implementation.decl.target_ty)?;

    // Get the interface to get return type
    let interface_name = &implementation.decl.name;
    let interface = context
        .interface_by_name
        .borrow()
        .get(interface_name)
        .cloned()
        .ok_or_else(|| {
            FaitalError(format!("Interface {} not found", interface_name))
        })?;

    let return_type = resolve_type(context, &interface.return_type)?;

    in_new_scope!(context.scopes, {
        // Register 'self' as the target type in scope
        context
            .scopes
            .borrow_mut()
            .add("self".to_string(), target_ty.clone());

        // Build resolved arguments (self)
        let resolved_args = vec![resolved_ast::Argument::Normal(target_ty, "self".to_string())];

        // Resolve body statements
        let mut resolved_statements = Vec::new();
        for statement in &implementation.body {
            resolved_statements.push(resolve_statement(context, statement)?);
        }

        // Add return statement if needed
        if resolved_statements.is_empty() {
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
                    if return_type == ResolvedType::Void {
                        resolved_statements.push(resolved_ast::Statement::Effect(effect));
                        resolved_statements.push(resolved_ast::Statement::Return(
                            resolved_ast::Return { expression: None },
                        ));
                    } else {
                        resolved_statements.push(resolved_ast::Statement::Return(
                            resolved_ast::Return {
                                expression: Some(effect.expression.clone()),
                            },
                        ));
                    }
                }
            }
        }

        let resolved_function = resolved_ast::Function {
            decl: resolved_ast::FunctionDecl {
                name: fn_name.to_string(),
                args: resolved_args,
                return_type,
            },
            body: resolved_statements,
        };

        context
            .resolved_functions
            .borrow_mut()
            .insert(fn_name.to_string(), resolved_function);
    });

    Ok(())
}

pub(crate) fn resolve_module(
    context: &ResolverContext,
    module: &crate::ast::Module,
    is_build_only: bool,
) -> Result<crate::resolved_ast::ResolvedModule, FaitalError> {
    context.scopes.borrow_mut().push_new();
    context.types.borrow_mut().push_new();
    // 組み込み関数の型を登録する
    {
        let mut function_by_name = context.function_by_name.borrow_mut();
        register_intrinsic_functions(&mut function_by_name);
    }
    register_intrinsic_types(context.types.borrow_mut().deref_mut());

    for toplevel in &module.toplevels {
        match &toplevel.value {
            // 関数を名前で引けるようにしておく
            TopLevel::Function(func) => {
                context
                    .function_by_name
                    .borrow_mut()
                    .insert(func.decl.name.clone(), func.clone());
            }
            // 型定義を名前で引けるようにしておく
            TopLevel::TypeDef(typedef) => {
                context
                    .type_defs
                    .borrow_mut()
                    .insert(typedef.name.clone(), typedef.clone());
            }
            TopLevel::Interface(interface) => {
                context
                    .interface_by_name
                    .borrow_mut()
                    .insert(interface.name.clone(), interface.clone());
            }
            TopLevel::Implemantation(_) => (),
        }
    }

    // TypeDefが登録された後、Implentationの登録に必要な型を解決する
    for toplevel in &module.toplevels {
        match &toplevel.value {
            TopLevel::Implemantation(implementation) => {
                let mut impls_by_name = context.impls_by_name.borrow_mut();
                let impl_by_target = impls_by_name
                    .entry(implementation.decl.name.clone())
                    .or_insert_with(Vec::new);

                if resolve_type(context, &implementation.decl.target_ty).is_ok() {
                    impl_by_target.push(implementation.clone());
                    continue;
                }

                match &implementation.decl.target_ty.value {
                    UnresolvedType::TypeRef(typeref) => {
                        if context.interface_by_name.borrow().contains_key(&typeref.name)
                        {
                            context.errors.borrow_mut().push(CompileError::new(
                                implementation.decl.target_ty.range,
                                crate::resolver::error::CompileErrorKind::NotImplemented {
                                    message: "".into(),
                                },
                            ));
                        } else {
                            context.errors.borrow_mut().push(CompileError::new(
                                implementation.decl.target_ty.range,
                                crate::resolver::error::CompileErrorKind::TypeNotFound {
                                    name: typeref.name.clone(),
                                },
                            ));
                        }
                    }
                    UnresolvedType::Ptr(_) => {
                        context.errors.borrow_mut().push(CompileError::new(
                            implementation.decl.target_ty.range,
                            crate::resolver::error::CompileErrorKind::ImplForPointerIsInvalid,
                        ));
                    }
                    UnresolvedType::Infer => {
                        context.errors.borrow_mut().push(CompileError::new(
                            implementation.decl.target_ty.range,
                            crate::resolver::error::CompileErrorKind::ImplForInferenceIsInvalid,
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    let function_by_name = context.function_by_name.borrow();
    let main_fn = function_by_name
        .get("main")
        .ok_or_else(|| FaitalError("No main function found".into()))?;

    let resolved_toplevels = RefCell::new(Vec::new());

    // main関数から辿れる関数を全て解決する
    resolve_function(&context, main_fn)?;

    for resolved_function in context.resolved_functions.borrow().values() {
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
                    resolve_function(&context, unresolved_function)?;
                    for resolved_function in context.resolved_functions.borrow().values() {
                        resolved_toplevels
                            .borrow_mut()
                            .push(resolved_ast::TopLevel::Function(resolved_function.clone()));
                    }
                }
                TopLevel::TypeDef(_) => {}
                // Implementations are resolved on-demand when interface calls are made
                // The resolved implementation is added as a Function to resolved_toplevels
                TopLevel::Implemantation(_) => {}
                // Interfaces don't need separate resolution - they're already in context.interface_by_name
                // and are used during interface call resolution
                TopLevel::Interface(_) => {}
            }
        }
    }

    Ok(resolved_ast::ResolvedModule {
        toplevels: resolved_toplevels.into_inner(),
    })
}
