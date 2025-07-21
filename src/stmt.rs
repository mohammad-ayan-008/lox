use crate::{Tokentype::Token, expr::Expr};

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr {
        expr: Expr,
    },
    Return {
        token: Token,
        value: Expr,
    },
    Print {
        expr: Expr,
    },
    Variable {
        op: Token,
        expr: Option<Expr>,
    },
    Block {
        stmts: Vec<Stmt>,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        stmts: Box<Stmt>,
        finally: Option<Expr>,
    },
    Function_Decl {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    Break,
    Continue,
}
