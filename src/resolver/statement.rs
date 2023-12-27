use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::rc::Rc;

use crate::ast;
use crate::ast::Statement;
use crate::resolved_ast;

use super::error::{CompileError, FaitalError};
use super::expression::resolve_expression;
use super::{resolve_type, TypeScopes, VariableScopes};

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
            let annotation = Some(resolve_type(
                errors,
                type_scopes.borrow_mut().deref_mut(),
                &decl.ty,
            )?);
            let resolved_expr = resolve_expression(
                errors,
                type_scopes.clone(),
                scopes.clone(),
                type_defs,
                function_by_name,
                resolved_functions,
                &decl.value,
                annotation,
            )?;
            scopes
                .borrow_mut()
                .add(decl.name.clone(), resolved_expr.ty.clone());
            resolved_ast::Statement::VariableDecl(resolved_ast::VariableDecl {
                name: decl.name.clone(),
                value: resolved_expr,
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
