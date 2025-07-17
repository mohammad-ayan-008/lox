/*
expression     → literal
               | unary
               | binary
               | grouping ;

literal        → NUMBER | STRING | "true" | "false" | "nil" ;
grouping       → "(" expression ")" ;
unary          → ( "-" | "!" ) expression ;
binary         → expression operator expression ;
operator       → "==" | "!=" | "<" | "<=" | ">" | ">="
               | "+"  | "-"  | "*" | "/" ;
*/

use std::{
    cell::RefCell,
    env::set_var,
    fmt::{Display, write},
    ops::Not,
    rc::Rc,
};

use crate::Tokentype::{Token, TokenType};

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    String(Rc<RefCell<String>>),
    False,
    True,
    Nil,
}
impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Nil => write!(f, "nil"),
            Literal::True => write!(f, "true"),
            Literal::False => write!(f, "false"),
            Literal::Number(a) => write!(f, "{}", a),
            Literal::String(a) => write!(f, "{}", a.borrow()),
        }
    }
}
impl Literal {
    pub fn is_truthly(&self) -> bool {
        match self {
            Literal::Nil => false,
            Literal::True => true,
            Literal::False => false,
            _ => true,
        }
    }
}
impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Literal::Nil, Literal::Nil) => true,
            (Literal::Nil, any) => false,
            (Literal::Number(a), Literal::Number(b)) => a == b,
            (Literal::String(a), Literal::String(b)) => {
                a.borrow().to_string() == b.borrow().to_string()
            }
            (Literal::False, any) => matches!(any, Literal::False),
            (Literal::True, any) => matches!(any, Literal::True),
            (Literal::Number(_), any) => matches!(any, Literal::Number(_)),
            (Literal::String(_), any) => matches!(any, Literal::String(_)),
        }
    }
}

impl From<bool> for Literal {
    fn from(value: bool) -> Self {
        if value { Literal::True } else { Literal::False }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal {
        value: Literal,
    },
    Group {
        value: Box<Expr>,
    },
    Unary {
        op: Token,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },
    Variable {
        token: Token,
    },
    Logical {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },
    Assign {
        token: Token,
        value: Box<Expr>,
    },
}
