use std::fmt::{Display, Write};

use crate::{
    ast::{BinaryOp, MultiOp, UnaryOp},
    common::typename::*,
    resolved_ast::ResolvedType,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ConcreteStructType {
    pub name: String,
    pub non_generic_name: String,
    pub fields: Vec<(String, ConcreteType)>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct InterfaceRestriction {
    pub name: String,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Restriction {
    Interface(InterfaceRestriction),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ConcreteGenericType {
    pub name: String,
    pub restrictions: Vec<Restriction>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ConcreteType {
    I32,
    I64,
    U32,
    U64,
    U8,
    Bool,
    Ptr(Box<ConcreteType>),
    Void,
    StructLike(ConcreteStructType),
}

impl ConcreteType {
    pub fn is_integer_type(&self) -> bool {
        match self {
            ConcreteType::I32 => true,
            ConcreteType::U8 => true,
            ConcreteType::U32 => true,
            ConcreteType::I64 => true,
            ConcreteType::U64 => true,
            ConcreteType::Ptr(_) => false,
            ConcreteType::Void => false,
            ConcreteType::StructLike(_) => false,
            ConcreteType::Bool => false,
        }
    }
    pub fn is_signed_integer_type(&self) -> bool {
        match self {
            ConcreteType::I32 => true,
            ConcreteType::I64 => true,
            _ => false,
        }
    }
    pub fn is_struct_type(&self) -> bool {
        if let ConcreteType::StructLike(_) = self {
            true
        } else {
            false
        }
    }
    pub fn is_pointer_type(&self) -> bool {
        if let ConcreteType::Ptr(_) = self {
            true
        } else {
            false
        }
    }
    pub fn unwrap_primitive_into_resolved_type(&self) -> ResolvedType {
        match self {
            ConcreteType::I32 => ResolvedType::I32,
            ConcreteType::I64 => ResolvedType::I64,
            ConcreteType::U32 => ResolvedType::U32,
            ConcreteType::U64 => ResolvedType::U64,
            ConcreteType::U8 => ResolvedType::U8,
            ConcreteType::Bool => ResolvedType::Bool,
            ConcreteType::Void => ResolvedType::Void,
            ConcreteType::Ptr(inner) => {
                ResolvedType::Ptr(Box::new(inner.unwrap_primitive_into_resolved_type()))
            }
            _ => unimplemented!(),
        }
    }
}

impl Display for ConcreteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let ConcreteType::Ptr(inner_type) = self {
            return write!(f, "[{}]", inner_type);
        }
        if self.is_pointer_type() {
            f.write_char('&')?;
            if let ConcreteType::Ptr(inner_type) = self {
                write!(f, "{}", inner_type)
            } else {
                unreachable!()
            }
        } else {
            write!(
                f,
                "{}",
                match self {
                    ConcreteType::I32 => I32_TYPE_NAME,
                    ConcreteType::I64 => I64_TYPE_NAME,
                    ConcreteType::U32 => U32_TYPE_NAME,
                    ConcreteType::U64 => U64_TYPE_NAME,
                    ConcreteType::U8 => U8_TYPE_NAME,
                    ConcreteType::Bool => BOOL_TYPE_NAME,
                    ConcreteType::Void => VOID_TYPE_NAME,
                    ConcreteType::Ptr(inner) => {
                        return write!(f, "*{}", inner);
                    }
                    ConcreteType::StructLike(ConcreteStructType {
                        name,
                        fields: _,
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
    pub args: Vec<ConcreteExpression>,
    pub generic_args: Option<Vec<ConcreteType>>,
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
    pub fields: Vec<(String, ConcreteExpression)>,
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub lhs: Box<ConcreteExpression>,
    pub rhs: Box<ConcreteExpression>,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub operand: Box<ConcreteExpression>,
}

#[derive(Debug, Clone)]
pub struct MultiExpr {
    pub op: MultiOp,
    pub operands: Vec<ConcreteExpression>,
}

#[derive(Debug, Clone)]
pub struct DerefExpr {
    pub target: Box<ConcreteExpression>,
}

#[derive(Debug, Clone)]
pub struct IndexAccessExpr {
    pub target: Box<ConcreteExpression>,
    pub index: Box<ConcreteExpression>,
}

#[derive(Debug, Clone)]
pub struct FieldAccessExpr {
    pub target: Box<ConcreteExpression>,
    pub field_name: String,
}

#[derive(Debug, Clone)]
pub struct IfExpr {
    pub cond: Box<ConcreteExpression>,
    pub then: Box<ConcreteExpression>,
    pub els: Box<ConcreteExpression>,
}

#[derive(Debug, Clone)]
pub struct WhenExpr {
    pub cond: Box<ConcreteExpression>,
    pub then: Box<ConcreteExpression>,
}

#[derive(Debug, Clone)]
pub enum ExpressionKind {
    SizeOf(ConcreteType),
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
    Return(Return),
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ConcreteExpression {
    pub ty: ConcreteType,
    pub kind: ExpressionKind,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub name: String,
    pub value: Box<ConcreteExpression>,
    pub deref_count: usize,
    pub index_access: Option<Box<ConcreteExpression>>,
}

#[derive(Debug, Clone)]
pub struct VariableDecl {
    pub name: String,
    pub value: Box<ConcreteExpression>,
}

#[derive(Debug, Clone)]
pub struct VariableDecls {
    pub decls: Vec<VariableDecl>,
}

#[derive(Debug, Clone)]
pub struct Return {
    pub expression: Option<Box<ConcreteExpression>>,
}

#[derive(Debug, Clone)]
pub struct Effect {
    pub expression: ConcreteExpression,
}

#[derive(Debug, Clone)]
pub enum Argument {
    VarArgs,
    Normal(ConcreteType, String),
}

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub args: Vec<Argument>,
    pub return_type: ConcreteType,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub decl: FunctionDecl,
    pub body: Vec<ConcreteExpression>,
}

#[derive(Debug, Clone)]
pub enum TopLevel {
    Function(Function),
}

#[derive(Debug)]
pub struct ConcreteModule {
    pub toplevels: Vec<TopLevel>,
}
