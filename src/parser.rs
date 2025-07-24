use crate::{
    Tokentype::{Token, TokenType},
    expr::{Expr, Literal},
    stmt::Stmt,
};

/*
expression     → equality ;
equality       → comparison ( ( "!=" | "==" ) comparison )* ;
comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           → factor ( ( "-" | "+" ) factor )* ;
factor         → unary ( ( "/" | "*" ) unary )* ;
unary          → ( "!" | "-" ) unary
               | primary ;
primary        → NUMBER | STRING | "true" | "false" | "nil"
               | "(" expression ")" ;

*/

#[derive(Debug, PartialEq, Clone)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    loop_depth: usize,
    id:usize
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            loop_depth: 0,
            id:0
        }
    }

    pub fn parse_stmt(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = vec![];
        while !self.is_end() {
            stmts.push(self.declaration()?);
        }
        Ok(stmts)
    }

    pub fn declaration(&mut self) -> Result<Stmt, String> {
        if self.match_(&[TokenType::Var]) {
            return self.variable();
        }
        self.statements()
    }

    pub fn variable(&mut self) -> Result<Stmt, String> {
        let expr = self.consume(TokenType::Identifier, "Expected an identifier")?;
        let mut init: Option<Expr> = None;
        if self.match_(&[TokenType::Equal]) {
            let expr = self.expression()?;
            init = Some(expr);
        }
        self.consume(TokenType::Semicolon, "Expected ; after expression")?;
        Ok(Stmt::Variable {
            op: expr,
            expr: init,
        })
    }

    pub fn statements(&mut self) -> Result<Stmt, String> {
        if self.match_(&[TokenType::Print]) {
            self.print_statement()
        } else if self.match_(&[TokenType::Return]) {
            self.return_statement()
        } else if self.match_(&[TokenType::Fun]) {
            self.func_statement("function")
        } else if self.match_(&[TokenType::For]) {
            self.for_statement()
        } else if self.match_(&[TokenType::If]) {
            self.if_statement()
        } else if self.match_(&[TokenType::While]) {
            self.while_statement()
        } else if self.match_(&[TokenType::LeftBrace]) {
            self.block()
        } else if self.match_(&[TokenType::Break]) {
            if self.loop_depth == 0 {
                return Err(format!(
                    "Syntax error at line {}: 'break' used outside of loop",
                    self.previous().line
                ));
            }
            let res = Ok(Stmt::Break);
            self.consume(TokenType::Semicolon, "Expected ; after break")?;
            res
        } else if self.match_(&[TokenType::Continue]) {
            if self.loop_depth == 0 {
                return Err(format!(
                    "Syntax error at line {}: 'continue' used outside of loop",
                    self.previous().line
                ));
            }
            let res = Ok(Stmt::Continue);
            self.consume(TokenType::Semicolon, "Expected ; after break")?;
            res
        } else {
            self.expression_statement()
        }
    }

    pub fn return_statement(&mut self) -> Result<Stmt, String> {
        let token = self.previous().clone();
        let mut expr = Expr::Literal {
            value: Literal::Nil,
        };
        if !self.check(TokenType::Semicolon) {
            expr = self.expression()?;
        }
        self.consume(TokenType::Semicolon, "Expected ; after an expression")?;
        Ok(Stmt::Return {
            token: token,
            value: expr,
        })
    }

    pub fn func_statement(&mut self, name: &str) -> Result<Stmt, String> {
        let fn_name = self.consume(
            TokenType::Identifier,
            &format!("Expected kind {} name", name),
        )?;
        let left_paren = self.consume(
            TokenType::LeftParen,
            &format!("Expected ( after {} name ", name),
        )?;
        let mut params = vec![];
        loop {
            if params.len() >= 255 {
                return Err("cant have more than 255 args".to_string());
            }
            if self.check(TokenType::RightParen) {
                break;
            }
            params.push(self.consume(TokenType::Identifier, "Expected parameter name")?);
            if !self.match_(&[TokenType::Comma]) {
                break;
            }
        }

        let right_paren = self.consume(TokenType::RightParen, "Expected ) after parameters")?;
        self.consume(TokenType::LeftBrace, "Expected { before a block")?;
        let val = self.block()?;
        let Stmt::Block { stmts } = val else {
            unreachable!()
        };
        Ok(Stmt::Function_Decl {
            name: fn_name,
            params,
            body: stmts,
        })
    }

    pub fn for_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expected ( after for")?;
        let mut init = None;
        if self.match_(&[TokenType::Semicolon]) {
            init = None;
        } else if self.match_(&[TokenType::Var]) {
            init = Some(self.variable()?);
        } else {
            init = Some(self.expression_statement()?);
        }
        let mut condition = None;
        if !self.check(TokenType::Semicolon) {
            condition = Some(self.expression()?);
        }
        self.consume(TokenType::Semicolon, "Expected ; after loop condition ")?;
        let mut expr = None;
        if !self.check(TokenType::RightParen) {
            expr = Some(self.expression()?);
        }
        self.consume(TokenType::RightParen, "Expected ) after for clause")?;
        self.loop_depth += 1;
        let mut body = self.statements()?;
        self.loop_depth -= 1;
        if !matches!(body.clone(), Stmt::Block { stmts }) {
            return Err("Expected a block".to_owned());
        }
        if condition.is_none() {
            condition = Some(Expr::Literal {
                value: Literal::True,
            })
        }
        body = Stmt::While {
            condition: condition.unwrap(),
            stmts: Box::new(body),
            finally: expr,
        };

        if init.is_some() {
            body = Stmt::Block {
                stmts: vec![init.unwrap(), body],
            }
        }
        Ok(body)
    }
    pub fn while_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expected (  after while ")?;
        let expr = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ) after condition")?;
        self.loop_depth += 1;
        let statements = self.statements()?;
        self.loop_depth -= 1;
        if !matches!(statements.clone(), Stmt::Block { stmts }) {
            Err(format!("Expected a block found {:?}", statements))
        } else {
            Ok(Stmt::While {
                condition: expr,
                stmts: Box::new(statements),
                finally: None,
            })
        }
    }

    pub fn if_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expected ( after if statement")?;
        let conditions = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ) after statement")?;
        // if block

        let then_block = self.statements()?;
        //else block
        let mut else_block = None;
        if self.match_(&[TokenType::Else]) {
            else_block = Some(Box::new(self.statements()?));
        }
        Ok(Stmt::If {
            condition: conditions,
            then_branch: Box::new(then_block),
            else_branch: else_block,
        })
    }

    pub fn block(&mut self) -> Result<Stmt, String> {
        let mut stmts = vec![];
        while !self.check(TokenType::RightBrace) && !self.is_end() {
            stmts.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace, "Expected } after the block")?;
        Ok(Stmt::Block { stmts })
    }

    pub fn print_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "expected ( before print statement")?;
        let expression = self.expression()?;
        self.consume(TokenType::RightParen, "expected ) after print statement")?;
        self.consume(TokenType::Semicolon, "expected ; after end of statement")?;
        Ok(Stmt::Print { expr: expression })
    }

    pub fn expression_statement(&mut self) -> Result<Stmt, String> {
        let rtes = Ok(Stmt::Expr {
            expr: self.expression()?,
        });
        self.consume(TokenType::Semicolon, "Expected ; afer statement")?;
        rtes
    }

    pub fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    pub fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.or()?;
        if self.match_(&[TokenType::Equal]) {
            let value = self.assignment()?;
            if let Expr::Variable { ref token,id } = expr {
                
                self.id += 1;
                Ok(Expr::Assign {
                    token: token.clone(),
                    value: Box::new(value),
                    id:self.id
                })
                
            } else {
                Err("Invalid assignment target".to_string())
            }
        } else {
            Ok(expr)
        }
    }

    pub fn or(&mut self) -> Result<Expr, String> {
        let mut expr = self.and()?;
        while self.match_(&[TokenType::Or]) {
            let token = self.previous().clone();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                op: token,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    pub fn and(&mut self) -> Result<Expr, String> {
        let mut expr = self.equality()?;
        while self.match_(&[TokenType::And]) {
            let token = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                op: token,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }
    pub fn equality(&mut self) -> Result<Expr, String> {
        let mut lhs = self.comparison()?;
        while self.match_(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let token = self.previous().clone();
            let rhs = self.comparison()?;
            lhs = Expr::Binary {
                left: Box::new(lhs),
                op: token,
                right: Box::new(rhs),
            }
        }
        Ok(lhs)
    }
    pub fn comparison(&mut self) -> Result<Expr, String> {
        let mut lhs = self.term()?;
        while self.match_(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let token = self.previous().clone();
            let rhs = self.term()?;
            lhs = Expr::Binary {
                left: Box::new(lhs),
                op: token,
                right: Box::new(rhs),
            }
        }
        Ok(lhs)
    }

    pub fn term(&mut self) -> Result<Expr, String> {
        let mut lhs = self.factor()?;
        while self.match_(&[TokenType::Minus, TokenType::Plus]) {
            let token = self.previous().clone();
            let rhs = self.factor()?;
            lhs = Expr::Binary {
                left: Box::new(lhs),
                op: token,
                right: Box::new(rhs),
            }
        }
        Ok(lhs)
    }

    pub fn factor(&mut self) -> Result<Expr, String> {
        let mut lhs = self.unary()?;
        while self.match_(&[TokenType::Slash, TokenType::Star]) {
            let token = self.previous().clone();
            let rhs = self.unary()?;
            lhs = Expr::Binary {
                left: Box::new(lhs),
                op: token,
                right: Box::new(rhs),
            }
        }
        Ok(lhs)
    }

    pub fn unary(&mut self) -> Result<Expr, String> {
        if self.match_(&[TokenType::Minus, TokenType::Bang]) {
            let token = self.previous().clone();
            let expr = self.unary()?;
            return Ok(Expr::Unary {
                op: token,
                expr: Box::new(expr),
            });
        }
        self.call()
    }

    pub fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;
        loop {
            if self.match_(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }
        Ok(expr)
    }
    pub fn finish_call(&mut self, expr: Expr) -> Result<Expr, String> {
        let mut args = vec![];
        if !self.check(TokenType::RightParen) {
            if args.len() >= 255 {
                return Err("cant have more than 255 args".to_string());
            }
            'lo: loop {
                args.push(self.expression()?);
                if !self.match_(&[TokenType::Comma]) {
                    break 'lo;
                }
            }
        }
        let token = self.consume(TokenType::RightParen, "Expected ) after arguments.")?;
        Ok(Expr::Call {
            callie: Box::new(expr),
            paren: token,
            args,
        })
    }

    pub fn consume(&mut self, token: TokenType, msg: &str) -> Result<Token, String> {
        if self.check(token) {
            Ok(self.advance())
        } else {
            Err(format!("error at Line {} {msg}", self.previous().line))
        }
    }

    pub fn primary(&mut self) -> Result<Expr, String> {
        let token = self.advance();
        match token.token_type {
            TokenType::Identifier =>{ 
                self.id += 1;
                let name = token.lexeme.as_ref().unwrap().clone(); 
                Ok(Expr::Variable { token,id: self.id})
            }
            TokenType::Number => Ok(Expr::Literal {
                value: crate::expr::Literal::Number(token.lexeme.unwrap().parse::<f64>().unwrap()),
            }),
            TokenType::StringLiteral => Ok(Expr::Literal {
                value: crate::expr::Literal::String(token.lexeme.unwrap()),
            }),
            TokenType::Nil => Ok(Expr::Literal {
                value: crate::expr::Literal::Nil,
            }),
            TokenType::True => Ok(Expr::Literal {
                value: crate::expr::Literal::True,
            }),
            TokenType::False => Ok(Expr::Literal {
                value: crate::expr::Literal::False,
            }),
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "expected ) after Expression")?;
                Ok(Expr::Group {
                    value: Box::new(expr),
                })
            }
            a => Err(format!("cant parse {:?}", a)),
        }
    }

    pub fn previous(&mut self) -> &Token {
        &self.tokens[self.current - 1]
    }

    pub fn match_(&mut self, tokens: &[TokenType]) -> bool {
        for i in tokens {
            if self.check(*i) {
                self.advance();
                return true;
            }
        }
        false
    }
    pub fn check(&self, token: TokenType) -> bool {
        if self.is_end() {
            return false;
        }
        self.peek().token_type == token
    }

    pub fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    pub fn is_end(&self) -> bool {
        self.tokens[self.current].token_type == TokenType::Eof
    }
    pub fn advance(&mut self) -> Token {
        let token = self.tokens[self.current].clone();
        self.current += 1;
        token
    }
}
