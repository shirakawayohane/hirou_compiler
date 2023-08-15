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

impl<T> Located<T> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Located<U> {
        Located {
            range: self.range,
            value: f(self.value),
        }
    }
    pub fn as_inner_ref(&self) -> Located<&T> {
        Located {
            range: self.range,
            value: &self.value,
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
    pub args: Vec<Located<Box<Expression>>>,
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
        args: Vec<Located<Box<Expression>>>,
    },
    CallExpr(CallExpr),
}

pub const VOID_TYPE_NAME: &str = "void";
pub const U8_TYPE_NAME: &str = "u8";
pub const U32_TYPE_NAME: &str = "u32";
pub const U64_TYPE_NAME: &str = "u64";
pub const I32_TYPE_NAME: &str = "i32";
pub const I64_TYPE_NAME: &str = "i64";
pub const USIZE_TYPE_NAME: &str = "usize";

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
pub enum IdentifierPrefix {
    Namespace(String),
    Alias(String),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TypeRef {
    pub prefix: Option<IdentifierPrefix>,
    pub name: String,
    pub generic_args: Option<Vec<UnresolvedType>>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum UnresolvedType {
    TypeRef(TypeRef),
    Ptr(Box<UnresolvedType>),
}

#[derive(Debug, Clone)]
pub struct GenericArgument {
    pub name: String,
    // TODO: impl bounds
}

#[derive(Debug)]
pub struct StructTypeDef {
    pub fields: Vec<(String, UnresolvedType)>
}

#[derive(Debug)]
pub enum TypeDefKind {
    Struct(StructTypeDef)
}

#[derive(Debug)]
pub struct TypeRegistration {
    pub ns: String,
    pub name: String,
    pub resolved_ty: ResolvedType
}

impl UnresolvedType {
    pub fn is_ptr_type(&self) -> bool {
        match self {
            UnresolvedType::Ptr(_) => true,
            _ => false,
        }
    }
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

impl Display for ResolvedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let ResolvedType::Ptr(inner_type) = self {
            return write!(f, "[{}]", inner_type);
        }
        write!(
            f,
            "{}",
            match self {
                ResolvedType::I32 => I32_TYPE_NAME,
                ResolvedType::U32 => U32_TYPE_NAME,
                ResolvedType::U64 => U64_TYPE_NAME,
                ResolvedType::USize => USIZE_TYPE_NAME,
                ResolvedType::U8 => U8_TYPE_NAME,
                ResolvedType::Ptr(_) => unreachable!(),
                ResolvedType::Void => VOID_TYPE_NAME,
            }
        )
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
