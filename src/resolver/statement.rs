use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::rc::Rc;

use clap::error;

use crate::ast::{Located, Statement};
use crate::resolved_ast::{self, ExpressionKind, ResolvedExpression, ResolvedType};
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
    loc_statement: &Located<ast::Statement>,
) -> Result<resolved_ast::Statement, FaitalError> {
    Ok(match &loc_statement.value {
        Statement::VariableDecl(decl) => {
            in_new_scope!(type_scopes, {
                let resolved_annotation = decl
                    .ty
                    .clone()
                    .map(|unresolved_ty| {
                        resolve_type(
                            errors,
                            type_scopes.borrow_mut().deref_mut(),
                            type_defs,
                            &unresolved_ty,
                        )
                    })
                    .transpose()?;
                let resolved_expr = resolve_expression(
                    errors,
                    type_scopes.clone(),
                    scopes.clone(),
                    type_defs,
                    function_by_name,
                    resolved_functions,
                    decl.value.as_ref(),
                    resolved_annotation.as_ref(),
                )?;
                if let Some(resolved_annotation) = resolved_annotation {
                    if !resolved_annotation.can_insert(&resolved_expr.ty) {
                        errors.push(CompileError::new(
                            loc_statement.range,
                            CompileErrorKind::TypeMismatch {
                                expected: resolved_annotation.clone(),
                                actual: resolved_expr.ty.clone(),
                            },
                        ));
                        scopes
                            .borrow_mut()
                            .add(decl.name.clone(), resolved_expr.ty.clone());
                        return Ok(resolved_ast::Statement::VariableDecl(
                            resolved_ast::VariableDecl {
                                name: decl.name.clone(),
                                value: ResolvedExpression {
                                    ty: resolved_annotation.clone(),
                                    kind: ExpressionKind::Unknown,
                                },
                            },
                        ));
                    }
                }
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
                        expr.as_ref(),
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
                effect.expression.as_ref(),
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
                assignment.expression.as_ref(),
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
                            x.as_ref(),
                            Some(&ResolvedType::USize),
                        )
                    })
                    .transpose()?,
            })
        }
    })
}
