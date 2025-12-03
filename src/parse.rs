use crate::lex::Lexer;
use crate::{Emitter, Token};
use std::collections::HashSet;

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

    fn primary(&mut self) {
        match self.lexer.next_token() {
            Token::NUMBER(n) => {
                self.emitter.emit(&n);
            }
            Token::IDENT(symbol) => {
                self.check_symbol(&symbol);
                self.emitter.emit(&symbol);
            }
            Token::OPENPAREN => {
                self.emitter.emit("(");
                self.expression();
                if !matches!(self.lexer.next_token(), Token::CLOSEPAREN) {
                    panic!("Parse error. Missing close paren after expression")
                }
                self.emitter.emit(")");
            }
            other => panic!(
                "Parse error. Expected number or identifier but found {:?}",
                other
            ),
        }
    }

    fn unary_expr(&mut self) {
        if matches!(self.lexer.peek_token(), Token::PLUS | Token::MINUS) {
            self.emitter.emit(self.lexer.next_token().text());
        }
        self.primary()
    }

    fn term_expr(&mut self) {
        self.unary_expr();

        while matches!(self.lexer.peek_token(), Token::SLASH | Token::ASTERISK) {
            self.emitter.emit(self.lexer.next_token().text());
            self.unary_expr();
        }
    }

    fn math_expr(&mut self) {
        self.term_expr();
        while matches!(self.lexer.peek_token(), Token::PLUS | Token::MINUS) {
            self.emitter.emit(self.lexer.next_token().text());
            self.term_expr();
        }
    }

    fn comparison_expr(&mut self) {
        self.math_expr();

        while self.lexer.peek_token().is_comparator() {
            self.emitter.emit(self.lexer.next_token().text());
            self.math_expr();
        }
    }

    fn not_expr(&mut self) {
        if matches!(self.lexer.peek_token(), Token::NOT) {
            self.emitter.emit(self.lexer.next_token().text());
        }
        self.comparison_expr();
    }

    fn and_expr(&mut self) {
        self.not_expr();
        while matches!(self.lexer.peek_token(), Token::AND) {
            self.emitter.emit(self.lexer.next_token().text());
            self.not_expr();
        }
    }

    fn expression(&mut self) {
        self.and_expr();
        while matches!(self.lexer.peek_token(), Token::OR) {
            self.emitter.emit(self.lexer.next_token().text());
            self.and_expr();
        }
    }

    fn if_block(&mut self) {
        self.expression();
        match self.lexer.next_token() {
            Token::THEN => self.emitter.emit_line(") {"),
            other => panic!("Parser error. Expected THEN but found {:?}", other),
        }

        self.newline();

        while !matches!(
            self.lexer.peek_token(),
            Token::ENDIF | Token::ELSEIF | Token::ELSE
        ) {
            self.statement();
        }

        self.emitter.emit_line("}");
    }

    fn statement(&mut self) {
        match self.lexer.next_token() {
            Token::EOF => unreachable!(),
            Token::PRINT => {
                self.emitter.emit("printf(");
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
                self.if_block();

                while matches!(self.lexer.peek_token(), Token::ELSEIF) {
                    self.emitter.emit("else if(");
                    self.lexer.next_token();
                    self.if_block();
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
