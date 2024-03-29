use std::fmt::{Display, Write};

use crate::ast::{BinaryOp, MultiOp, UnaryOp};

pub const VOID_TYPE_NAME: &str = "void";
pub const U8_TYPE_NAME: &str = "u8";
pub const U32_TYPE_NAME: &str = "u32";
pub const U64_TYPE_NAME: &str = "u64";
pub const I32_TYPE_NAME: &str = "i32";
pub const I64_TYPE_NAME: &str = "i64";
pub const USIZE_TYPE_NAME: &str = "usize";
pub const BOOL_TYPE_NAME: &str = "bool";
pub const UNKNOWN_TYPE_NAME: &str = "unknown";

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ResolvedStructType {
    pub name: String,
    pub non_generic_name: String,
    pub fields: Vec<(String, ResolvedType)>,
    pub generic_args: Option<Vec<ResolvedType>>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ResolvedType {
    I32,
    I64,
    U32,
    U64,
    USize,
    U8,
    Bool,
    Ptr(Box<ResolvedType>),
    Void,
    Unknown,
    StructLike(ResolvedStructType),
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
            ResolvedType::StructLike(_) => false,
            ResolvedType::Bool => false,
        }
    }
    pub fn is_signed_integer_type(&self) -> bool {
        match self {
            ResolvedType::I32 => true,
            ResolvedType::I64 => true,
            _ => false,
        }
    }
    pub fn is_struct_type(&self) -> bool {
        if let ResolvedType::StructLike(_) = self {
            true
        } else {
            false
        }
    }
    pub fn is_pointer_type(&self) -> bool {
        if let ResolvedType::Ptr(_) = self {
            true
        } else {
            false
        }
    }
    pub fn can_insert(&self, other: &ResolvedType) -> bool {
        // void* には任意のポインタ型を代入できる
        {
            if let ResolvedType::Ptr(pointee_type) = self {
                if ResolvedType::Void == **pointee_type {
                    return other.is_pointer_type();
                }
            }
            if let ResolvedType::Ptr(pointee_type) = other {
                if ResolvedType::Void == **pointee_type {
                    return self.is_pointer_type();
                }
            }
        }
        // TODO: より高等な型チェック
        self == other
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
                write!(f, "{}", inner_type)
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
                    ResolvedType::Bool => BOOL_TYPE_NAME,
                    ResolvedType::Void => VOID_TYPE_NAME,
                    ResolvedType::Ptr(inner) => {
                        return write!(f, "*{}", inner);
                    }
                    ResolvedType::Unknown => UNKNOWN_TYPE_NAME,
                    ResolvedType::StructLike(ResolvedStructType {
                        name,
                        fields: _,
                        generic_args: _,
                        non_generic_name: _,
                    }) => {
                        name
                    }
                }
            )
        }
    }
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub callee: String,
    pub args: Vec<ResolvedExpression>,
    pub generic_args: Option<Vec<ResolvedType>>,
}

#[derive(Debug, Clone)]
pub struct VariableRefExpr {
    pub name: String,
}

// TODO: type毎に細かく分ける
#[derive(Debug, Clone)]
pub struct NumberLiteral {
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct BoolLiteral {
    pub value: bool,
}

#[derive(Debug, Clone)]
pub struct StructLiteral {
    pub fields: Vec<(String, ResolvedExpression)>,
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub lhs: Box<ResolvedExpression>,
    pub rhs: Box<ResolvedExpression>,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub operand: Box<ResolvedExpression>,
}

#[derive(Debug, Clone)]
pub struct MultiExpr {
    pub op: MultiOp,
    pub operands: Vec<ResolvedExpression>,
}

#[derive(Debug, Clone)]
pub struct DerefExpr {
    pub target: Box<ResolvedExpression>,
}

#[derive(Debug, Clone)]
pub struct IndexAccessExpr {
    pub target: Box<ResolvedExpression>,
    pub index: Box<ResolvedExpression>,
}

#[derive(Debug, Clone)]
pub struct FieldAccessExpr {
    pub target: Box<ResolvedExpression>,
    pub field_name: String,
}

#[derive(Debug, Clone)]
pub struct IfExpr {
    pub cond: Box<ResolvedExpression>,
    pub then: Box<ResolvedExpression>,
    pub els: Box<ResolvedExpression>,
}

#[derive(Debug, Clone)]
pub struct WhenExpr {
    pub cond: Box<ResolvedExpression>,
    pub then: Box<ResolvedExpression>,
}

#[derive(Debug, Clone)]
pub enum ExpressionKind {
    SizeOf(ResolvedType),
    VariableRef(VariableRefExpr),
    NumberLiteral(NumberLiteral),
    StringLiteral(StringLiteral),
    StructLiteral(StructLiteral),
    BoolLiteral(BoolLiteral),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Multi(MultiExpr),
    CallExpr(CallExpr),
    Deref(DerefExpr),
    IndexAccess(IndexAccessExpr),
    FieldAccess(FieldAccessExpr),
    If(IfExpr),
    When(WhenExpr),
    VariableDecls(VariableDecls),
    Assignment(Assignment),
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ResolvedExpression {
    pub ty: ResolvedType,
    pub kind: ExpressionKind,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub name: String,
    pub value: Box<ResolvedExpression>,
    pub deref_count: usize,
    pub index_access: Option<Box<ResolvedExpression>>,
}

#[derive(Debug, Clone)]
pub struct VariableDecl {
    pub name: String,
    pub value: Box<ResolvedExpression>,
}

#[derive(Debug, Clone)]
pub struct VariableDecls {
    pub decls: Vec<VariableDecl>,
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
    Return(Return),
    Effect(Effect),
}

#[derive(Debug, Clone)]
pub enum Argument {
    VarArgs,
    Normal(ResolvedType, String),
}

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub args: Vec<Argument>,
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
