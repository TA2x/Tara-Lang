#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum TypeAnnotation {
    Int,
    Float,
    Bool,
    Str,
    Void,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Expr {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Identifier(String),

    Assignment {
        name: String,
        value: Box<Expr>,
    },

    BinaryOp {
        left_operand: Box<Expr>,
        operator: BinaryOperator,
        right_operand: Box<Expr>,
    },

    UnaryOp {
        operator: UnaryOperator,
        operand: Box<Expr>,
    },

    Grouped(Box<Expr>),

    Call {
        callee: String,
        arguments: Vec<Expr>,
    },
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
    Negate,
    Not,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Stmt {
    Make {
        name: String,
        initializer: Expr,
    },

    Show {
        arguments: Vec<Expr>,
    },

    Return {
        value: Option<Expr>,
    },

    When {
        condition: Expr,
        then_branch: Vec<Stmt>,
        otherwise_branch: Option<Vec<Stmt>>,
    },

    During {
        condition: Expr,
        body: Vec<Stmt>,
    },

    For {
        init: Option<Box<Stmt>>,
        condition: Option<Expr>,
        update: Option<Box<Expr>>,
        body: Vec<Stmt>,
    },

    FuncDef {
        name: String,
        return_type: TypeAnnotation,
        parameters: Vec<(String, TypeAnnotation)>,
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
        Self { statements: Vec::new() }
    }
}