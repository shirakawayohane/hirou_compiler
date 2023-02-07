use std::fmt::{Display, Write};

pub const U8_TYPE_NAME: &str = "u8";
pub const I32_TYPE_NAME: &str = "i32";
pub const U32_TYPE_NAME: &str = "u32";
pub const U64_TYPE_NAME: &str = "u64";
pub const USIZE_TYPE_NAME: &str = "usize";
pub const VOID_TYPE_NAME: &str = "void";

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Position {
    pub line: u32,
    pub col: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Range<'a> {
    pub from: Position,
    pub to: Position,
    pub fragment: &'a str,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Located<'a, T> {
    pub range: Range<'a>,
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
pub enum Expression<'a> {
    VariableRef {
        deref_count: u32,
        index_access: Option<Located<'a, Box<Expression<'a>>>>,
        name: String,
    },
    NumberLiteral {
        value: String,
    },
    BinaryExpr {
        op: BinaryOp,
        args: Vec<Located<'a, Expression<'a>>>,
    },
    CallExpr {
        name: String,
        args: Vec<Located<'a, Expression<'a>>>,
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

#[derive(Debug)]
pub enum Statement<'a> {
    Asignment {
        deref_count: u32,
        index_access: Option<Located<'a, Expression<'a>>>,
        name: String,
        expression: Located<'a, Expression<'a>>,
    },
    VariableDecl {
        ty: Located<'a, UnresolvedType>,
        name: String,
        value: Located<'a, Expression<'a>>,
    },
    Return {
        expression: Option<Located<'a, Expression<'a>>>,
    },
    Effect {
        expression: Located<'a, Expression<'a>>,
    },
}

#[derive(Debug)]
pub struct FunctionDecl<'a> {
    pub name: String,
    pub params: Vec<(Located<'a, UnresolvedType>, String)>,
    pub return_type: Located<'a, UnresolvedType>,
}

#[derive(Debug)]
pub enum TopLevel<'a> {
    Function {
        decl: FunctionDecl<'a>,
        body: Vec<Located<'a, Statement<'a>>>,
    },
}

#[derive(Debug)]
pub struct Module<'a> {
    pub toplevels: Vec<Located<'a, TopLevel<'a>>>,
}
