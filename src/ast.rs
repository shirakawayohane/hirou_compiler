#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone)]
pub enum Expression {
    VariableRef {
        name: String,
    },
    NumberLiteral {
        value: String,
    },
    BinaryExpr {
        op: BinaryOp,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    CallExpr {
        name: String,
        args: Vec<Expression>,
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
    pub fn is_primitive(&self) -> bool {
        match self {
            Type::I32 => true,
            Type::USize => true,
            Type::U32 => true,
            Type::U64 => true,
            Type::U8 => true,
            Type::Ptr(_) => true,
            Type::Void => false,
        }
    }
    pub fn is_float_type(&self) -> bool {
        false
    }
    pub fn is_integer_type(&self) -> bool {
        match self {
            Type::I32 => true,
            Type::USize => true,
            Type::U8 => true,
            Type::U32 => true,
            Type::U64 => true,
            Type::Ptr(_) => false,
            Type::Void => false
        }
    }
    pub fn is_pointer(&self) -> bool {
        match self {
            Type::Ptr(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum Statement {
    Asignment {
        name: String,
        expression: Expression,
    },
    VariableDecl {
        ty: Type,
        name: String,
        value: Expression,
    },
    Return {
        expression: Option<Expression>,
    },
    DiscardedExpression {
        expression: Expression,
    },
}

#[derive(Debug)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<(Type, String)>,
    pub return_type: Type,
}

#[derive(Debug)]
pub enum TopLevel {
    Function {
        decl: FunctionDecl,
        body: Vec<Statement>,
    },
}

#[derive(Debug)]
pub struct Module {
    pub toplevels: Vec<TopLevel>,
}
