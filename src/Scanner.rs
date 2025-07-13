use std::{
    collections::HashMap,
    env::{current_dir, set_var},
};

use crate::Tokentype::{self, Token, TokenType};

pub struct Scanner {
    source: String,
    pub tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    keywords: HashMap<String, TokenType>,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        let mut keywords = HashMap::new();
        keywords.insert("and".to_string(), TokenType::And);
        keywords.insert("class".to_string(), TokenType::Class);
        keywords.insert("else".to_string(), TokenType::Else);
        keywords.insert("false".to_string(), TokenType::False);
        keywords.insert("for".to_string(), TokenType::For);
        keywords.insert("fun".to_string(), TokenType::Fun);
        keywords.insert("if".to_string(), TokenType::If);
        keywords.insert("nil".to_string(), TokenType::Nil);
        keywords.insert("or".to_string(), TokenType::Or);
        keywords.insert("print".to_string(), TokenType::Print);
        keywords.insert("return".to_string(), TokenType::Return);
        keywords.insert("super".to_string(), TokenType::Super);
        keywords.insert("this".to_string(), TokenType::This);
        keywords.insert("true".to_string(), TokenType::True);
        keywords.insert("var".to_string(), TokenType::Var);
        keywords.insert("while".to_string(), TokenType::While);
        Self {
            source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
            keywords,
        }
    }

    pub fn scan_tokens(&mut self) {
        while !self.is_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.add_token(TokenType::Eof, None);
    }

    pub fn scan_token(&mut self) {
        match self.advance() {
            ';' => self.add_token(Tokentype::TokenType::Semicolon, None),
            '(' => self.add_token(Tokentype::TokenType::LeftParen, None),
            ')' => self.add_token(Tokentype::TokenType::RightParen, None),
            '}' => self.add_token(Tokentype::TokenType::RightBrace, None),
            '{' => self.add_token(Tokentype::TokenType::LeftBrace, None),
            ',' => self.add_token(Tokentype::TokenType::Comma, None),
            '.' => self.add_token(Tokentype::TokenType::Dot, None),
            '+' => self.add_token(Tokentype::TokenType::Plus, None),
            '-' => self.add_token(Tokentype::TokenType::Minus, None),
            '*' => self.add_token(Tokentype::TokenType::Star, None),
            '!' => {
                if self.match_token('=') {
                    self.add_token(TokenType::BangEqual, None);
                } else {
                    self.add_token(TokenType::Bang, None);
                }
            }
            '=' => {
                if self.match_token('=') {
                    self.add_token(TokenType::EqualEqual, None);
                } else {
                    self.add_token(TokenType::Equal, None);
                }
            }
            '>' => {
                if self.match_token('=') {
                    self.add_token(TokenType::GreaterEqual, None);
                } else {
                    self.add_token(TokenType::Greater, None);
                }
            }
            '<' => {
                if self.match_token('=') {
                    self.add_token(TokenType::LessEqual, None);
                } else {
                    self.add_token(TokenType::Less, None);
                }
            }
            '/' => {
                if self.match_token('/') {
                    while self.peek() != '\n' && !self.is_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash, None);
                }
            }
            ' ' | '\t' | '\r' => {}
            '\n' => self.line += 1,
            '"' => self.string_literal(),
            a if a.is_ascii_digit() => {
                self.number();
            }
            a if a.is_ascii_alphabetic() => {
                self.identifier();
            }
            a => {
                println!("unexpected ERROR at line {} {} ", self.line, a)
            }
        }
    }

    pub fn identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() {
            self.advance();
        }
        let text = self.source[self.start..self.current].to_owned();
        if let Some(a) = self.keywords.get(&text) {
            self.add_token(*a, None);
        } else {
            self.add_token(TokenType::Identifier, Some(text));
        }
    }

    pub fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        let number = self.source[self.start..self.current].to_owned();
        self.add_token(TokenType::Number, Some(number));
    }

    pub fn peek_next(&self) -> char {
        self.source.chars().nth(self.current + 1).unwrap_or('\0')
    }

    pub fn string_literal(&mut self) {
        while self.peek() != '"' && !self.is_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_end() {
            panic!("Unterminated string");
        }
        // consuming "
        self.advance();
        let string_literal = self.source[self.start + 1..self.current - 1].to_owned();
        self.add_token(TokenType::StringLiteral, Some(string_literal));
    }

    pub fn peek(&self) -> char {
        self.source.chars().nth(self.current).unwrap_or('\0')
    }
    pub fn match_token(&mut self, token: char) -> bool {
        if self.is_end() {
            return false;
        }

        if self.source.chars().nth(self.current).unwrap() != token {
            return false;
        }
        self.current += 1;
        true
    }
    pub fn is_end(&self) -> bool {
        self.current >= self.source.len()
    }
    pub fn add_token(&mut self, token: TokenType, lexeme: Option<String>) {
        let token = Token {
            token_type: token,
            lexeme,
            line: self.line,
        };
        self.tokens.push(token);
    }

    pub fn advance(&mut self) -> char {
        let char_data = self.source.chars().nth(self.current).unwrap_or('\0');
        self.current += 1;
        char_data
    }
}
