#![allow(unused)]
use std::fmt::Display;

use thiserror::Error;

use crate::{ast::UnresolvedType, resolved_ast::ResolvedType};

#[derive(Debug)]
pub enum ContextType {
    // expressions
    VariableRefExpression,
    CallExpression,
    NumberLiteralExpression,
    BinaryExpression,
    IntrinsicExpression,
    // statements
    AsignStatement,
    ReturnStatement,
    DiscardedExpressionStatement,
    VariableDeclStatement,
    // toplevels
    Function,
}

#[derive(Debug, Error)]
pub enum CompileErrorKind {
    #[error("in {0:?}")]
    Context(ContextType),
    #[error("Variable `{name:?}` is not found in this scope.")]
    VariableNotFound { name: String },
    #[error("Function `{name:?}` is not found.")]
    FunctionNotFound { name: String },
    #[error("`{name:?}` is not a function")]
    IsNotFunction { name: String },
    #[error("`{name:?}` is not a typename")]
    IsNotType { name: String },
    #[error("`{name:?}` is not a variable")]
    IsNotVariable { name: String },
    #[error("Invalid operand.")]
    InvalidOperand(String),
    #[error("Invalid operand.")]
    InvalidArgument,
    #[error("Asign value does not match. expected `{expected}`, but got `{actual}`")]
    TypeMismatch { expected: String, actual: String },
    #[error("Return value does not match. expected `{expected}`, but got `{actual}`")]
    ReturnTypeMismatch { expected: String, actual: String },
    #[error("Cannot deref {name} for {deref_count:?} times.")]
    InvalidDeref { name: String, deref_count: u32 },
    #[error("Cannot access {ty} by index.")]
    InvalidIndexAccess { ty: ResolvedType },
    #[error("Array index must be an integer value")]
    InvalidArrayIndex,
    #[error("Cannot find type name {name}")]
    TypeNotFound { name: String },
    #[error("Mismatch generic args privided. `{name}` requires {expected} generic arguments, but got {actual}")]
    MismatchGenericArgCount {
        name: String,
        expected: usize,
        actual: usize,
    },
    #[error("`{name}` requires  no generic arguments.")]
    UnnecessaryGenericArgs { name: String },
    #[error("`{name}` requires generic arguments.")]
    NoGenericArgs { name: String },
    #[error("Cannot find field `{field_name:?}` in type `{type_name:?}`")]
    FieldNotFound {
        field_name: String,
        type_name: String,
    },
    #[error("Generic args length mismatch. expected {expected}, but got {actual}")]
    InvalidGenericArgsLength { expected: usize, actual: usize },
}

#[derive(Debug, Error)]
pub struct CompileError {
    errors: Vec<CompileErrorKind>,
}

#[derive(Debug)]
pub struct FaitalError(pub String);

impl CompileError {
    pub fn from_error_kind(kind: CompileErrorKind) -> Self {
        CompileError { errors: vec![kind] }
    }
    pub fn append(kind: CompileErrorKind, mut other: Self) -> Self {
        other.errors.push(kind);
        other
    }
}

impl Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for err in &self.errors {
            writeln!(f, "{}", err)?;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! error_context {
    ( $context_type:expr, $generator_block:expr ) => {
        match $generator_block {
            Ok(ret) => Ok(ret),
            Err(compile_error) => Err(self::error::CompileError::append(
                self::error::CompileErrorKind::Context($context_type),
                compile_error,
            )),
        }
    };
}
