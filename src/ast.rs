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
    IntValue {
        value: i32,
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
pub enum Statement {
    Asignment {
        name: String,
        expression: Expression,
    },
    VariableDecl {
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
    pub params: Vec<String>,
}

#[derive(Debug)]
pub struct Function {
    pub decl: FunctionDecl,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub struct Module {
    pub functions: Vec<Function>,
}
