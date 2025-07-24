use std::collections::HashMap;

use crate::{Tokentype::Token, expr::Expr, interpreter::Interpreter, stmt::Stmt};




pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
}
impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: vec![HashMap::new()],
        }
    }

    pub fn visit_block(&mut self, stmt: Stmt) {
        let Stmt::Block { stmts } = stmt else {
            unreachable!()
        };
        self.begin_scope();
        self.resolve(stmts);
        self.end_scope();
    }

    pub fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    pub fn resolve(&mut self, stmt: Vec<Stmt>) {
        for i in stmt {
            self.resolve_stmt(i);
        }
    }

    fn resolve_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Block { stmts } => {
                self.begin_scope();
                self.resolve(stmts);
                self.end_scope();
            }
            Stmt::Variable { op, expr } => {
                let name = op.lexeme.unwrap();
                self.declare(&name);
                if let Some(expr) = expr {
                    self.resolve_expr(expr);
                }
                self.define(&name);
            }
            Stmt::Function_Decl {
                ref name,
                params: _,
                body: _,
            } => {
                let ref name = name.lexeme.clone().unwrap();
                self.declare(name);
                self.define(name);
                self.resolve_function(stmt)
            }
            Stmt::Expr { expr } => {
                self.resolve_expr(expr);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition);
                self.resolve_stmt(*then_branch);
                if else_branch.is_some() {
                    self.resolve_stmt(*else_branch.unwrap());
                }
            }
            Stmt::Print { expr } => {
                self.resolve_expr(expr);
            }
            Stmt::Return { token, value } => {
                self.resolve_expr(value);
            }
            Stmt::While {
                condition,
                stmts,
                finally,
            } => {
                self.resolve_expr(condition);
                self.resolve_stmt(*stmts);
            }
            _ => (),
        }
    }

    pub fn resolve_function(&mut self, stmt: Stmt) {
        self.begin_scope();
        let Stmt::Function_Decl { name, params, body } = stmt else {
            unreachable!()
        };
        for i in params {
            let name = i.lexeme.as_ref().unwrap();
            self.declare(name);
            self.define(name);
        }
        self.resolve(body);
        self.end_scope();
    }
    pub fn define(&mut self, name: &String) {
        if self.scopes.is_empty() {
            return;
        }
        self.scopes.last_mut().unwrap().insert(name.clone(), true);
    }

    pub fn resolve_expr(&mut self, expr: Expr) {
        match expr {
            Expr::Variable { ref token , ref id} => {
                if !self.scopes.is_empty() && self.scopes.last().unwrap().get(token.lexeme.as_ref().unwrap()) ==Some(&false){
                    eprintln!("Can't read local variable in its own initializer.");
                    return;
                }
                let name = token.clone();
                self.resolve_local(name,*id);
            }
            Expr::Assign { token, value , id} => {
                self.resolve_expr(*value.clone());
                self.resolve_local(token.clone(),id);
            }
            Expr::Binary { left, op, right } => {
                self.resolve_expr(*left);
                self.resolve_expr(*right);
            }
            Expr::Call {
                callie,
                paren,
                args,
            } => {
                self.resolve_expr(*callie);
                for i in args {
                    self.resolve_expr(i);
                }
            }
            Expr::Group { value } => {
                self.resolve_expr(*value);
            }
            Expr::Unary { op, expr } => {
                self.resolve_expr(*expr);
            }
            Expr::Literal { value } => {}
            Expr::Logical { left, op, right } => {
                self.resolve_expr(*left);
                self.resolve_expr(*right);
            }
            _ => (),
        }
    }


 pub fn resolve_local(&mut self, token: Token, id: usize) {
    if let Some(name) = &token.lexeme {
        for (depth, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name) {
                self.interpreter.resolve(id, depth);
                return;
            }
        }
    }
}

    

    pub fn declare(&mut self, name: &String) {
        if self.scopes.is_empty() {
            return;
        }
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.clone(), false);
        }
    }

    pub fn end_scope(&mut self) {
        self.scopes.pop();
    }
}
