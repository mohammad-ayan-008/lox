use crate::{Tokentype::Token, expr::Expr};

#[derive(Debug)]
pub enum Stmt {
    Expr { expr: Expr },
    Print { expr: Expr },
    Variable { op: Token, expr: Option<Expr> },
    Block { stmts: Vec<Stmt> },
}
