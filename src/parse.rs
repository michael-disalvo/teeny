use crate::Token;
use crate::lex::Lexer;
use std::collections::HashSet;

pub struct Parser {
    lexer: Lexer,
    symbols: HashSet<String>,
    labels_declared: HashSet<String>,
    labels_gotoed: HashSet<String>,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self {
            lexer,
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
        println!("NEWLINE");
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
        println!("PRIMARY");
        match self.lexer.next_token() {
            Token::NUMBER(..) => {}
            Token::IDENT(symbol) => self.check_symbol(&symbol),
            other => panic!(
                "Parse error. Expected number or identifier but found {:?}",
                other
            ),
        }
    }

    fn unary(&mut self) {
        println!("UNARY");
        match self.lexer.peek_token() {
            Token::PLUS | Token::MINUS => {
                let _ = self.lexer.next_token();
            }
            _ => {}
        }
        self.primary()
    }

    fn term(&mut self) {
        println!("TERM");

        self.unary();

        while matches!(self.lexer.peek_token(), Token::SLASH | Token::ASTERISK) {
            self.lexer.next_token();
            self.unary();
        }
    }

    fn expression(&mut self) {
        println!("EXPRESSION");

        self.term();
        while matches!(self.lexer.peek_token(), Token::PLUS | Token::MINUS) {
            self.lexer.next_token();
            self.term();
        }
    }

    fn comparison(&mut self) {
        println!("COMPARISON");
        self.expression();

        if !self.lexer.peek_token().is_comparator() {
            panic!(
                "Parse error. Expected comparator, found {:?}",
                self.lexer.peek_token()
            );
        }

        while self.lexer.peek_token().is_comparator() {
            self.lexer.next_token();
            self.expression();
        }
    }

    fn statement(&mut self) {
        println!("STATEMENT");
        match self.lexer.next_token() {
            Token::EOF => unreachable!(),
            Token::PRINT => match self.lexer.peek_token() {
                &Token::STRING(..) => {
                    let _ = self.lexer.next_token();
                }
                _ => self.expression(),
            },
            Token::IF => {
                self.comparison();
                match self.lexer.next_token() {
                    Token::THEN => {}
                    other => panic!("Parser error. Expected THEN but found {:?}", other),
                }

                self.newline();
                while *self.lexer.peek_token() != Token::ENDIF {
                    self.statement();
                }
                self.lexer.next_token();
            }
            Token::WHILE => {
                self.comparison();
                match self.lexer.next_token() {
                    Token::REPEAT => {}
                    other => panic!("Parser error. Expected REPEAT but found {:?}", other),
                }
                self.newline();
                while *self.lexer.peek_token() != Token::ENDWHILE {
                    self.statement();
                }
                self.lexer.next_token();
            }
            Token::LABEL => match self.lexer.next_token() {
                Token::IDENT(label_declared) => {
                    self.labels_declared.insert(label_declared);
                }
                other => panic!("Parser error. Expected identifier but found {:?}", other),
            },
            Token::GOTO => match self.lexer.next_token() {
                Token::IDENT(label_gotoed) => {
                    self.labels_gotoed.insert(label_gotoed);
                }
                other => panic!("Parser error. Expected identifier but found {:?}", other),
            },
            Token::INPUT => match self.lexer.next_token() {
                Token::IDENT(symbol) => {
                    self.symbols.insert(symbol);
                }
                other => panic!("Parser error. Expected identifier but found {:?}", other),
            },
            Token::LET => {
                match self.lexer.next_token() {
                    Token::IDENT(symbol) => {
                        self.symbols.insert(symbol);
                    }
                    other => panic!("Parser error. Expected identifier but found {:?}", other),
                }
                match self.lexer.next_token() {
                    Token::EQ => {}
                    other => panic!("Parser error. Expected `=` but found {:?}", other),
                }
                self.expression();
            }
            other => panic!(
                "Parser error. Expected start of statement but found {:?}",
                other
            ),
        }

        self.newline();
    }

    pub fn program(&mut self) {
        println!("PROGRAM");
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
