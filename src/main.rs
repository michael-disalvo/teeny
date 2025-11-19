use std::iter::Peekable;
use std::ops::{Deref, DerefMut};
use std::str::Chars;

#[repr(isize)]
#[derive(Debug)]
pub enum Token {
    EOF = -1,
    NEWLINE = 0,
    NUMBER(String) = 1,
    IDENT(String) = 2,
    STRING(String) = 3,
    // Keywords
    LABEL = 101,
    GOTO = 102,
    PRINT = 103,
    INPUT = 104,
    LET = 105,
    IF = 106,
    THEN = 107,
    ENDIF = 108,
    WHILE = 109,
    REPEAT = 110,
    ENDWHILE = 111,
    // Operators
    EQ = 201,
    PLUS = 202,
    MINUS = 203,
    ASTERISK = 204,
    SLASH = 205,
    EQEQ = 206,
    NOTEQ = 207,
    LT = 208,
    LTEQ,
    GT = 210,
    GTEQ = 211,
}

impl Token {
    fn is_eof(&self) -> bool {
        matches!(self, Token::EOF)
    }
}

pub struct Lexer<'s>(Peekable<Chars<'s>>);

impl<'s> Lexer<'s> {
    pub fn new(s: &'s str) -> Lexer<'s> {
        Lexer(s.chars().peekable())
    }

    pub fn skip_whitespace(&mut self) {
        while matches!(self.0.peek(), Some(' ') | Some('\t') | Some('\r')) {
            self.0.next();
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let Some(next_ch) = self.0.next() else {
            return Token::EOF;
        };

        let token = match next_ch {
            '+' => Token::PLUS,
            '-' => Token::MINUS,
            '*' => Token::ASTERISK,
            '/' => Token::SLASH,
            '\n' => Token::NEWLINE,
            '=' => {
                if let Some('=') = self.0.peek() {
                    self.0.next();
                    Token::EQEQ
                } else {
                    Token::EQEQ
                }
            }
            '>' => {
                if let Some('=') = self.0.peek() {
                    self.0.next();
                    Token::GTEQ
                } else {
                    Token::GT
                }
            }
            '<' => {
                if let Some('=') = self.0.peek() {
                    self.0.next();
                    Token::LTEQ
                } else {
                    Token::LT
                }
            }

            ch => panic!("\'{}\': Lexing error. Unknown character", ch),
        };

        token
    }
}

impl<'s> Deref for Lexer<'s> {
    type Target = Peekable<Chars<'s>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'s> DerefMut for Lexer<'s> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn main() {
    let s = "+- */ = == > >= <===".to_string();

    let mut lexer = Lexer::new(&s);

    loop {
        let token = lexer.next_token();
        println!("{:?}", token);
        if token.is_eof() {
            break;
        }
    }
}
