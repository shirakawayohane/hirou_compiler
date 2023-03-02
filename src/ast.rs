use std::fmt::{Display, Write};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub struct Position {
    pub line: u32,
    pub col: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub struct Range {
    pub from: Position,
    pub to: Position,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Located<T> {
    pub range: Range,
    pub value: T,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone)]
pub enum Expression {
    VariableRef {
        deref_count: u32,
        index_access: Option<Located<Box<Expression>>>,
        name: String,
    },
    NumberLiteral {
        value: String,
    },
    BinaryExpr {
        op: BinaryOp,
        args: Vec<Located<Expression>>,
    },
    CallExpr {
        name: String,
        args: Vec<Located<Expression>>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ResolvedType {
    I32,
    U32,
    U64,
    USize,
    U8,
    Ptr(Box<ResolvedType>),
    Void,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum UnresolvedType {
    TypeRef {
        name: String,
        generic_args: Option<Vec<UnresolvedType>>,
    },
    Array(Box<UnresolvedType>),
}

impl Display for UnresolvedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnresolvedType::TypeRef { name, generic_args } => {
                f.write_str(name)?;
                if let Some(args) = generic_args {
                    f.write_char('<')?;
                    for arg in args {
                        write!(f, "{}", arg)?;
                    }
                    f.write_char('>')?;
                }
            }
            UnresolvedType::Array(inner_type) => {
                f.write_char('[')?;
                write!(f, "{}", inner_type)?;
                f.write_char(']')?;
            }
        }
        Ok(())
    }
}

impl ResolvedType {
    pub fn is_integer_type(&self) -> bool {
        match self {
            ResolvedType::I32 => true,
            ResolvedType::USize => true,
            ResolvedType::U8 => true,
            ResolvedType::U32 => true,
            ResolvedType::U64 => true,
            ResolvedType::Ptr(_) => false,
            ResolvedType::Void => false,
        }
    }
    pub fn is_valid_as_operand(&self) -> bool {
        match self {
            ResolvedType::I32 => true,
            ResolvedType::U32 => true,
            ResolvedType::U64 => true,
            ResolvedType::USize => true,
            ResolvedType::U8 => true,
            ResolvedType::Ptr(_) => false,
            ResolvedType::Void => false,
        }
    }
    pub fn is_pointer_type(&self) -> bool {
        if let ResolvedType::Ptr(_) = self {
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    Asignment {
        deref_count: u32,
        index_access: Option<Located<Expression>>,
        name: String,
        expression: Located<Expression>,
    },
    VariableDecl {
        ty: Located<UnresolvedType>,
        name: String,
        value: Located<Expression>,
    },
    Return {
        expression: Option<Located<Expression>>,
    },
    Effect {
        expression: Located<Expression>,
    },
}

#[derive(Debug, Clone)]
pub struct GenericArgument {
    pub name: String,
    // TODO: impl bounds
}

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub generic_args: Option<Vec<Located<GenericArgument>>>,
    pub params: Vec<(Located<UnresolvedType>, String)>,
    pub return_type: Located<UnresolvedType>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub decl: FunctionDecl,
    pub body: Vec<Located<Statement>>,
}

#[derive(Debug, Clone)]
pub enum TopLevel {
    Function(Function),
}

#[derive(Debug)]
pub struct Module {
    pub toplevels: Vec<Located<TopLevel>>,
}
