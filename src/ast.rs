#[derive(Debug, Clone)]
pub enum Expr {
    Integer(i64),
    Float(f64),
    Identifier(String),

    BinaryOp {
        left_operand: Box<Expr>,
        operator: BinaryOperator,
        right_operand: Box<Expr>,
    },

    UnaryOp {
        operator: UnaryOperator,
        operand: Box<Expr>,
    },

    Grouped(Box<Expr>), // For expressions in parentheses

    Call {
        callee: String,
        arguments: Vec<Expr>,
    }
}

#[derive(Debug, Clonem PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,

    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Negate, // For unary minus (arithmetic negation)
    Not,    // For logical NOT (boolean negation)
}

#[derive(Debug, Clone)]
pub enum Stmt {
    LetBinding {
        name: String,
        value: Expr,
    },

    Return {
        value: Option<Expr>,
    },

    IfElse {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },

    WhileLoop {
        condition: Expr,
        body: Vec<Stmt>,
    },

    ForLoop {
        body: Vec<Stmt>,
    },

    FuncDef {
        name: String,
        parameters: Vec<String>,
        body: Vec<Stmt>,
    },

    Expression(Expr),
}