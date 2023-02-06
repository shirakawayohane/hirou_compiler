use std::fmt::{Display, Write};

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub line: u32,
    pub col: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Range<'a> {
    pub from: Position,
    pub to: Position,
    pub fragment: &'a str,
}

#[derive(Debug, Clone)]
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
pub enum UnresolvedType<'a> {
    TypeRef {
        name: &'a str,
        generic_args: Option<Vec<UnresolvedType<'a>>>,
    },
    Pointer {
        inner_type: Box<UnresolvedType<'a>>,
    },
    Array {
        inner_type: Box<UnresolvedType<'a>>,
    },
}

impl Display for UnresolvedType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnresolvedType::TypeRef { name, generic_args } => {
                f.write_str(name);
                if let Some(args) = generic_args {
                    f.write_char('<')?;
                    for arg in args {
                        arg.fmt(f)?;
                    }
                    f.write_char('>')?;
                }
            }
            UnresolvedType::Pointer { inner_type } => {
                f.write_char('*');
                inner_type.fmt(f)?;
            }
            UnresolvedType::Array { inner_type } => todo!(),
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
        ty: Located<'a, ResolvedType>,
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
    pub params: Vec<(Located<'a, UnresolvedType<'a>>, String)>,
    pub return_type: Located<'a, UnresolvedType<'a>>,
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
