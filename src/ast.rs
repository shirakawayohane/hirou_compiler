use std::{
    fmt::{Display, Write},
    ops::Deref,
};

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

impl Deref for Range {
    type Target = Position;
    fn deref(&self) -> &Self::Target {
        &self.from
    }
}

impl<T> Located<T> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Located<U> {
        Located {
            range: self.range,
            value: f(self.value),
        }
    }
    pub fn default(value: T) -> Self {
        Located {
            range: Range::default(),
            value,
        }
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

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub name: String,
    pub generic_args: Option<Vec<Located<UnresolvedType>>>,
    pub args: Vec<LocatedExpr>,
}

#[derive(Debug, Clone)]
pub struct VariableRefExpr {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct NumberLiteralExpr {
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct StringLiteralExpr {
    pub value: String,
}

pub type LocatedExpr = Located<Box<Expression>>;

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub lhs: LocatedExpr,
    pub rhs: LocatedExpr,
}

#[derive(Debug, Clone)]
pub struct DerefExpr {
    pub target: LocatedExpr,
}

#[derive(Debug, Clone)]
pub struct IndexAccessExpr {
    pub target: LocatedExpr,
    pub index: LocatedExpr,
}

#[derive(Debug, Clone)]
pub enum Expression {
    VariableRef(VariableRefExpr),
    NumberLiteral(NumberLiteralExpr),
    StringLiteral(StringLiteralExpr),
    BinaryExpr(BinaryExpr),
    Call(CallExpr),
    DerefExpr(DerefExpr),
    IndexAccess(IndexAccessExpr),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TypeRef {
    pub name: String,
    pub generic_args: Option<Vec<UnresolvedType>>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum UnresolvedType {
    TypeRef(TypeRef),
    Ptr(Box<UnresolvedType>),
}

#[derive(Debug)]
pub struct StructTypeDef {
    pub fields: Vec<(String, UnresolvedType)>,
}

#[derive(Debug)]
pub enum TypeDefKind {
    Struct(StructTypeDef),
}

impl Display for UnresolvedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnresolvedType::TypeRef(typeref) => {
                f.write_str(&typeref.name)?;
                if let Some(args) = &typeref.generic_args {
                    f.write_char('<')?;
                    for arg in args {
                        write!(f, "{}", arg)?;
                    }
                    f.write_char('>')?;
                }
            }
            UnresolvedType::Ptr(inner_type) => {
                f.write_char('[')?;
                write!(f, "{}", inner_type)?;
                f.write_char(']')?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AssignmentStatement {
    pub deref_count: u32,
    pub index_access: Option<Located<Expression>>,
    pub name: String,
    pub expression: Located<Expression>,
}

#[derive(Debug, Clone)]
pub struct VariableDeclStatement {
    pub ty: Located<UnresolvedType>,
    pub name: String,
    pub value: Located<Expression>,
}

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub expression: Option<Located<Expression>>,
}

#[derive(Debug, Clone)]
pub struct EffectStatement {
    pub expression: Located<Expression>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assignment(AssignmentStatement),
    VariableDecl(VariableDeclStatement),
    Return(ReturnStatement),
    Effect(EffectStatement),
}

#[derive(Debug, Clone)]
pub struct GenericArgument {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub generic_args: Option<Vec<Located<GenericArgument>>>,
    pub args: Vec<(Located<UnresolvedType>, String)>,
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
