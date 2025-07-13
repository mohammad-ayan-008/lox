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
impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Literal::Nil, Literal::Nil) => true,
            (Literal::Nil, any) => false,
            (Literal::Number(a), Literal::Number(b)) => a == b,
            (Literal::String(a), Literal::String(b)) => {
                a.borrow().to_string() == b.borrow().to_string()
            }
            (Literal::False, any) => {
                if let Literal::False = any {
                    true
                } else {
                    false
                }
            }
            (Literal::True, any) => {
                if let Literal::True = any {
                    true
                } else {
                    false
                }
            }
            (Literal::Number(_), any) => {
                if let Literal::Number(_) = any {
                    true
                } else {
                    false
                }
            }
            (Literal::String(_), any) => {
                if let Literal::String(_) = any {
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl From<bool> for Literal {
    fn from(value: bool) -> Self {
        if value == true {
            Literal::True
        } else {
            Literal::False
        }
    }
}

impl Not for Literal {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Literal::True => Literal::False,
            Literal::False => Literal::True,
            Literal::Number(a) => {
                if (a == 0.0) {
                    Literal::False
                } else {
                    Literal::True
                }
            }
            Literal::String(a) => {
                if a.borrow().is_empty() {
                    Literal::False
                } else {
                    Literal::True
                }
            }
            Literal::Nil => Literal::False,
        }
    }
}

#[derive(Debug)]
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
    Assign {
        token: Token,
        value: Box<Expr>,
    },
}
