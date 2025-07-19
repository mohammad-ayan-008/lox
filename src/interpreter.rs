use std::{cell::RefCell, fmt::Debug, io::{stdin, stdout, Seek, Write}, ops::Deref, rc::Rc, time::{SystemTime, UNIX_EPOCH}};

use crate::{
    Tokentype::TokenType,
    environment::Environment,
    expr::{Expr, Literal},
    stmt::{self, Stmt},
};


pub trait LoxCallable{
    fn arity(&self)->usize;
    fn call(&self,interpreter:&mut Interpreter,args:Vec<Literal>)-> Literal;
}

pub struct Interpreter {
    environemt: Rc<RefCell<Environment>>,
    global:Rc<RefCell<Environment>>
}
// global functions
struct clock;
impl LoxCallable for clock{
    fn arity(&self)->usize {
       0 
    }

    fn call(&self,interpreter:&mut Interpreter,args:Vec<Literal>)-> Literal {
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        Literal::Number(time as f64)
    }
}
struct input;
impl LoxCallable for input{
    fn arity(&self)->usize {
       1
    }

    fn call(&self,interpreter:&mut Interpreter,args:Vec<Literal>)-> Literal {
        if let Literal::String(a) = args.get(0).unwrap(){
            print!("{}",a.clone().borrow());
            stdout().flush().unwrap();
            let mut data = String::new();
            stdin().read_line(&mut data).unwrap();
            return Literal::String(Rc::new(RefCell::new(data)));
        }
        Literal::Nil
    }
}


pub enum Error {
    Break,
    Continue,
    Other(String),
}

impl Default for Interpreter{
    fn default() -> Self {
        let env =Rc::new(RefCell::new(Environment::new()));
     Interpreter {
            global:env.clone(),
            environemt: env
        }
 
    }
}

impl Interpreter {
    pub fn new() -> Self {
        let i = Interpreter::default();
        let func_clock:Rc<dyn LoxCallable> = Rc::new(clock);
        let func_inp:Rc<dyn LoxCallable> = Rc::new(input);
        i.global.as_ref().borrow_mut().define("clock".to_string(), Literal::Function(func_clock));
        i.global.as_ref().borrow_mut().define("input".to_string(), Literal::Function(func_inp));
        i
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<(), Error> {
        for i in statements {
            self.execute(i)?;
        }
        Ok(())
    }
    pub fn execute_block(&mut self, stmt: Vec<Stmt>) -> Result<(), Error> {
        let mut env = Rc::new(RefCell::new(Environment::new()));
        env.as_ref().borrow_mut().enclosing = Some(self.environemt.clone());

        let previous = self.environemt.clone();
        self.environemt = env;

        for i in stmt {
            self.execute(i)?;
        }
        self.environemt = previous;
        Ok(())
    }

    pub fn execute(&mut self, stmt: Stmt) -> Result<(), Error> {
        match stmt {
            Stmt::Break => Err(Error::Break),
            Stmt::Continue => Err(Error::Continue),

            Stmt::While {
                ref condition,
                ref stmts,
                finally
            } => {
                'not_lb: while self.eval(condition.clone()).unwrap().is_truthly() {
                    match self.execute(*stmts.clone()) {
                        Err(Error::Continue) => {
                            // what if i just increment before the continue .?? it will fix the
                            // infinte loop ig
                            finally.as_ref().map(|a| {
                               self.eval(*a.clone()).unwrap()
                              });
                            continue 'not_lb;
                        }
                        Err(Error::Break) => break 'not_lb,
                        Err(Error::Other(a)) => return Err(Error::Other(a)),
                        Ok(_) => {}
                    }
                    finally.as_ref().map(|a| {
                      self.eval(*a.clone()).unwrap()
                    });
                }
                Ok(())
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.eval(condition).unwrap();
                if condition.is_truthly() {
                    self.execute(*then_branch)?;
                } else if else_branch.is_some() {
                    self.execute(*else_branch.unwrap())?;
                }
                Ok(())
            }
            Stmt::Block { stmts } => self.execute_block(stmts),
            Stmt::Variable { op, expr } => {
                if let Some(a) = expr {
                    let eval = self.eval(a).unwrap();
                    self.environemt
                        .as_ref()
                        .borrow_mut()
                        .define(op.lexeme.unwrap(), eval)
                } else {
                    let val = Literal::Nil;
                    self.environemt
                        .as_ref()
                        .borrow_mut()
                        .define(op.lexeme.unwrap(), val);
                }
                Ok(())
            }
            Stmt::Print { expr } => {
                let value = self.eval(expr).unwrap();
                println!("{}", value);
                Ok(())
            }
            Stmt::Expr { expr } => {
                self.eval(expr).unwrap();
                Ok(())
            }
        }
    }
    pub fn eval(&mut self, expr: Expr) -> Result<Literal, String> {
        match expr {
            Expr::Call { callie, paren, args }=>{
                let callie = self.eval(*callie)?;
                let mut arg = vec![];
                for i in args{
                    arg.push(self.eval(i.clone())?);
                }

                if let Literal::Function(a) = callie{
                    if a.arity() != arg.len(){
                      return Err("arguments mismached".to_string())
                    }
                    Ok(a.call(self,arg))
                }else {
                    Err("given type is not callable".to_string())
                }
            },
            Expr::Logical { left, op, right } => {
                let left = self.eval(*left)?;
                if op.token_type == TokenType::Or {
                    if left.is_truthly() {
                        return Ok(left);
                    }
                } else if !left.is_truthly() {
                    return Ok(left);
                }
                Ok(self.eval(*right)?)
            }
            Expr::Assign { token, value } => {
                let expr = self.eval(*value)?;
                self.environemt
                    .as_ref()
                    .borrow_mut()
                    .assign(&token.lexeme.unwrap(), &expr)?;
                Ok(expr)
            }
            Expr::Variable { token } => self
                .environemt
                .as_ref()
                .borrow_mut()
                .get(token.lexeme.unwrap()),
            Expr::Literal { value } => Ok(value.clone()),
            Expr::Group { value } => Ok(self.eval(*value)?),
            Expr::Unary { op, expr } => {
                let token_type = op.token_type;
                let expr = self.eval(*expr);
                match (token_type, expr) {
                    (TokenType::Bang, Ok(any)) => Ok(Literal::from(!any.is_truthly())),
                    (TokenType::Minus, Ok(Literal::Number(a))) => Ok(Literal::Number(-a)),
                    _ => Err("not implemented for this type".to_string()),
                }
            }
            Expr::Binary { left, op, right } => {
                let left = self.eval(*left)?;
                let right = self.eval(*right)?;
                let token = op.token_type;
                match (left, token, right) {
                    (Literal::Number(a), TokenType::Minus, Literal::Number(b)) => {
                        Ok(Literal::Number(a - b))
                    }
                    (Literal::Number(a), TokenType::Plus, Literal::Number(b)) => {
                        Ok(Literal::Number(a + b))
                    }
                    (Literal::Number(a), TokenType::Slash, Literal::Number(b)) => {
                        Ok(Literal::Number(a / b))
                    }
                    (Literal::Number(a), TokenType::Star, Literal::Number(b)) => {
                        Ok(Literal::Number(a * b))
                    }
                    (Literal::String(a), TokenType::Plus, Literal::String(b)) => {
                        a.borrow_mut().push_str(b.borrow().as_str());
                        Ok(Literal::String(a))
                    }
                    (Literal::String(a), TokenType::Plus, Literal::Number(b)) => {
                        a.borrow_mut().push_str(b.to_string().as_str());
                        Ok(Literal::String(a))
                    }
                    (Literal::Number(a), TokenType::Greater, Literal::Number(b)) => {
                        Ok(Literal::from(a > b))
                    }
                    (Literal::Number(a), TokenType::GreaterEqual, Literal::Number(b)) => {
                        Ok(Literal::from(a >= b))
                    }
                    (Literal::Number(a), TokenType::Less, Literal::Number(b)) => {
                        Ok(Literal::from(a < b))
                    }
                    (Literal::Number(a), TokenType::LessEqual, Literal::Number(b)) => {
                        Ok(Literal::from(a <= b))
                    }

                    (a, TokenType::EqualEqual, b) => Ok(Literal::from(a == b)),
                    (a, TokenType::BangEqual, b) => Ok(Literal::from(a != b)),

                    _ => Err("not implemented for this type".to_string()),
                }
            }
        }
    }
}


