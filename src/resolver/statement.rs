use std::cell::RefCell;
use std::collections::HashMap;

use std::rc::Rc;

use crate::ast::{Located, Statement};
use crate::resolved_ast::{self};

use crate::ast;

use super::error::{CompileError, FaitalError};
use super::expression::resolve_expression;

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
    })
}
