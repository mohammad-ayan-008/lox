use std::{cell::RefCell, env::set_var, rc::Rc};

use crate::{
    Tokentype::TokenType,
    environment::Environment,
    expr::{Expr, Literal},
    stmt::{self, Stmt},
};

pub struct Interpreter {
    environemt: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environemt: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<(), String> {
        for i in statements {
            self.execute(i)?;
        }
        Ok(())
    }
    pub fn execute_block(&mut self, stmt: Vec<Stmt>) -> Result<(), String> {
        

       
        let mut env = Rc::new(RefCell::new (Environment::new()));
        env.as_ref().borrow_mut().enclosing = Some(self.environemt.clone());

        let previous = self.environemt.clone();
        self.environemt = env;
        
        for i in stmt {
            self.execute(i)?;
        }
        self.environemt = previous;
        Ok(())
    }

    pub fn execute(&mut self, stmt: Stmt) -> Result<(), String> {
        match stmt {
            Stmt::While { condition, stmts }=>{
                while self.eval(condition.clone())?.is_truthly(){
                    self.execute(*stmts.clone())?;
                }
                Ok(())
            },
            Stmt::If { condition, then_branch, else_branch }=>{
                let condition = self.eval(condition)?;
                if condition.is_truthly(){
                    self.execute(*then_branch)?;
                }else if else_branch.is_some(){
                    self.execute(*else_branch.unwrap())?;
                }
                Ok(())
            },
            Stmt::Block { stmts } => self.execute_block(stmts),
            Stmt::Variable { op, expr } => {
                if let Some(a) = expr {
                    let eval = self.eval(a)?;
                    self.environemt.as_ref().borrow_mut().define(op.lexeme.unwrap(), eval)
                } else {
                    let val = Literal::Nil;
                    self.environemt.as_ref().borrow_mut().define(op.lexeme.unwrap(), val);
                }
                Ok(())
            }
            Stmt::Print { expr } => {
                let value = self.eval(expr)?;
                println!("{}", value);
                Ok(())
            }
            Stmt::Expr { expr } => {
                self.eval(expr)?;
                Ok(())
            }
        }
    }
    pub fn eval(&mut self, expr: Expr) -> Result<Literal, String> {
        match expr {
            Expr::Logical { left, op, right }=>{
                let left = self.eval(*left)?;
                if op.token_type == TokenType::Or{
                    if left.is_truthly(){
                        return Ok(left);
                    }
                }else if !left.is_truthly(){
                         return Ok(left);
                }
                Ok(self.eval(*right)?)
            },
            Expr::Assign { token, value } => {
                let expr = self.eval(*value)?;
                self.environemt.as_ref().borrow_mut().assign(&token.lexeme.unwrap(), &expr)?;
                Ok(expr)
            }
            Expr::Variable { token } => self.environemt.as_ref().borrow_mut().get(token.lexeme.unwrap()),
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
