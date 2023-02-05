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
pub enum Type {
    I32,
    U32,
    U64,
    USize,
    U8,
    Ptr(Box<Type>),
    Void,
}

impl Type {
    pub fn is_integer_type(&self) -> bool {
        match self {
            Type::I32 => true,
            Type::USize => true,
            Type::U8 => true,
            Type::U32 => true,
            Type::U64 => true,
            Type::Ptr(_) => false,
            Type::Void => false,
        }
    }
    pub fn is_valid_as_operand(&self) -> bool {
        match self {
            Type::I32 => true,
            Type::U32 => true,
            Type::U64 => true,
            Type::USize => true,
            Type::U8 => true,
            Type::Ptr(_) => false,
            Type::Void => false,
        }
    }
    pub fn is_pointer_type(&self) -> bool {
        if let Type::Ptr(_) = self {
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
        ty: Located<'a, Type>,
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
    pub params: Vec<(Located<'a, Type>, String)>,
    pub return_type: Located<'a, Type>,
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
