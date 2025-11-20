use std::iter::Peekable;
use std::ops::{Deref, DerefMut};
use std::str::Chars;

#[repr(isize)]
#[derive(Debug, PartialEq)]
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

    fn try_keyword(s: &str) -> Option<Self> {
        match s {
            "LABEL" => Some(Token::LABEL),
            "GOTO" => Some(Token::GOTO),
            "PRINT" => Some(Token::PRINT),
            "INPUT" => Some(Token::INPUT),
            "LET" => Some(Token::LET),
            "IF" => Some(Token::IF),
            "THEN" => Some(Token::THEN),
            "ENDIF" => Some(Token::ENDIF),
            "WHILE" => Some(Token::WHILE),
            "REPEAT" => Some(Token::REPEAT),
            "ENDWHILE" => Some(Token::ENDWHILE),
            _ => None,
        }
    }

    fn try_keyword_or_ident(s: String) -> Self {
        Token::try_keyword(&s).unwrap_or(Token::IDENT(s))
    }
}

// we need a pattern of "build up a token, continue getting chars until you have a token"
#[derive(PartialEq, Debug)]
enum State {
    Started,
    InString,
    InNumeric(bool),
    InAlpha,
    InOperator(StartOperator),
    Finished(Token),
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum StartOperator {
    EQ,
    GT,
    LT,
}

impl StartOperator {
    fn into_token_with_eq(self) -> Token {
        match self {
            StartOperator::EQ => Token::EQEQ,
            StartOperator::GT => Token::GTEQ,
            StartOperator::LT => Token::LTEQ,
        }
    }

    fn into_token(self) -> Token {
        match self {
            StartOperator::EQ => Token::EQ,
            StartOperator::GT => Token::GT,
            StartOperator::LT => Token::LT,
        }
    }
}

pub struct Lexer<'s>(Peekable<Chars<'s>>);

impl<'s> Lexer<'s> {
    pub fn new(s: &'s str) -> Lexer<'s> {
        Lexer(s.chars().peekable())
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.0.peek(), Some(' ') | Some('\t') | Some('\r')) {
            self.0.next();
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.0.peek().copied()
    }

    fn next_char(&mut self) -> Option<char> {
        self.0.next()
    }

    // all remaining tokens are basically strings of text, delimited by newlines or spaces, that
    // need to be categorized
    pub fn next_token(&mut self) -> Token {
        let mut token_string = String::new();
        let mut state = State::Started;

        loop {
            state = match state {
                State::Finished(token) => return token,
                State::Started => {
                    self.skip_whitespace();
                    match self.next_char() {
                        None => State::Finished(Token::EOF),
                        Some(ch) => {
                            token_string.push(ch);
                            match ch {
                                '+' => State::Finished(Token::PLUS),
                                '-' => State::Finished(Token::MINUS),
                                '*' => State::Finished(Token::ASTERISK),
                                '/' => State::Finished(Token::SLASH),
                                '\n' => State::Finished(Token::NEWLINE),
                                '=' => State::InOperator(StartOperator::EQ),
                                '>' => State::InOperator(StartOperator::GT),
                                '<' => State::InOperator(StartOperator::LT),
                                '"' => State::InString,
                                other if other == '-' || other.is_digit(10) => {
                                    State::InNumeric(false)
                                }
                                other if other.is_alphabetic() => State::InAlpha,
                                other => panic!("Lexer error. Unknown start to token: {}", other),
                            }
                        }
                    }
                }
                State::InString => match self.next_char() {
                    None => panic!("Lexer error. File finished with open quote"),
                    Some(ch) => {
                        token_string.push(ch);
                        match ch {
                            '"' => State::Finished(Token::STRING(token_string.clone())),
                            other => State::InString,
                        }
                    }
                },
                State::InOperator(start_operator) => match self.peek() {
                    None => State::Finished(start_operator.into_token()),
                    Some(ch) => match ch {
                        '=' => {
                            self.next_char();
                            token_string.push(ch);
                            State::Finished(start_operator.into_token_with_eq())
                        }
                        _ => State::Finished(start_operator.into_token()),
                    },
                },
                State::InNumeric(seen_period) => match self.peek() {
                    None => State::Finished(Token::NUMBER(token_string.clone())),
                    Some(ch) => match ch {
                        '.' => {
                            if seen_period {
                                panic!(
                                    "Lexer error. Already have seen period in numeric: {}",
                                    token_string
                                );
                            }
                            token_string.push(ch);
                            self.next_char();
                            State::InNumeric(true)
                        }
                        d if d.is_digit(10) => {
                            token_string.push(d);
                            self.next_char();
                            State::InNumeric(seen_period)
                        }
                        w if w.is_ascii_whitespace() => {
                            State::Finished(Token::NUMBER(token_string.clone()))
                        }
                        other => {
                            panic!("Lexer error. Invalid character in numeric token: {}", other)
                        }
                    },
                },
                State::InAlpha => match self.peek() {
                    None => State::Finished(Token::try_keyword_or_ident(token_string.clone())),
                    Some(ch) => match ch {
                        '_' => {
                            token_string.push(ch);
                            self.next_char();
                            State::InAlpha
                        }
                        ch if ch.is_alphanumeric() => {
                            token_string.push(ch);
                            self.next_char();
                            State::InAlpha
                        }
                        w if w.is_ascii_whitespace() => {
                            State::Finished(Token::try_keyword_or_ident(token_string.clone()))
                        }
                        other => panic!(
                            "Lexer error. Invalid character in keyword or identifier token: {}",
                            other
                        ),
                    },
                },
            }
        }
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
    let s = r#"
LET X = 442
LET Y = 32.12
LET Z = X + Y
PRINT Z
LET MY_STRING = "WOW"
PRINT MY_STRING 
IF Z > 5 THEN
    PRINT "WORKED"
ENDIF
LET NEG_X = 4.312
        "#;

    let mut lexer = Lexer::new(&s);

    loop {
        let token = lexer.next_token();
        println!("{:?}", token);
        if token.is_eof() {
            break;
        }
    }
}
