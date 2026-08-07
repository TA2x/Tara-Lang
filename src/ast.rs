#[derive(Debug, Clone, PartialEq)]
pub enum TypeAnnotation {
    Int,
    Float,
    Bool,
    Str,
    Void,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Integer(i64),
    Float(f64),
    StringLit(String),
    Boolean(bool),
    Nil,
    Identifier(String),

    Assignment {
        name: String,
        value: Box<Expr>,
    },

    BinaryOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },

    UnaryOp {
        op: UnOp,
        operand: Box<Expr>,
    },

    Grouped(Box<Expr>),

    Call {
        callee: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    LogicalAnd,
    LogicalOr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Make {
        name: String,
        init: Expr,
    },

    Show {
        args: Vec<Expr>,
    },

    Return {
        value: Option<Expr>,
    },

    When {
        condition: Expr,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
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
        return_type: Option<TypeAnnotation>,
        params: Vec<String>,
        body: Vec<Stmt>,
    },

    ExprStmt(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

impl Program {
    pub fn new() -> Self {
        Self { stmts: Vec::new() }
    }
}
