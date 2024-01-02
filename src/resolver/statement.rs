use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::rc::Rc;

use crate::ast::{Statement, StructTypeDef, UnresolvedType};
use crate::resolved_ast::{
    self, ExpressionKind, ResolvedExpression, ResolvedStructType, ResolvedType,
};
use crate::resolver::error::CompileErrorKind;
use crate::{ast, in_new_scope};

use super::error::{CompileError, FaitalError};
use super::expression::resolve_expression;
use super::ty::resolve_type;
use super::{TypeScopes, VariableScopes};

pub fn resolve_statement(
    errors: &mut Vec<CompileError>,
    type_scopes: Rc<RefCell<TypeScopes>>,
    scopes: Rc<RefCell<VariableScopes>>,
    type_defs: &HashMap<String, ast::TypeDef>,
    function_by_name: &HashMap<String, ast::Function>,
    resolved_functions: &mut HashMap<String, resolved_ast::Function>,
    statement: &ast::Statement,
) -> Result<resolved_ast::Statement, FaitalError> {
    Ok(match statement {
        Statement::VariableDecl(decl) => {
            in_new_scope!(type_scopes, {
                let annotation = if let Some(unresolved_annotation) = &decl.ty {
                    let resolved_annotation = resolve_type(
                        errors,
                        type_scopes.borrow_mut().deref_mut(),
                        type_defs,
                        unresolved_annotation,
                    )?;

                    match &decl.value.value {
                        ast::Expression::Call(call_expr) => {
                            if let Some(callee) = function_by_name.get(&call_expr.name) {
                                let return_ty = &callee.decl.return_type.value;
                                if let UnresolvedType::TypeRef(return_ty_type_ref) = return_ty {
                                    // annotationの型と関数定義の型が一致している場合、関数定義の方のジェネリクスを解決できるよう、annotationを参考にスコープに型を登録する。
                                    if return_ty_type_ref.name == resolved_annotation.get_name() {
                                        if let Some(type_def) =
                                            type_defs.get(&return_ty_type_ref.name)
                                        {
                                            match &type_def.kind {
                                                ast::TypeDefKind::Struct(StructTypeDef {
                                                    generic_args: Some(generic_args),
                                                    fields: _,
                                                }) => {
                                                    if let resolved_ast::ResolvedType::Struct(
                                                        ResolvedStructType {
                                                            name: _,
                                                            non_generic_name: _,
                                                            generic_args:
                                                                Some(resolved_ty_generic_args),
                                                            fields: _,
                                                        },
                                                    ) = &resolved_annotation
                                                    {
                                                        for (i, generic_arg) in
                                                            generic_args.iter().enumerate()
                                                        {
                                                            type_scopes.borrow_mut().add(
                                                                generic_arg.name.clone(),
                                                                resolved_ty_generic_args[i].clone(),
                                                            );
                                                        }
                                                    }
                                                }
                                                _ => todo!(),
                                            }
                                        }
                                    }
                                }
                            } else {
                                errors.push(CompileError::from_error_kind(
                                    CompileErrorKind::FunctionNotFound {
                                        name: call_expr.name.clone(),
                                    },
                                ));
                                return Ok(resolved_ast::Statement::VariableDecl(
                                    resolved_ast::VariableDecl {
                                        name: decl.name.clone(),
                                        value: ResolvedExpression {
                                            kind: ExpressionKind::Unknown,
                                            ty: ResolvedType::Unknown,
                                        },
                                    },
                                ));
                            }
                        }
                        // TODO: Call以外の式の場合も同様に型を解決する
                        _ => {}
                    }

                    Some(resolved_annotation)
                } else {
                    None
                };
                let resolved_expr = resolve_expression(
                    errors,
                    type_scopes.clone(),
                    scopes.clone(),
                    type_defs,
                    function_by_name,
                    resolved_functions,
                    &decl.value,
                    annotation.clone(),
                )?;
                scopes
                    .borrow_mut()
                    .add(decl.name.clone(), resolved_expr.ty.clone());
                resolved_ast::Statement::VariableDecl(resolved_ast::VariableDecl {
                    name: decl.name.clone(),
                    value: resolved_expr,
                })
            })
        }
        Statement::Return(ret) => {
            if let Some(expr) = &ret.expression {
                resolved_ast::Statement::Return(resolved_ast::Return {
                    expression: Some(resolve_expression(
                        errors,
                        type_scopes.clone(),
                        scopes.clone(),
                        type_defs,
                        function_by_name,
                        resolved_functions,
                        expr,
                        None,
                    )?),
                })
            } else {
                resolved_ast::Statement::Return(resolved_ast::Return { expression: None })
            }
        }
        Statement::Effect(effect) => resolved_ast::Statement::Effect(resolved_ast::Effect {
            expression: resolve_expression(
                errors,
                type_scopes.clone(),
                scopes.clone(),
                type_defs,
                function_by_name,
                resolved_functions,
                &effect.expression,
                None,
            )?,
        }),
        Statement::Assignment(assignment) => {
            let resolved_expr = resolve_expression(
                errors,
                type_scopes.clone(),
                scopes.clone(),
                type_defs,
                function_by_name,
                resolved_functions,
                &assignment.expression,
                None,
            )?;
            resolved_ast::Statement::Assignment(resolved_ast::Assignment {
                name: assignment.name.clone(),
                expression: resolved_expr,
                deref_count: assignment.deref_count as usize,
                index_access: assignment
                    .index_access
                    .as_ref()
                    .map(|x| {
                        resolve_expression(
                            errors,
                            type_scopes.clone(),
                            scopes.clone(),
                            type_defs,
                            function_by_name,
                            resolved_functions,
                            x,
                            Some(resolved_ast::ResolvedType::USize),
                        )
                    })
                    .transpose()?,
            })
        }
    })
}
