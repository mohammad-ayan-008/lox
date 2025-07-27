use std::{
    borrow::BorrowMut, cell::RefCell, collections::HashMap, fmt::Debug, io::{stdin, stdout, Write}, rc::Rc, time::{SystemTime, UNIX_EPOCH}
};

use crate::{
    environment::Environment, expr::{Expr, Literal}, stmt::Stmt, Tokentype::TokenType
};


#[derive(Clone,Debug)]
pub struct LoxInstance{
    pub class:LoxClass,
    pub fields:HashMap<String,Literal>,
    pub functions:HashMap<String,LoxFunction>,
}
impl LoxInstance{
    pub fn new(class:LoxClass,)->Self{
        Self{
            class,
            fields:HashMap::new(),
            functions:HashMap::new(),
        }
    }
    pub fn get(env:Rc<RefCell<Self>>,name:String)->Result<Literal,String>{
        if env.as_ref().borrow().fields.contains_key(&name){
            return Ok(env.as_ref().borrow().fields.get(&name).unwrap().clone());
        }

        if env.as_ref().borrow().functions.contains_key(&name){
            let lox_fn = env.as_ref().borrow().functions.get(&name).unwrap().clone().bind(env.clone());
            let callable:Rc<dyn LoxCallable> = Rc::new(lox_fn); 
            let fun:Literal = Literal::Callable(callable); 
            return Ok(fun);
        }
        Err("undefined property get".to_string())
    }

    pub fn set(&mut self,name:String,value:Literal)->Result<Literal,String>{
        if self.fields.contains_key(&name){
            self.fields.insert(name, value.clone());
            return Ok(value);
        }
        Err("undefined property".to_string())
    }

    pub fn set_new(&mut self,name:String,value:Literal){
        self.fields.insert(name,value);
    }
    pub fn set_new_fn(&mut self,name:String,value:LoxFunction){
        self.functions.insert(name,value);
    }
    

}

#[derive(Clone,Debug)]
pub struct LoxClass{
    pub name:String,
    pub fields:Vec<Stmt>,
    pub methods:HashMap<String,LoxFunction>
}
 
impl LoxCallable for LoxClass{
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Literal>) -> Literal {
        let mut class = LoxInstance::new(self.clone());
        for i in self.methods.iter(){

            class.set_new_fn(i.0.clone(), i.1.clone());
        }
        for i in self.fields.iter(){
            if let Stmt::Variable { op, expr }= i{
                 if let Some(a) = expr.as_ref(){
                 let value = interpreter.eval(a.clone()).unwrap();
                 class.set_new(op.lexeme.as_ref().unwrap().clone(), value);
                }else {   
                 class.set_new(op.lexeme.as_ref().unwrap().clone(), Literal::Nil);
                }
            }
        }
        let instance = Literal::Instance(Rc::new(RefCell::new( class)));
        instance
    }
}


#[derive(Clone,Debug)]
struct LoxFunction {
    decl: Stmt, // will be f(x)
    closure: Rc<RefCell<Environment>>,
}

impl LoxFunction {
    fn new(decl: Stmt,closure:Rc<RefCell<Environment>>) -> Self {
        Self { decl,closure }
    }
    fn bind(&mut self,instance:Rc<RefCell<LoxInstance>>)->Self{
        let env = Environment::enclose(self.closure.clone());
        env.as_ref().borrow_mut().define("this".to_owned(), Literal::Instance(instance));
        LoxFunction::new(self.decl.clone(), env)
    }
}
impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        let Stmt::Function_Decl { name:_, params, body:_ } = &self.decl else {
            unreachable!()
        };
        params.len()
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Literal>) -> Literal {

        let environment = Environment::enclose(Rc::clone(&self.closure));

        let Stmt::Function_Decl { name:_, params, body } = &self.decl else {
            unreachable!()
        };
        for (index, i) in params.iter().enumerate() {
            environment.as_ref().borrow_mut().define(
                i.lexeme.as_ref().unwrap().clone(),
                args.get(index).unwrap().clone(),
            );
        }
        if let Err(Error::Return(a)) = interpreter.execute_block(body.clone(), environment) {
            return a;
        }
        Literal::Nil
    }
}

pub trait LoxCallable {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, args: Vec<Literal>) -> Literal;
}

pub struct Interpreter {
    environemt: Rc<RefCell<Environment>>,
    global: Rc<RefCell<Environment>>,
    pub locals: HashMap<usize, usize>,
}
// global functions
struct Clock;
impl LoxCallable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _interpreter: &mut Interpreter, _args: Vec<Literal>) -> Literal {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        Literal::Number(time as f64)
    }
}
struct Input;
impl LoxCallable for Input {
    fn arity(&self) -> usize {
        1
    }

    fn call(&self, _interpreter: &mut Interpreter, args: Vec<Literal>) -> Literal {
        if let Literal::String(a) = args.get(0).unwrap() {
            stdout().flush().unwrap();
            let mut data = String::new();
            stdin().read_line(&mut data).unwrap();
            return Literal::String(data);
        }
        Literal::Nil
    }
}

#[derive(Debug)]
pub enum Error {
    Break,
    Continue,
    Return(Literal),
    Other(String),
}

impl Default for Interpreter {
    fn default() -> Self {
        let env = Rc::new(RefCell::new(Environment::new()));
        Interpreter {
            global: env.clone(),
            environemt: env,
            locals: HashMap::new(),
        }
    }
}

impl Interpreter {
    pub fn new() -> Self {
        let i = Interpreter::default();
        let func_clock: Rc<dyn LoxCallable> = Rc::new(Clock);
        let func_inp: Rc<dyn LoxCallable> = Rc::new(Input);
        i.global
            .as_ref()
            .borrow_mut()
            .define("clock".to_owned(), Literal::Callable(func_clock));
        i.global
            .as_ref()
            .borrow_mut()
            .define("input".to_owned(), Literal::Callable(func_inp));
        i
    }
    pub fn resolve(&mut self, id:usize, depth: usize) {
        self.locals.insert(id, depth);
    }
    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<(), Error> {
        for i in statements {
            self.execute(i)?;
        }
        Ok(())
    }
    pub fn execute_block(
        &mut self,
        stmt: Vec<Stmt>,
        env: Rc<RefCell<Environment>>,
    ) -> Result<(), Error> {

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
            Stmt::Class { name, functions,instance_variables }=>{
                self.environemt.as_ref().borrow_mut().define(name.lexeme.as_ref().unwrap().clone(), Literal::Nil);
                let mut met = HashMap::new();
                for i in functions.iter(){
                    if let Stmt::Function_Decl { name, params, body }= i{
                     let closure_snapshot = Rc::clone(&self.environemt);
                     let lox_fn = LoxFunction::new(i.clone(),closure_snapshot);
                     met.insert(name.lexeme.as_ref().unwrap().clone(), lox_fn);
                    }
                }

                let lox_class:Rc<dyn LoxCallable> = Rc::new(LoxClass{
                    name: name.lexeme.as_ref().unwrap().clone(),
                    fields:instance_variables,
                    methods:met
                });
                self.environemt.as_ref().borrow_mut().assign(&name.lexeme.as_ref().unwrap().clone(), &Literal::Callable(lox_class));
                Ok(())
            },
            Stmt::Return { token, value } => {
                let value = self.eval(value).unwrap();
                Err(Error::Return(value))
            }
            Stmt::Break => Err(Error::Break),
            Stmt::Continue => Err(Error::Continue),
            Stmt::Function_Decl {
                ref name,
                params: _,
                body: _,
            } => {
                let closure_snapshot = Rc::clone(&self.environemt);
                let lox_fn = LoxFunction::new(stmt.clone(),closure_snapshot);
                let callable: Rc<dyn LoxCallable> = Rc::new(lox_fn);
                self.environemt
                    .as_ref()
                    .borrow_mut()
                    .define(name.lexeme.clone().unwrap(), Literal::Callable(callable));
                Ok(())
            }
            Stmt::While {
                ref condition,
                ref stmts,
                finally,
            } => {
                'not_lb: while self.eval(condition.clone()).unwrap().is_truthly() {
                    match self.execute(*stmts.clone()) {
                        Err(Error::Continue) => {
                            // what if i just increment before the continue .?? it will fix the
                            // infinte loop ig
                            finally.as_ref().map(|a| self.eval(a.clone()).unwrap());
                            continue 'not_lb;
                        }
                        Err(Error::Break) => break 'not_lb,
                        Err(Error::Other(a)) => return Err(Error::Other(a)),
                        Err(Error::Return(a)) => return Err(Error::Return(a)),
                        Ok(_) => {}
                    }
                    finally.as_ref().map(|a| self.eval(a.clone()).unwrap());
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
            Stmt::Block { stmts } => {
                let child = Environment::enclose(Rc::clone(&self.environemt));
                self.execute_block(stmts, child)
            }
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

    pub fn look_up_variable(&mut self, name: &String, id:usize) -> Result<Literal, String> {
        let distance = self.locals.get(&id);

        if let Some(a) = distance {
            Environment::get_At(self.environemt.clone(), *a, name.clone())
        } else {
            self.environemt.as_ref().borrow_mut().get(name.to_string())
        }
    }
    pub fn eval(&mut self, expr: Expr) -> Result<Literal, String> {
        match expr {
            Expr::This { keyword, id }=>{
                let ins =self.look_up_variable(&"this".to_string(), id)?;  
                Ok(ins)
            },
            Expr::Set { expr, token, value }=>{
                let mut  v = self.eval(*expr)?;
                if let Literal::Instance(a) = v{
                     let val = self.eval(*value)?;
                    let values = a.as_ref().borrow_mut().set(token.lexeme.as_ref().unwrap().clone(), val)?;
                    return Ok(values);
                }
                Err("only instance have fields".to_string())
            }
            Expr::Get { expr, token }=>{
                let ob = self.eval(*expr)?;
                if let Literal::Instance(a) = ob{
                    let val = LoxInstance::get(a.clone(),token.lexeme.as_ref().unwrap().clone())?;
                    return Ok(val);
                }
                Err(format!(" only instances have property"))
                
            },
            Expr::Call {
                callie,
                paren,
                args,
            } => {
                let callie = self.eval(*callie)?;
                let mut arg = vec![];
                for i in args {
                    arg.push(self.eval(i.clone())?);
                }

                if let Literal::Callable(a) = callie {
                    if a.arity() != arg.len() {
                        return Err("arguments mismached".to_owned());
                    }
                    Ok(a.call(self, arg))
                } else {
                    Err("given type is not callable".to_owned())
                }
            }
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
            Expr::Assign { token, ref value,ref id } => {
                let expr = self.eval(*value.clone())?.clone();
                let distance = self.locals.get(id);
                if let Some(a) = distance {
                    Environment::ASSIGN_AT(
                        self.environemt.clone(),
                        *a,
                        &token.lexeme.unwrap(),
                        expr.clone(),
                    );
                } else {
                    self.environemt
                        .as_ref()
                        .borrow_mut()
                        .assign(&token.lexeme.unwrap(), &expr)?;
                }
                Ok(expr)
            }
            Expr::Variable { ref token, id } => {
                let ref name = token.lexeme.clone().unwrap();
                self.look_up_variable(name, id)
            }
            Expr::Literal { value } => Ok(value.clone()),
            Expr::Group { value } => Ok(self.eval(*value)?),
            Expr::Unary { op, expr } => {
                let token_type = op.token_type;
                let expr = self.eval(*expr);
                match (token_type, expr) {
                    (TokenType::Bang, Ok(any)) => Ok(Literal::from(!any.is_truthly())),
                    (TokenType::Minus, Ok(Literal::Number(a))) => Ok(Literal::Number(-a)),
                    _ => Err("not implemented for this type".to_owned()),
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
                        let str = format!("{}{}", a, b);
                        Ok(Literal::String(str))
                    }
                    (Literal::String(a), TokenType::Plus, Literal::Number(b)) => {
                        let str = format!("{}{}", a, b);
                        Ok(Literal::String(str))
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
