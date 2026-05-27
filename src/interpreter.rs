use std::collections::HashMap;
use crate::ast::{Expr, Stmt, Program, BinaryOperator, UnaryOperator};

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
                let s = f.to_string();
                if s.ends_with(".0") {
                    s[..s.len()-2].to_string()
                } else {
                    s
                }
            },
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Nil => "nil".to_string(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Nil => false,
            Value::Integer(n) => *n != 0,
            _ => true,
        }
    }
}

pub struct Interpreter {
    variables: HashMap<String, Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, program: &Program) {
        for stmt in &program.statements {
            if let Err(e) = self.execute_stmt(stmt) {
                eprintln!("Runtime error: {}", e);
                break;
            }
        }
    }

    fn execute_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Make { name, initializer } => {
                let value = self.eval_expr(initializer)?;
                self.variables.insert(name.clone(), value);
                Ok(())
            }

            Stmt::Show { arguments } => {
                for arg in arguments {
                    let value = self.eval_expr(arg)?;
                    println!("{}", value.to_string());
                }
                Ok(())
            }

            Stmt::When { condition, then_branch, otherwise_branch } => {
                let cond_value = self.eval_expr(condition)?;
                if cond_value.is_truthy() {
                    for stmt in then_branch {
                        self.execute_stmt(stmt)?;
                    }
                } else if let Some(else_stmts) = otherwise_branch {
                    for stmt in else_stmts {
                        self.execute_stmt(stmt)?;
                    }
                }
                Ok(())
            }

            Stmt::During { condition, body } => {
                while self.eval_expr(condition)?.is_truthy() {
                    for stmt in body {
                        self.execute_stmt(stmt)?;
                    }
                }
                Ok(())
            }

            Stmt::For { init, condition, update, body } => {
                if let Some(init_stmt) = init {
                    self.execute_stmt(init_stmt)?;
                }

                while if let Some(cond) = condition {
                    self.eval_expr(cond)?.is_truthy()
                } else {
                    true
                } {
                    for stmt in body {
                        self.execute_stmt(stmt)?;
                    }

                    if let Some(upd) = update {
                        self.eval_expr(upd)?;
                    }
                }
                Ok(())
            }

            Stmt::Return { value } => {
                if let Some(expr) = value {
                    let val = self.eval_expr(expr)?;
                    println!("{}", val.to_string());
                }
                Ok(())
            }

            Stmt::ExpressionStmt(expr) => {
                self.eval_expr(expr)?;
                Ok(())
            }

            Stmt::FuncDef { .. } => {
                // Function definitions not implemented yet
                Ok(())
            }
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Integer(n) => Ok(Value::Integer(*n)),
            Expr::Float(f) => Ok(Value::Float(*f)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Boolean(b) => Ok(Value::Boolean(*b)),

            Expr::Identifier(name) => {
                self.variables.get(name)
                    .cloned()
                    .ok_or_else(|| format!("Undefined variable: {}", name))
            }

            Expr::Assignment { name, value } => {
                let val = self.eval_expr(value)?;
                self.variables.insert(name.clone(), val.clone());
                Ok(val)
            }

            Expr::BinaryOp { left_operand, operator, right_operand } => {
                let left = self.eval_expr(left_operand)?;
                let right = self.eval_expr(right_operand)?;
                self.apply_binary_op(&left, operator, &right)
            }

            Expr::UnaryOp { operator, operand } => {
                let val = self.eval_expr(operand)?;
                match operator {
                    UnaryOperator::Negate => {
                        match val {
                            Value::Integer(n) => Ok(Value::Integer(-n)),
                            Value::Float(f) => Ok(Value::Float(-f)),
                            _ => Err("Cannot negate non-numeric value".to_string()),
                        }
                    }
                    UnaryOperator::Not => {
                        Ok(Value::Boolean(!val.is_truthy()))
                    }
                }
            }

            Expr::Grouped(inner) => self.eval_expr(inner),

            Expr::Call { callee, arguments } => {
                // Function calls not implemented yet
                Err(format!("Function calls not yet implemented: {}", callee))
            }
        }
    }

    fn apply_binary_op(&self, left: &Value, op: &BinaryOperator, right: &Value) -> Result<Value, String> {
        match (left, op, right) {
            // Arithmetic
            (Value::Integer(a), BinaryOperator::Add, Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Integer(a), BinaryOperator::Subtract, Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (Value::Integer(a), BinaryOperator::Multiply, Value::Integer(b)) => Ok(Value::Integer(a * b)),
            (Value::Integer(a), BinaryOperator::Divide, Value::Integer(b)) => {
                if *b == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::Integer(a / b))
                }
            }

            // Float arithmetic
            (Value::Float(a), BinaryOperator::Add, Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Float(a), BinaryOperator::Subtract, Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Float(a), BinaryOperator::Multiply, Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Float(a), BinaryOperator::Divide, Value::Float(b)) => {
                if *b == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::Float(a / b))
                }
            }

            // Mixed int/float
            (Value::Integer(a), BinaryOperator::Add, Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), BinaryOperator::Add, Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::Integer(a), BinaryOperator::Subtract, Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::Float(a), BinaryOperator::Subtract, Value::Integer(b)) => Ok(Value::Float(a - *b as f64)),
            (Value::Integer(a), BinaryOperator::Multiply, Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            (Value::Float(a), BinaryOperator::Multiply, Value::Integer(b)) => Ok(Value::Float(a * *b as f64)),
            (Value::Integer(a), BinaryOperator::Divide, Value::Float(b)) => {
                if *b == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::Float(*a as f64 / b))
                }
            }
            (Value::Float(a), BinaryOperator::Divide, Value::Integer(b)) => {
                if *b == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::Float(a / *b as f64))
                }
            }

            // String concatenation
            (Value::String(a), BinaryOperator::Add, Value::String(b)) => {
                Ok(Value::String(format!("{}{}", a, b)))
            }

            // Comparisons
            (Value::Integer(a), BinaryOperator::Less, Value::Integer(b)) => Ok(Value::Boolean(a < b)),
            (Value::Integer(a), BinaryOperator::Greater, Value::Integer(b)) => Ok(Value::Boolean(a > b)),
            (Value::Integer(a), BinaryOperator::LessEqual, Value::Integer(b)) => Ok(Value::Boolean(a <= b)),
            (Value::Integer(a), BinaryOperator::GreaterEqual, Value::Integer(b)) => Ok(Value::Boolean(a >= b)),

            // Float comparisons
            (Value::Float(a), BinaryOperator::Less, Value::Float(b)) => Ok(Value::Boolean(a < b)),
            (Value::Float(a), BinaryOperator::Greater, Value::Float(b)) => Ok(Value::Boolean(a > b)),
            (Value::Float(a), BinaryOperator::LessEqual, Value::Float(b)) => Ok(Value::Boolean(a <= b)),
            (Value::Float(a), BinaryOperator::GreaterEqual, Value::Float(b)) => Ok(Value::Boolean(a >= b)),

            // Equality
            (left, BinaryOperator::DoubleEqual, right) => Ok(Value::Boolean(left == right)),
            (left, BinaryOperator::NotEqual, right) => Ok(Value::Boolean(left != right)),

            _ => Err(format!("Invalid operation: {:?} {:?} {:?}", left, op, right)),
        }
    }
}
