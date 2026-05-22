#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Expr {
    Integer(i64),
    Float(f64),
    String(String),
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

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,

    Equal,
    DoubleEqual,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum UnaryOperator {
    Negate, // For unary minus (arithmetic negation)
    Not,    // For logical NOT (boolean negation)
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Stmt {
    LetBinding {
        name: String,
        initializer: Expr,
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

    ExpressionStmt(Expr),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    }
}