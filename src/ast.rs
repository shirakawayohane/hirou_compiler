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

#[derive(Debug)]
pub enum Type {
    I32,
    U64,
    U8,
    Ptr(Box<Type>),
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
