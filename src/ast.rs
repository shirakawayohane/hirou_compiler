use std::{
    fmt::{Display, Write},
    ops::Deref,
};

use crate::common::{AllocMode, StructKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Position {
    pub line: u32,
    pub col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Range {
    pub from: Position,
    pub to: Position,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Located<T> {
    pub range: Range,
    pub value: T,
}

impl<T, U> Located<T>
where
    T: Deref<Target = U>,
{
    pub fn as_inner_deref(&self) -> Located<&U> {
        Located {
            range: self.range,
            value: self.value.deref(),
        }
    }
}

#[allow(unused)]
impl<T> Located<T> {
    pub fn default_from(value: T) -> Self {
        Self {
            range: Default::default(),
            value,
        }
    }
    pub fn map(self, f: impl FnOnce(T) -> T) -> Self {
        Self {
            range: self.range,
            value: f(self.value),
        }
    }
    pub fn as_ref(&self) -> Located<&T> {
        Located {
            range: self.range,
            value: &self.value,
        }
    }
    pub fn as_deref(&self) -> Located<&T::Target>
    where
        T: Deref,
    {
        Located {
            range: self.range,
            value: self.value.deref(),
        }
    }
    pub fn transfer<U>(from: Located<T>, to: U) -> Located<U> {
        Located {
            range: from.range,
            value: to,
        }
    }
}

impl Deref for Range {
    type Target = Position;
    fn deref(&self) -> &Self::Target {
        &self.from
    }
}

impl<T> Deref for Located<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> Located<Box<T>> {
    pub fn unbox(self) -> Located<T> {
        Located {
            range: self.range,
            value: *self.value,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Equals,
    NotEquals,
    LessThan,
    LessThanOrEquals,
    GreaterThan,
    GreaterThanOrEquals,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum UnaryOp {
    Not,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MultiOp {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespacePath {
    pub segments: Vec<String>,
}

impl NamespacePath {
    pub fn simple(name: String) -> Self {
        Self { segments: vec![name] }
    }

    pub fn to_string(&self) -> String {
        self.segments.join("::")
    }

    pub fn is_namespaced(&self) -> bool {
        self.segments.len() > 1
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallExpr {
    pub name: NamespacePath,
    pub generic_args: Option<Vec<Located<UnresolvedType>>>,
    pub args: Vec<LocatedExpr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SizeOfExpr {
    pub ty: Located<UnresolvedType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableRefExpr {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberLiteralExpr {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringLiteralExpr {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoolLiteralExpr {
    pub value: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructLiteralExpr {
    pub name: String,
    pub generic_args: Option<Vec<Located<UnresolvedType>>>,
    pub fields: Vec<(String, LocatedExpr)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayLiteralExpr {
    pub elements: Vec<LocatedExpr>,
}

pub type LocatedExpr = Located<Box<Expression>>;

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub lhs: LocatedExpr,
    pub rhs: LocatedExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub operand: LocatedExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MultiExpr {
    pub op: MultiOp,
    pub operands: Vec<LocatedExpr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DerefExpr {
    pub target: LocatedExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AddressOfExpr {
    pub target: LocatedExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexAccessExpr {
    pub target: LocatedExpr,
    pub index: LocatedExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldAccessExpr {
    pub target: LocatedExpr,
    pub field_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfExpr {
    pub cond: LocatedExpr,
    pub then: LocatedExpr,
    pub els: LocatedExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhenExpr {
    pub cond: LocatedExpr,
    pub then: LocatedExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileExpr {
    pub cond: LocatedExpr,
    pub body: LocatedExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignExpr {
    pub deref_count: u32,
    pub index_access: Option<LocatedExpr>,
    pub name: String,
    pub value: LocatedExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDecl {
    pub name: String,
    pub ty: Option<Located<UnresolvedType>>,
    pub value: LocatedExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclsExpr {
    pub decls: Vec<Located<VariableDecl>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    SizeOf(SizeOfExpr),
    VariableRef(VariableRefExpr),
    NumberLiteral(NumberLiteralExpr),
    StringLiteral(StringLiteralExpr),
    BoolLiteral(BoolLiteralExpr),
    StructLiteral(StructLiteralExpr),
    ArrayLiteral(ArrayLiteralExpr),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Multi(MultiExpr),
    Call(CallExpr),
    DerefExpr(DerefExpr),
    AddressOf(AddressOfExpr),
    IndexAccess(IndexAccessExpr),
    FieldAccess(FieldAccessExpr),
    If(IfExpr),
    When(WhenExpr),
    While(WhileExpr),
    Assignment(AssignExpr),
    VariableDecl(VariableDeclsExpr),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TypeRef {
    pub name: String,
    pub generic_args: Option<Vec<Located<UnresolvedType>>>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum UnresolvedType {
    TypeRef(TypeRef),
    Ptr(Box<Located<UnresolvedType>>),
    Infer,
}

impl Display for UnresolvedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnresolvedType::TypeRef(typeref) => {
                f.write_str(&typeref.name)?;
                if let Some(args) = &typeref.generic_args {
                    f.write_char('<')?;
                    for arg in args {
                        write!(f, "{}", arg.value)?;
                    }
                    f.write_char('>')?;
                }
            }
            UnresolvedType::Ptr(inner_type) => {
                f.write_char('[')?;
                write!(f, "{}", inner_type.value)?;
                f.write_char(']')?;
            }
            UnresolvedType::Infer => {
                f.write_str("_")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStatement {
    pub expression: Option<Located<Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectStatement {
    pub expression: Located<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Return(ReturnStatement),
    Effect(EffectStatement),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Restriction {
    Interface(String),
}

impl Display for Restriction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Restriction::Interface(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericArgument {
    pub name: String,
    pub restrictions: Vec<Restriction>,
}

impl Display for GenericArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.restrictions.is_empty() {
            f.write_char(':')?;
            for (i, restriction) in self.restrictions.iter().enumerate() {
                if i != 0 {
                    f.write_str(" + ")?;
                }
                write!(f, "{}", restriction)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    VarArgs,
    SelfArg,
    Normal(Located<UnresolvedType>, String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub alloc_mode: Option<AllocMode>,
    pub name: String,
    pub generic_args: Option<Vec<Located<GenericArgument>>>,
    pub args: Vec<Argument>,
    pub return_type: Located<UnresolvedType>,
    pub is_intrinsic: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interface {
    pub name: String,
    pub generic_args: Vec<Located<GenericArgument>>,
    pub args: Vec<Argument>,
    pub return_type: Located<UnresolvedType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub decl: FunctionDecl,
    pub body: Vec<Located<Statement>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplementationDecl {
    pub alloc_mode: Option<AllocMode>,
    pub name: String,
    pub generic_args: Option<Vec<Located<GenericArgument>>>,
    pub target_ty: Located<UnresolvedType>,
    pub args: Vec<Argument>,
    pub return_type: Option<Located<UnresolvedType>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Implementation {
    pub decl: ImplementationDecl,
    pub body: Vec<Located<Statement>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructLikeTypeDef {
    pub struct_kind: StructKind,
    pub generic_args: Option<Vec<Located<GenericArgument>>>,
    pub fields: Vec<(String, Located<UnresolvedType>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeDefKind {
    StructLike(StructLikeTypeDef),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    pub name: String,
    pub kind: TypeDefKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UseStatement {
    pub path: NamespacePath,
    pub wildcard: bool, // true for `use Vec::*`, false for `use Vec::push`
}

#[derive(Debug, Clone, PartialEq)]
pub enum TopLevel {
    Function(Function),
    Implemantation(Implementation),
    TypeDef(TypeDef),
    Interface(Interface),
    Use(UseStatement),
}

#[derive(Debug)]
pub struct Module {
    pub toplevels: Vec<Located<TopLevel>>,
}
