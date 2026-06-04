use std::collections::HashMap;
use crate::ast::{BinaryOperator, Expr, Program, Stmt, TypeAnnotation, UnaryOperator};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Value::Integer(n) => n.to_string(),
            Value::Float(f) => {
                let text = f.to_string();
                if text.ends_with(".0") { text[..text.len() - 2].to_string() } else { text }
            }
            Value::String(s)  => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Nil        => "nil".to_string(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Nil        => false,
            Value::Integer(n) => *n != 0,
            _                 => true,
        }
    }
}

#[derive(Debug, Clone)]
struct FuncDef {
    parameters: Vec<(String, TypeAnnotation)>,
    body: Vec<Stmt>,
}

enum StmtOutcome {
    Normal,
    Return(Value),
}

pub struct Interpreter {
    variables: HashMap<String, Value>,
    functions: HashMap<String, FuncDef>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, program: &Program) {
        for stmt in &program.statements {
            match self.execute_stmt(stmt) {
                Ok(StmtOutcome::Normal)    => {}
                Ok(StmtOutcome::Return(_)) => {}
                Err(runtime_error) => {
                    eprintln!("Runtime error: {}", runtime_error);
                    break;
                }
            }
        }
    }

    fn execute_stmt(&mut self, stmt: &Stmt) -> Result<StmtOutcome, String> {
        match stmt {

            Stmt::Make { name, initializer } => {
                let value = self.eval_expr(initializer)?;
                self.variables.insert(name.clone(), value);
                Ok(StmtOutcome::Normal)
            }

            Stmt::Show { arguments } => {
                let parts: Result<Vec<String>, String> = arguments
                    .iter()
                    .map(|arg| self.eval_expr(arg).map(|v| v.to_string()))
                    .collect();
                println!("{}", parts?.join(""));
                Ok(StmtOutcome::Normal)
            }

            Stmt::Return { value } => {
                let return_value = match value {
                    Some(expr) => self.eval_expr(expr)?,
                    None       => Value::Nil,
                };
                Ok(StmtOutcome::Return(return_value))
            }

            Stmt::When { condition, then_branch, otherwise_branch } => {
                let is_true = self.eval_expr(condition)?.is_truthy();
                let branch = if is_true {
                    then_branch
                } else if let Some(else_stmts) = otherwise_branch {
                    else_stmts
                } else {
                    return Ok(StmtOutcome::Normal);
                };
                self.execute_block(branch)
            }

            Stmt::During { condition, body } => {
                while self.eval_expr(condition)?.is_truthy() {
                    match self.execute_block(body)? {
                        StmtOutcome::Return(v) => return Ok(StmtOutcome::Return(v)),
                        StmtOutcome::Normal    => {}
                    }
                }
                Ok(StmtOutcome::Normal)
            }

            Stmt::For { init, condition, update, body } => {
                if let Some(init_stmt) = init {
                    self.execute_stmt(init_stmt)?;
                }

                loop {
                    let should_run = match condition {
                        Some(cond) => self.eval_expr(cond)?.is_truthy(),
                        None       => true,
                    };
                    if !should_run { break; }

                    match self.execute_block(body)? {
                        StmtOutcome::Return(v) => return Ok(StmtOutcome::Return(v)),
                        StmtOutcome::Normal    => {}
                    }

                    if let Some(upd) = update {
                        self.eval_expr(upd)?;
                    }
                }
                Ok(StmtOutcome::Normal)
            }

            Stmt::FuncDef { name, parameters, body, .. } => {
                self.functions.insert(name.clone(), FuncDef {
                    parameters: parameters.clone(),
                    body: body.clone(),
                });
                Ok(StmtOutcome::Normal)
            }

            Stmt::ExpressionStmt(expr) => {
                self.eval_expr(expr)?;
                Ok(StmtOutcome::Normal)
            }
        }
    }

    fn execute_block(&mut self, stmts: &[Stmt]) -> Result<StmtOutcome, String> {
        for stmt in stmts {
            match self.execute_stmt(stmt)? {
                StmtOutcome::Return(v) => return Ok(StmtOutcome::Return(v)),
                StmtOutcome::Normal    => {}
            }
        }
        Ok(StmtOutcome::Normal)
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Integer(n)  => Ok(Value::Integer(*n)),
            Expr::Float(f)    => Ok(Value::Float(*f)),
            Expr::String(s)   => Ok(Value::String(s.clone())),
            Expr::Boolean(b)  => Ok(Value::Boolean(*b)),

            Expr::Identifier(name) => {
                self.variables
                    .get(name)
                    .cloned()
                    .ok_or_else(|| format!("Undefined variable: '{}'", name))
            }

            Expr::Assignment { name, value } => {
                if !self.variables.contains_key(name) {
                    return Err(format!(
                        "Cannot assign to undeclared variable '{}'. Use 'make {} = ...' first.", name, name
                    ));
                }
                let val = self.eval_expr(value)?;
                self.variables.insert(name.clone(), val.clone());
                Ok(val)
            }

            Expr::BinaryOp { left_operand, operator, right_operand } => {
                let left  = self.eval_expr(left_operand)?;
                let right = self.eval_expr(right_operand)?;
                self.apply_binary_op(&left, operator, &right)
            }

            Expr::UnaryOp { operator, operand } => {
                let val = self.eval_expr(operand)?;
                match operator {
                    UnaryOperator::Negate => match val {
                        Value::Integer(n) => Ok(Value::Integer(-n)),
                        Value::Float(f)   => Ok(Value::Float(-f)),
                        _ => Err("Cannot negate a non-numeric value".to_string()),
                    },
                    UnaryOperator::Not => Ok(Value::Boolean(!val.is_truthy())),
                }
            }

            Expr::Grouped(inner) => self.eval_expr(inner),

            Expr::Call { callee, arguments } => self.call_function(callee, arguments),
        }
    }

    fn call_function(&mut self, name: &str, arg_exprs: &[Expr]) -> Result<Value, String> {
        let arg_values: Vec<Value> = arg_exprs
            .iter()
            .map(|expr| self.eval_expr(expr))
            .collect::<Result<_, _>>()?;

        let func = self.functions
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Undefined function: '{}'", name))?;

        if arg_values.len() != func.parameters.len() {
            return Err(format!(
                "Function '{}' expects {} argument(s) but got {}",
                name, func.parameters.len(), arg_values.len()
            ));
        }

        let caller_scope = self.variables.clone();

        for ((param_name, _param_type), arg_value) in func.parameters.iter().zip(arg_values) {
            self.variables.insert(param_name.clone(), arg_value);
        }

        let return_value = match self.execute_block(&func.body)? {
            StmtOutcome::Return(v) => v,
            StmtOutcome::Normal    => Value::Nil,
        };

        self.variables = caller_scope;

        Ok(return_value)
    }

    fn apply_binary_op(&self, left: &Value, op: &BinaryOperator, right: &Value) -> Result<Value, String> {
        match (left, op, right) {
            (Value::Integer(a), BinaryOperator::Add,      Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Integer(a), BinaryOperator::Subtract, Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (Value::Integer(a), BinaryOperator::Multiply, Value::Integer(b)) => Ok(Value::Integer(a * b)),
            (Value::Integer(a), BinaryOperator::Divide,   Value::Integer(b)) => {
                if *b == 0 { Err("Division by zero".to_string()) }
                else       { Ok(Value::Integer(a / b)) }
            }

            (Value::Float(a), BinaryOperator::Add,      Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Float(a), BinaryOperator::Subtract, Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Float(a), BinaryOperator::Multiply, Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Float(a), BinaryOperator::Divide,   Value::Float(b)) => {
                if *b == 0.0 { Err("Division by zero".to_string()) }
                else         { Ok(Value::Float(a / b)) }
            }

            (Value::Integer(a), BinaryOperator::Add,      Value::Float(b))   => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a),   BinaryOperator::Add,      Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::Integer(a), BinaryOperator::Subtract, Value::Float(b))   => Ok(Value::Float(*a as f64 - b)),
            (Value::Float(a),   BinaryOperator::Subtract, Value::Integer(b)) => Ok(Value::Float(a - *b as f64)),
            (Value::Integer(a), BinaryOperator::Multiply, Value::Float(b))   => Ok(Value::Float(*a as f64 * b)),
            (Value::Float(a),   BinaryOperator::Multiply, Value::Integer(b)) => Ok(Value::Float(a * *b as f64)),
            (Value::Integer(a), BinaryOperator::Divide,   Value::Float(b))   => {
                if *b == 0.0 { Err("Division by zero".to_string()) }
                else         { Ok(Value::Float(*a as f64 / b)) }
            }
            (Value::Float(a),   BinaryOperator::Divide,   Value::Integer(b)) => {
                if *b == 0 { Err("Division by zero".to_string()) }
                else       { Ok(Value::Float(a / *b as f64)) }
            }

            (Value::String(a), BinaryOperator::Add, Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),

            (Value::Integer(a), BinaryOperator::Less,         Value::Integer(b)) => Ok(Value::Boolean(a < b)),
            (Value::Integer(a), BinaryOperator::Greater,      Value::Integer(b)) => Ok(Value::Boolean(a > b)),
            (Value::Integer(a), BinaryOperator::LessEqual,    Value::Integer(b)) => Ok(Value::Boolean(a <= b)),
            (Value::Integer(a), BinaryOperator::GreaterEqual, Value::Integer(b)) => Ok(Value::Boolean(a >= b)),

            (Value::Float(a), BinaryOperator::Less,         Value::Float(b)) => Ok(Value::Boolean(a < b)),
            (Value::Float(a), BinaryOperator::Greater,      Value::Float(b)) => Ok(Value::Boolean(a > b)),
            (Value::Float(a), BinaryOperator::LessEqual,    Value::Float(b)) => Ok(Value::Boolean(a <= b)),
            (Value::Float(a), BinaryOperator::GreaterEqual, Value::Float(b)) => Ok(Value::Boolean(a >= b)),

            (l, BinaryOperator::DoubleEqual, r) => Ok(Value::Boolean(l == r)),
            (l, BinaryOperator::NotEqual,    r) => Ok(Value::Boolean(l != r)),

            _ => Err(format!("Type error: cannot apply {:?} to {:?} and {:?}", op, left, right)),
        }
    }
}