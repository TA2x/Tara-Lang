use crate::ast::{BinOp, Expr, Program, Stmt, UnOp};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Func {
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Nil,
}

impl Value {
    pub fn display(&self) -> String {
        match self {
            Value::Integer(n) => n.to_string(),
            Value::Float(f) => {
                let s = f.to_string();
                if s.ends_with(".0") {
                    s[..s.len() - 2].to_string()
                } else {
                    s
                }
            }
            Value::Str(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::Func { .. } => "<func>".into(),
            Value::Nil => "nil".into(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Nil => false,
            Value::Integer(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            _ => true,
        }
    }
}

#[derive(Debug)]
enum Signal {
    Continue,
    Return(Value),
}

struct Env {
    scopes: Vec<HashMap<String, Value>>,
}

impl Env {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    fn define_current(&mut self, name: String, val: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, val);
        }
    }

    fn declare_or_assign(&mut self, name: String, val: Value) {
        if self.assign(&name, val.clone()) {
            return;
        }
        self.define_current(name, val);
    }

    fn get(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v.clone());
            }
        }
        None
    }

    fn assign(&mut self, name: &str, val: Value) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), val);
                return true;
            }
        }
        false
    }
}

pub struct Interpreter {
    env: Env,
}

impl Interpreter {
    pub fn new() -> Self {
        Self { env: Env::new() }
    }

    pub fn run(&mut self, program: &Program) -> Result<(), String> {
        for stmt in &program.stmts {
            match self.exec(stmt)? {
                Signal::Return(_) | Signal::Continue => {}
            }
        }
        Ok(())
    }

    fn exec(&mut self, stmt: &Stmt) -> Result<Signal, String> {
        match stmt {
            Stmt::Make { name, init } => {
                let val = self.eval(init)?;
                self.env.declare_or_assign(name.clone(), val);
                Ok(Signal::Continue)
            }

            Stmt::Show { args } => {
                let parts: Result<Vec<String>, String> = args
                    .iter()
                    .map(|arg| self.eval(arg).map(|v| v.display()))
                    .collect();
                println!("{}", parts?.join(""));
                Ok(Signal::Continue)
            }

            Stmt::Return { value } => {
                let val = match value {
                    Some(expr) => self.eval(expr)?,
                    None => Value::Nil,
                };
                Ok(Signal::Return(val))
            }

            Stmt::When {
                condition,
                then_body,
                else_body,
            } => {
                if self.eval(condition)?.is_truthy() {
                    self.exec_block(then_body)
                } else if let Some(else_stmts) = else_body {
                    self.exec_block(else_stmts)
                } else {
                    Ok(Signal::Continue)
                }
            }

            Stmt::During { condition, body } => {
                while self.eval(condition)?.is_truthy() {
                    match self.exec_block(body)? {
                        Signal::Return(v) => return Ok(Signal::Return(v)),
                        Signal::Continue => {}
                    }
                }
                Ok(Signal::Continue)
            }

            Stmt::For {
                init,
                condition,
                update,
                body,
            } => {
                self.env.push();

                if let Some(s) = init {
                    self.exec(s)?;
                }

                loop {
                    let keep_going = match condition {
                        Some(cond) => self.eval(cond)?.is_truthy(),
                        None => true,
                    };
                    if !keep_going {
                        break;
                    }

                    match self.exec_block(body)? {
                        Signal::Return(v) => {
                            self.env.pop();
                            return Ok(Signal::Return(v));
                        }
                        Signal::Continue => {}
                    }

                    if let Some(upd) = update {
                        self.eval(upd)?;
                    }
                }

                self.env.pop();
                Ok(Signal::Continue)
            }

            Stmt::FuncDef {
                name, params, body, ..
            } => {
                let func = Value::Func {
                    params: params.clone(),
                    body: body.clone(),
                };
                self.env.declare_or_assign(name.clone(), func);
                Ok(Signal::Continue)
            }

            Stmt::ExprStmt(expr) => {
                self.eval(expr)?;
                Ok(Signal::Continue)
            }
        }
    }

    fn exec_block(&mut self, stmts: &[Stmt]) -> Result<Signal, String> {
        self.env.push();
        for stmt in stmts {
            match self.exec(stmt)? {
                Signal::Return(v) => {
                    self.env.pop();
                    return Ok(Signal::Return(v));
                }
                Signal::Continue => {}
            }
        }
        self.env.pop();
        Ok(Signal::Continue)
    }

    fn eval(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Integer(n) => Ok(Value::Integer(*n)),
            Expr::Float(f) => Ok(Value::Float(*f)),
            Expr::StringLit(s) => Ok(Value::Str(s.clone())),
            Expr::Boolean(b) => Ok(Value::Bool(*b)),
            Expr::Nil => Ok(Value::Nil),

            Expr::Identifier(name) => self
                .env
                .get(name)
                .ok_or_else(|| format!("undefined variable '{}'", name)),

            Expr::Assignment { name, value } => {
                let val = self.eval(value)?;
                if self.env.assign(name, val.clone()) {
                    Ok(val)
                } else {
                    Err(format!("'{}' is not declared — use `make` first", name))
                }
            }

            Expr::BinaryOp { left, op, right } => {
                let lhs = self.eval(left)?;
                let rhs = self.eval(right)?;
                apply_binop(&lhs, op, &rhs)
            }

            Expr::UnaryOp { op, operand } => {
                let val = self.eval(operand)?;
                match op {
                    UnOp::Neg => match val {
                        Value::Integer(n) => Ok(Value::Integer(-n)),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Err(format!("unary `-` on non-numeric value: {}", val.display())),
                    },
                    UnOp::Not => Ok(Value::Bool(!val.is_truthy())),
                }
            }

            Expr::Grouped(inner) => self.eval(inner),

            Expr::Call { callee, args } => self.call_function(callee, args),
        }
    }

    fn call_function(&mut self, callee: &str, args: &[Expr]) -> Result<Value, String> {
        let func = self
            .env
            .get(callee)
            .ok_or_else(|| format!("undefined function '{}'", callee))?;

        let (params, body) = match func {
            Value::Func { params, body } => (params, body),
            _ => return Err(format!("'{}' is not a function", callee)),
        };

        if args.len() != params.len() {
            return Err(format!(
                "'{}' expects {} arg(s), got {}",
                callee,
                params.len(),
                args.len()
            ));
        }

        let arg_vals: Vec<Value> = args
            .iter()
            .map(|arg| self.eval(arg))
            .collect::<Result<_, _>>()?;

        self.env.push();
        for (param, val) in params.into_iter().zip(arg_vals) {
            self.env.define_current(param, val);
        }

        let mut ret = Value::Nil;
        match self.exec_block(&body)? {
            Signal::Return(v) => ret = v,
            Signal::Continue => {}
        }

        self.env.pop();
        Ok(ret)
    }
}

fn apply_binop(left: &Value, op: &BinOp, right: &Value) -> Result<Value, String> {
    match (left, op, right) {
        (Value::Integer(a), BinOp::Add, Value::Integer(b)) => Ok(Value::Integer(a + b)),
        (Value::Integer(a), BinOp::Sub, Value::Integer(b)) => Ok(Value::Integer(a - b)),
        (Value::Integer(a), BinOp::Mul, Value::Integer(b)) => Ok(Value::Integer(a * b)),
        (Value::Integer(_), BinOp::Div, Value::Integer(0)) => {
            Err("integer division by zero".into())
        }
        (Value::Integer(a), BinOp::Div, Value::Integer(b)) => Ok(Value::Integer(a / b)),

        (Value::Float(a), BinOp::Add, Value::Float(b)) => Ok(Value::Float(a + b)),
        (Value::Float(a), BinOp::Sub, Value::Float(b)) => Ok(Value::Float(a - b)),
        (Value::Float(a), BinOp::Mul, Value::Float(b)) => Ok(Value::Float(a * b)),
        (Value::Float(_), BinOp::Div, Value::Float(b)) if *b == 0.0 => {
            Err("float division by zero".into())
        }
        (Value::Float(a), BinOp::Div, Value::Float(b)) => Ok(Value::Float(a / b)),

        (Value::Integer(a), BinOp::Add, Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
        (Value::Float(a), BinOp::Add, Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
        (Value::Integer(a), BinOp::Sub, Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
        (Value::Float(a), BinOp::Sub, Value::Integer(b)) => Ok(Value::Float(a - *b as f64)),
        (Value::Integer(a), BinOp::Mul, Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
        (Value::Float(a), BinOp::Mul, Value::Integer(b)) => Ok(Value::Float(a * *b as f64)),
        (Value::Integer(_), BinOp::Div, Value::Float(b)) if *b == 0.0 => {
            Err("division by zero".into())
        }
        (Value::Integer(a), BinOp::Div, Value::Float(b)) => Ok(Value::Float(*a as f64 / b)),
        (Value::Float(_), BinOp::Div, Value::Integer(0)) => Err("division by zero".into()),
        (Value::Float(a), BinOp::Div, Value::Integer(b)) => Ok(Value::Float(a / *b as f64)),

        (Value::Str(a), BinOp::Add, Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),

        (Value::Integer(a), BinOp::Lt, Value::Integer(b)) => Ok(Value::Bool(a < b)),
        (Value::Integer(a), BinOp::LtEq, Value::Integer(b)) => Ok(Value::Bool(a <= b)),
        (Value::Integer(a), BinOp::Gt, Value::Integer(b)) => Ok(Value::Bool(a > b)),
        (Value::Integer(a), BinOp::GtEq, Value::Integer(b)) => Ok(Value::Bool(a >= b)),

        (Value::Float(a), BinOp::Lt, Value::Float(b)) => Ok(Value::Bool(a < b)),
        (Value::Float(a), BinOp::LtEq, Value::Float(b)) => Ok(Value::Bool(a <= b)),
        (Value::Float(a), BinOp::Gt, Value::Float(b)) => Ok(Value::Bool(a > b)),
        (Value::Float(a), BinOp::GtEq, Value::Float(b)) => Ok(Value::Bool(a >= b)),

        (Value::Integer(a), BinOp::Lt, Value::Float(b)) => Ok(Value::Bool((*a as f64) < *b)),
        (Value::Integer(a), BinOp::LtEq, Value::Float(b)) => Ok(Value::Bool((*a as f64) <= *b)),
        (Value::Integer(a), BinOp::Gt, Value::Float(b)) => Ok(Value::Bool((*a as f64) > *b)),
        (Value::Integer(a), BinOp::GtEq, Value::Float(b)) => Ok(Value::Bool((*a as f64) >= *b)),
        (Value::Float(a), BinOp::Lt, Value::Integer(b)) => Ok(Value::Bool(*a < *b as f64)),
        (Value::Float(a), BinOp::LtEq, Value::Integer(b)) => Ok(Value::Bool(*a <= *b as f64)),
        (Value::Float(a), BinOp::Gt, Value::Integer(b)) => Ok(Value::Bool(*a > *b as f64)),
        (Value::Float(a), BinOp::GtEq, Value::Integer(b)) => Ok(Value::Bool(*a >= *b as f64)),

        (l, BinOp::Eq, r) => Ok(Value::Bool(l == r)),
        (l, BinOp::NotEq, r) => Ok(Value::Bool(l != r)),

        _ => Err(format!(
            "type error: cannot apply {:?} to {} and {}",
            op,
            left.display(),
            right.display()
        )),
    }
}
