use crate::lex::Lexer;
use crate::{Emitter, Token};
use std::collections::HashSet;

use crate::token::{BinaryOp, UnaryOp};

pub enum Expr {
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Number(String),
    Identifier(String),
}

pub struct IfBranch {
    pub condition: Expr,
    pub body: Vec<Stmt>,
}

pub struct IfStmt {
    pub first_branch: IfBranch,
    pub other_branches: Vec<IfBranch>,
    pub else_body: Option<Vec<Stmt>>,
}

pub struct WhileStmt {
    pub condition: Expr,
    pub body: Vec<Stmt>,
}

pub enum PrintValue {
    Str(String),
    Expr(Expr),
}

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
    pub emitter: Emitter,
    symbols: HashSet<String>,
    labels_declared: HashSet<String>,
    labels_gotoed: HashSet<String>,
}

impl Parser {
    pub fn new(lexer: Lexer, emitter: Emitter) -> Self {
        Self {
            lexer,
            emitter,
            symbols: Default::default(),
            labels_declared: Default::default(),
            labels_gotoed: Default::default(),
        }
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

    fn check_symbol(&mut self, symbol: &str) {
        if !self.symbols.contains(symbol) {
            panic!("Parse error. Unknown symbol \"{}\"", symbol);
        }
    }

    fn primary(&mut self) -> Expr {
        match self.lexer.next_token() {
            Token::NUMBER(n) => Expr::Number(n),
            Token::IDENT(s) => Expr::Identifier(s),
            Token::OPENPAREN => {
                let expr = self.expression();
                if !matches!(self.lexer.next_tokne(), Token::CLOSEPAREN) {
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
        if matches!(self.lexer.peek_token(), Token::PLUS | Tokne::MINUS) {
            let unary_op = match self.lexer.next_tokne() {
                Token::PLUS => UnaryOp::Plus,
                Token::MINUS => UnaryOp::Minus,
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
            let binary_op = self.lexer.next_token() {
                Token::SLASH => BinaryOp::Slash,
                Tokne::ASTERISK => BinaryOp::Asterisk,
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
            let binary_op = self.lexer.next_token() {
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
            let binary_op = self.lexer.next_token().binary_op().expect("checked for comparator");
            let rhs = self.math_expr();
            lhs = Expr::Binary(binary_op, Box::new(lhs), Box::new(rhs)); 
        }
        lhs
    }

    fn not_expr(&mut self) -> Expr {
        if matches!(self.lexer.peek_token(), Token::NOT) {
            let unary_op = UnaryOp::Not;
            let _ = self.lexer.next_token();
            Expr::Unary(unary_op, Box::new(self.comparison_expr()));
        } else {
            self.comparison_expr()
        }
    }

    fn and_expr(&mut self) -> Expr {
        let mut lhs = self.not_expr();
        while matches!(self.lexer.peek_token(), Token::AND) {
            let rhs = self.not_expr();
            lhs = Expr::Binary(binary_op, Box::new(lhs), Box::new(rhs));
        }
        lhs
    }

    fn expression(&mut self) -> Expr {
        let mut lhs = self.and_expr();
        while matches!(self.lexer.peek_token(), Token::OR) {
            let rhs = self.and_expr();
            lhs = Expr::Binary(binary_op, Box::new(lhs), Box::new(rhs));
        }
        lhs
    }

    fn if_branch(&mut self) -> IfBranch {
        let condition = self.expression();

        if !matches!(self.lexer.next_token(), Token::THEN) {
            other => panic!("Parser error. Expected THEN but found {:?}", other);
        }

        self.newline();

        let mut body = Vec::new();
        while !matches!(
            self.lexer.peek_token(),
            Token::ENDIF | Token::ELSEIF | Token::ELSE,
        ) {
            body.push(self.statement());
        }

        IfBranch {
            condition,
            body,
        }
    }

    fn statement(&mut self) {
        match self.lexer.next_token() {
            Token::EOF => unreachable!(),
            Token::PRINT => {
                match self.lexer.peek_token() {
                    Token::STRING(s) => {
                        self.emitter.emit(format!("\"{}\\n\"", s).as_str());
                        let _ = self.lexer.next_token();
                    }
                    _ => {
                        self.emitter.emit(r#""%.2f\n", (float)"#);
                        self.expression();
                    }
                }
                self.emitter.emit_line(");");
            }
            Token::IF => {
                self.emitter.emit("if(");

                self.if_branch();

                while matches!(self.lexer.peek_token(), Token::ELSEIF) {
                    self.emitter.emit("else if(");
                    self.lexer.next_token();
                    self.if_branch();
                }

                if matches!(self.lexer.peek_token(), Token::ELSE) {
                    self.lexer.next_token();
                    self.newline();
                    self.emitter.emit_line("else {");
                    while !matches!(self.lexer.peek_token(), Token::ENDIF) {
                        self.statement();
                    }
                }

                self.lexer.next_token();
                self.emitter.emit_line("}");
            }
            Token::WHILE => {
                self.emitter.emit("while (");
                self.expression();
                match self.lexer.next_token() {
                    Token::REPEAT => self.emitter.emit_line(") {"),
                    other => panic!("Parser error. Expected REPEAT but found {:?}", other),
                }
                self.newline();
                while *self.lexer.peek_token() != Token::ENDWHILE {
                    self.statement();
                }
                self.emitter.emit_line("}");
                self.lexer.next_token();
            }
            Token::LABEL => match self.lexer.next_token() {
                Token::IDENT(label_declared) => {
                    self.emitter
                        .emit_line(format!("{}: ", label_declared).as_str());
                    self.labels_declared.insert(label_declared);
                }
                other => panic!("Parser error. Expected identifier but found {:?}", other),
            },
            Token::GOTO => match self.lexer.next_token() {
                Token::IDENT(label_gotoed) => {
                    self.emitter
                        .emit_line(format!("goto {};", label_gotoed).as_str());
                    self.labels_gotoed.insert(label_gotoed);
                }
                other => panic!("Parser error. Expected identifier but found {:?}", other),
            },
            Token::INPUT => match self.lexer.next_token() {
                Token::IDENT(symbol) => {
                    if self.symbols.insert(symbol.clone()) {
                        self.emitter
                            .emit_line(format!("float {};", symbol).as_str());
                    }
                    self.emitter
                        .emit_line(format!("if(0 == scanf(\"%f\", &{})) {{", symbol).as_str());
                    self.emitter.emit_line(format!("{} = 0;", symbol).as_str());
                    self.emitter.emit_line(r#"scanf("%*s");"#);
                    self.emitter.emit_line("}");
                }
                other => panic!("Parser error. Expected identifier but found {:?}", other),
            },
            Token::LET => {
                match self.lexer.next_token() {
                    Token::IDENT(symbol) => {
                        if self.symbols.insert(symbol.clone()) {
                            self.emitter.emit("float ");
                        }
                        self.emitter.emit(&symbol);
                    }
                    other => panic!("Parser error. Expected identifier but found {:?}", other),
                }
                match self.lexer.next_token() {
                    Token::EQ => self.emitter.emit("="),
                    other => panic!("Parser error. Expected `=` but found {:?}", other),
                }
                self.expression();
                self.emitter.emit_line(";");
            }
            other => panic!(
                "Parser error. Expected start of statement but found {:?}",
                other
            ),
        }

        self.newline();
    }

    pub fn program(&mut self) {
        self.newline_optional();
        while !self.lexer.peek_token().is_eof() {
            self.statement()
        }

        for label_gotoed in &self.labels_gotoed {
            if !self.labels_declared.contains(label_gotoed) {
                panic!("Parser error. Unknown label gotoed: \"{}\"", label_gotoed);
            }
        }
    }
}
