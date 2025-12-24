use crate::lex::Lexer;
use crate::{Emitter, Token};
use std::collections::HashSet;

use crate::token::{BinaryOp, UnaryOp};

#[derive(Debug, Clone)]
pub enum Expr {
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Number(String),
    Identifier(String),
}

#[derive(Debug, Clone)]
pub struct IfBranch {
    pub condition: Expr,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub first_branch: IfBranch,
    pub other_branches: Vec<IfBranch>,
    pub else_body: Option<Vec<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum PrintValue {
    Str(String),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Print(PrintValue),
    If(IfStmt),
    While(WhileStmt),
    Label(String),
    Goto(String),
    Input(String),
    Let(String, Expr),
}

pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self { lexer }
    }
    fn newline_optional(&mut self) {
        while *self.lexer.peek_token() == Token::NEWLINE {
            let _ = self.lexer.next_token();
        }
    }

    fn newline(&mut self) {
        match self.lexer.next_token() {
            Token::NEWLINE => self.newline_optional(),
            other => panic!("Parse error. Expected newline but found {:?}", other),
        }
    }

    fn primary(&mut self) -> Expr {
        match self.lexer.next_token() {
            Token::NUMBER(n) => Expr::Number(n),
            Token::IDENT(s) => Expr::Identifier(s),
            Token::OPENPAREN => {
                let expr = self.expression();
                if !matches!(self.lexer.next_token(), Token::CLOSEPAREN) {
                    panic!("Parse error. Missing close paren after expression")
                }
                expr
            }
            other => panic!(
                "Parse error. Expected number or identifier but found {:?}",
                other
            ),
        }
    }

    fn unary_expr(&mut self) -> Expr {
        if matches!(self.lexer.peek_token(), Token::PLUS | Token::MINUS) {
            let unary_op = match self.lexer.next_token() {
                Token::PLUS => UnaryOp::Plus,
                Token::MINUS => UnaryOp::Minus,
                _ => unreachable!(),
            };

            let expr = Box::new(self.primary());
            Expr::Unary(unary_op, expr)
        } else {
            self.primary()
        }
    }

    fn term_expr(&mut self) -> Expr {
        let mut lhs = self.unary_expr();

        while matches!(self.lexer.peek_token(), Token::SLASH | Token::ASTERISK) {
            let binary_op = match self.lexer.next_token() {
                Token::SLASH => BinaryOp::Slash,
                Token::ASTERISK => BinaryOp::Asterisk,
                _ => unreachable!(),
            };

            let rhs = self.unary_expr();
            lhs = Expr::Binary(binary_op, Box::new(lhs), Box::new(rhs));
        }

        lhs
    }

    fn math_expr(&mut self) -> Expr {
        let mut lhs = self.term_expr();
        while matches!(self.lexer.peek_token(), Token::PLUS | Token::MINUS) {
            let binary_op = match self.lexer.next_token() {
                Token::PLUS => BinaryOp::Plus,
                Token::MINUS => BinaryOp::Minus,
                _ => unreachable!(),
            };

            let rhs = self.term_expr();
            lhs = Expr::Binary(binary_op, Box::new(lhs), Box::new(rhs));
        }
        lhs
    }

    fn comparison_expr(&mut self) -> Expr {
        let mut lhs = self.math_expr();

        while self.lexer.peek_token().is_comparator() {
            let binary_op = self
                .lexer
                .next_token()
                .binary_op()
                .expect("checked for comparator");
            let rhs = self.math_expr();
            lhs = Expr::Binary(binary_op, Box::new(lhs), Box::new(rhs));
        }
        lhs
    }

    fn not_expr(&mut self) -> Expr {
        if matches!(self.lexer.peek_token(), Token::NOT) {
            let unary_op = UnaryOp::Not;
            let _ = self.lexer.next_token();
            Expr::Unary(unary_op, Box::new(self.comparison_expr()))
        } else {
            self.comparison_expr()
        }
    }

    fn and_expr(&mut self) -> Expr {
        let mut lhs = self.not_expr();
        while matches!(self.lexer.peek_token(), Token::AND) {
            let rhs = self.not_expr();
            lhs = Expr::Binary(BinaryOp::And, Box::new(lhs), Box::new(rhs));
        }
        lhs
    }

    fn expression(&mut self) -> Expr {
        let mut lhs = self.and_expr();
        while matches!(self.lexer.peek_token(), Token::OR) {
            let rhs = self.and_expr();
            lhs = Expr::Binary(BinaryOp::Or, Box::new(lhs), Box::new(rhs));
        }
        lhs
    }

    fn if_branch(&mut self) -> IfBranch {
        let condition = self.expression();

        let next_token = self.lexer.next_token();
        if !matches!(next_token, Token::THEN) {
            panic!("Parser error. Expected THEN but found {:?}", next_token);
        }

        self.newline();

        let mut body = Vec::new();
        while !matches!(
            self.lexer.peek_token(),
            Token::ENDIF | Token::ELSEIF | Token::ELSE,
        ) {
            body.push(self.statement());
        }

        IfBranch { condition, body }
    }

    fn statement(&mut self) -> Stmt {
        let stmt = match self.lexer.next_token() {
            Token::EOF => {
                panic!("Parser error: Expected start of statement but reached end of file")
            }
            Token::PRINT => {
                let print_value = match self.lexer.peek_token().clone() {
                    Token::STRING(s) => {
                        let _ = self.lexer.next_token();
                        PrintValue::Str(s)
                    }
                    _ => PrintValue::Expr(self.expression()),
                };

                Stmt::Print(print_value)
            }
            Token::IF => {
                let first_branch = self.if_branch();

                let mut other_branches = Vec::new();
                while matches!(self.lexer.peek_token(), Token::ELSEIF) {
                    self.lexer.next_token();
                    other_branches.push(self.if_branch());
                }

                let else_body = if matches!(self.lexer.peek_token(), Token::ELSE) {
                    self.lexer.next_token();
                    self.newline();

                    let mut stmts = Vec::new();
                    while !matches!(self.lexer.peek_token(), Token::ENDIF) {
                        stmts.push(self.statement())
                    }
                    Some(stmts)
                } else {
                    None
                };

                // consume the last seen ENDIF
                if !matches!(self.lexer.next_token(), Token::ENDIF) {
                    panic!("Parse error: Expected next token to be ENDIF")
                }

                Stmt::If(IfStmt {
                    first_branch,
                    other_branches,
                    else_body,
                })
            }
            Token::WHILE => {
                let condition = self.expression();

                match self.lexer.next_token() {
                    Token::REPEAT => {}
                    other => panic!("Parser error. Expected REPEAT but found {:?}", other),
                }
                self.newline();

                let mut body = Vec::new();
                while *self.lexer.peek_token() != Token::ENDWHILE {
                    body.push(self.statement());
                }

                // consume the ENDWHILE
                self.lexer.next_token();

                Stmt::While(WhileStmt { condition, body })
            }
            Token::LABEL => match self.lexer.next_token() {
                Token::IDENT(label) => Stmt::Label(label),
                other => panic!("Parser error. Expected identifier but found {:?}", other),
            },
            Token::GOTO => match self.lexer.next_token() {
                Token::IDENT(label) => Stmt::Goto(label),
                other => panic!("Parser error. Expected identifier but found {:?}", other),
            },
            Token::INPUT => match self.lexer.next_token() {
                Token::IDENT(symbol) => Stmt::Input(symbol),
                other => panic!("Parser error. Expected identifier but found {:?}", other),
            },
            Token::LET => {
                let symbol = match self.lexer.next_token() {
                    Token::IDENT(symbol) => symbol,
                    other => panic!("Parser error. Expected identifier but found {:?}", other),
                };

                match self.lexer.next_token() {
                    Token::EQ => {}
                    other => panic!("Parser error. Expected `=` but found {:?}", other),
                }

                let expr = self.expression();
                Stmt::Let(symbol, expr)
            }
            other => panic!(
                "Parser error. Expected start of statement but found {:?}",
                other
            ),
        };

        self.newline();

        stmt
    }

    pub fn program(&mut self) -> Vec<Stmt> {
        self.newline_optional();
        let mut stmts = Vec::new();
        while !self.lexer.peek_token().is_eof() {
            stmts.push(self.statement())
        }

        stmts
    }
}
