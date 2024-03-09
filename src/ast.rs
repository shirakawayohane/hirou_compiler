use std::{
    fmt::{Display, Write},
    ops::Deref,
};

#[derive(Debug, Clone, Copy, PartialEq, Hash, Default)]
pub struct Position {
    pub line: u32,
    pub col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Default)]
pub struct Range {
    pub from: Position,
    pub to: Position,
}

#[derive(Debug, Clone, PartialEq, Hash)]
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
            value: &self.value.deref(),
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallExpr {
    pub name: String,
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

pub type LocatedExpr = Located<Box<Expression>>;

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub lhs: LocatedExpr,
    pub rhs: LocatedExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DerefExpr {
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
pub enum Expression {
    SizeOf(SizeOfExpr),
    VariableRef(VariableRefExpr),
    NumberLiteral(NumberLiteralExpr),
    StringLiteral(StringLiteralExpr),
    BoolLiteral(BoolLiteralExpr),
    StructLiteral(StructLiteralExpr),
    BinaryExpr(BinaryExpr),
    Call(CallExpr),
    DerefExpr(DerefExpr),
    IndexAccess(IndexAccessExpr),
    FieldAccess(FieldAccessExpr),
}

#[derive(Debug, PartialEq, Hash, Clone)]
pub struct TypeRef {
    pub name: String,
    pub generic_args: Option<Vec<Located<UnresolvedType>>>,
}

#[derive(Debug, PartialEq, Hash, Clone)]
pub enum UnresolvedType {
    TypeRef(TypeRef),
    Ptr(Box<Located<UnresolvedType>>),
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
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentStatement {
    pub deref_count: u32,
    pub index_access: Option<Located<Expression>>,
    pub name: String,
    pub expression: Located<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclStatement {
    pub ty: Option<Located<UnresolvedType>>,
    pub name: String,
    pub value: Located<Expression>,
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
    Assignment(AssignmentStatement),
    VariableDecl(VariableDeclStatement),
    Return(ReturnStatement),
    Effect(EffectStatement),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericArgument {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    VarArgs,
    Normal(Located<UnresolvedType>, String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    pub generic_args: Option<Vec<Located<GenericArgument>>>,
    pub args: Vec<Argument>,
    pub return_type: Located<UnresolvedType>,
    pub is_intrinsic: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub decl: FunctionDecl,
    pub body: Vec<Located<Statement>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructTypeDef {
    pub generic_args: Option<Vec<Located<GenericArgument>>>,
    pub fields: Vec<(String, Located<UnresolvedType>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeDefKind {
    Struct(StructTypeDef),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    pub name: String,
    pub kind: TypeDefKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TopLevel {
    Function(Function),
    TypeDef(TypeDef),
}

#[derive(Debug)]
pub struct Module {
    pub toplevels: Vec<Located<TopLevel>>,
}
