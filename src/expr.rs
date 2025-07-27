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
    cell::RefCell, fmt::{write, Debug, Display}, rc::Rc
};

use crate::{
    Tokentype::Token,
    interpreter::LoxCallable,
    interpreter::{LoxClass,LoxInstance},
};

#[derive(Clone)]
pub enum Literal {
    Instance(LoxInstance),
    Callable(Rc<dyn LoxCallable>),
    Number(f64),
    String(String),
    False,
    True,
    Nil,
}
impl Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Instance(_)=>write!(f,"klass"),
            Literal::Callable(_) => write!(f, "func"),
            Literal::Nil => write!(f, "nil"),
            Literal::True => write!(f, "true"),
            Literal::False => write!(f, "false"),
            Literal::Number(a) => write!(f, "{}", a),
            Literal::String(a) => write!(f, "{}", a),
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Instance(a)=> write!(f,"{}",a.class.name),
            Literal::Callable(_) => write!(f, "func"),
            Literal::Nil => write!(f, "nil"),
            Literal::True => write!(f, "true"),
            Literal::False => write!(f, "false"),
            Literal::Number(a) => write!(f, "{}", a),
            Literal::String(a) => write!(f, "{}", a),
        }
    }
}
impl Literal {
    pub fn is_truthly(&self) -> bool {
        match self {
            Literal::Nil => false,
            Literal::False => false,
            _ => true,
        }
    }
}
impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Literal::Callable(a), any) => panic!("cant compare a function to other type"),
            (Literal::Nil, Literal::Nil) => true,
            (Literal::True, Literal::True) => true,
            (Literal::False, Literal::False) => true,
            (Literal::Number(a), Literal::Number(b)) => a == b,
            (Literal::String(a), Literal::String(b)) => a == b,
            _ => false,
        }
    }
}

impl From<bool> for Literal {
    fn from(value: bool) -> Self {
        if value { Literal::True } else { Literal::False }
    }
}

#[derive(Debug, Clone, PartialEq)]
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
        id:usize
    },
    Logical {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },
    Assign {
        token: Token,
        value: Box<Expr>,
        id:usize
    },
    Call {
        callie: Box<Expr>,
        paren: Token,
        args: Vec<Expr>,
    },
    Get{
        expr:Box<Expr>,
        token:Token
    },
    Set{
        expr:Box<Expr>,
        token:Token,
        value:Box<Expr>
    },
    This{
        keyword:Token,
        id:usize
    }
}
