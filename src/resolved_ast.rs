use std::fmt::{Display, Write};

use crate::ast::BinaryOp;

pub const VOID_TYPE_NAME: &str = "void";
pub const U8_TYPE_NAME: &str = "u8";
pub const U32_TYPE_NAME: &str = "u32";
pub const U64_TYPE_NAME: &str = "u64";
pub const I32_TYPE_NAME: &str = "i32";
pub const I64_TYPE_NAME: &str = "i64";
pub const USIZE_TYPE_NAME: &str = "usize";
pub const UNKNOWN_TYPE_NAME: &str = "unknown";

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ResolvedType {
    I32,
    I64,
    U32,
    U64,
    USize,
    U8,
    Ptr(Box<ResolvedType>),
    Void,
    Unknown,
}

impl ResolvedType {
    pub fn is_integer_type(&self) -> bool {
        match self {
            ResolvedType::I32 => true,
            ResolvedType::USize => true,
            ResolvedType::U8 => true,
            ResolvedType::U32 => true,
            ResolvedType::I64 => true,
            ResolvedType::U64 => true,
            ResolvedType::Ptr(_) => false,
            ResolvedType::Void => false,
            ResolvedType::Unknown => false,
        }
    }
    pub fn is_valid_as_operand(&self) -> bool {
        match self {
            ResolvedType::I32 => true,
            ResolvedType::I64 => true,
            ResolvedType::U32 => true,
            ResolvedType::U64 => true,
            ResolvedType::USize => true,
            ResolvedType::U8 => true,
            ResolvedType::Ptr(_) => false,
            ResolvedType::Void => false,
            ResolvedType::Unknown => false,
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

impl Display for ResolvedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let ResolvedType::Ptr(inner_type) = self {
            return write!(f, "[{}]", inner_type);
        }
        if self.is_pointer_type() {
            f.write_char('&')?;
            if let ResolvedType::Ptr(inner_type) = self {
                return write!(f, "{}", inner_type);
            } else {
                unreachable!()
            }
        } else {
            write!(
                f,
                "{}",
                match self {
                    ResolvedType::I32 => I32_TYPE_NAME,
                    ResolvedType::I64 => I64_TYPE_NAME,
                    ResolvedType::U32 => U32_TYPE_NAME,
                    ResolvedType::U64 => U64_TYPE_NAME,
                    ResolvedType::USize => USIZE_TYPE_NAME,
                    ResolvedType::U8 => U8_TYPE_NAME,
                    ResolvedType::Void => VOID_TYPE_NAME,
                    ResolvedType::Ptr(_) => unreachable!(),
                    ResolvedType::Unknown => UNKNOWN_TYPE_NAME,
                }
            )
        }
    }
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub name: String,
    pub args: Vec<ResolvedExpression>,
}

#[derive(Debug, Clone)]
pub struct VariableRefExpr {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct NumberLiteral {
    pub value: String,
    pub annotation: Option<ResolvedType>,
}

#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub lhs: Box<ResolvedExpression>,
    pub rhs: Box<ResolvedExpression>,
}

#[derive(Debug, Clone)]
pub struct DerefExpr {
    pub target: Box<ResolvedExpression>,
}

#[derive(Debug, Clone)]
pub struct IndexAccessExor {
    pub target: Box<ResolvedExpression>,
    pub index: Box<ResolvedExpression>,
}

#[derive(Debug, Clone)]
pub enum ExpressionKind {
    VariableRef(VariableRefExpr),
    NumberLiteral(NumberLiteral),
    StringLiteral(StringLiteral),
    BinaryExpr(BinaryExpr),
    CallExpr(CallExpr),
    Deref(DerefExpr),
    IndexAccess(IndexAccessExor),
}

#[derive(Debug, Clone)]
pub struct ResolvedExpression {
    pub ty: ResolvedType,
    pub kind: ExpressionKind,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub index_access: Option<ResolvedExpression>,
    pub deref_count: u32,
    pub name: String,
    pub expression: ResolvedExpression,
}

#[derive(Debug, Clone)]
pub struct VariableDecl {
    pub name: String,
    pub value: ResolvedExpression,
}

#[derive(Debug, Clone)]
pub struct Return {
    pub expression: Option<ResolvedExpression>,
}

#[derive(Debug, Clone)]
pub struct Effect {
    pub expression: ResolvedExpression,
}

#[derive(Debug, Clone)]
pub enum Statement {
    VariableDecl(VariableDecl),
    Return(Return),
    Assignmetn(Assignment),
    Effect(Effect),
}

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub args: Vec<(ResolvedType, String)>,
    pub return_type: ResolvedType,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub decl: FunctionDecl,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum TopLevel {
    Function(Function),
}

#[derive(Debug)]
pub struct Module {
    pub toplevels: Vec<TopLevel>,
}
