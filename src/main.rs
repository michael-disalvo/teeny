use std::iter::Peekable;

struct IntoChars {
    string: String,
    pos: usize,
}

impl IntoChars {
    fn new(string: String) -> Self {
        let string = string.to_string();
        Self { string, pos: 0 }
    }
}

impl std::iter::Iterator for IntoChars {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        let c = self.string[self.pos..].chars().next()?;
        self.pos += c.len_utf8();
        Some(c)
    }
}

#[repr(isize)]
#[derive(Debug, PartialEq, Clone)]
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
    fn is_comparator(&self) -> bool {
        matches!(
            self,
            Token::EQEQ | Token::NOTEQ | Token::LT | Token::LTEQ | Token::GT | Token::GTEQ
        )
    }

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

fn newline_optional(lexer: &mut Lexer) {
    while *lexer.peek_token() == Token::NEWLINE {
        let _ = lexer.next_token();
    }
}

fn newline(lexer: &mut Lexer) {
    println!("NEWLINE");
    match lexer.next_token() {
        Token::NEWLINE => newline_optional(lexer),
        other => panic!("Parse error. Expected newline but found {:?}", other),
    }
}

fn primary(lexer: &mut Lexer) {
    println!("PRIMARY");
    match lexer.next_token() {
        Token::NUMBER(..) => {}
        Token::IDENT(..) => {}
        other => panic!(
            "Parse error. Expected number or identifier but found {:?}",
            other
        ),
    }
}

fn unary(lexer: &mut Lexer) {
    println!("UNARY");
    match lexer.peek_token() {
        Token::PLUS | Token::MINUS => {
            let _ = lexer.next_token();
        }
        _ => {}
    }
    primary(lexer)
}

fn term(lexer: &mut Lexer) {
    println!("TERM");

    unary(lexer);

    while matches!(lexer.peek_token(), Token::SLASH | Token::ASTERISK) {
        lexer.next_token();
        unary(lexer);
    }
}

fn expression(lexer: &mut Lexer) {
    println!("EXPRESSION");

    term(lexer);
    while matches!(lexer.peek_token(), Token::PLUS | Token::MINUS) {
        lexer.next_token();
        term(lexer);
    }
}

fn comparison(lexer: &mut Lexer) {
    println!("COMPARISON");
    expression(lexer);

    if !lexer.peek_token().is_comparator() {
        panic!(
            "Parse error. Expected comparator, found {:?}",
            lexer.peek_token()
        );
    }

    while lexer.peek_token().is_comparator() {
        lexer.next_token();
        expression(lexer);
    }
}

fn statement(lexer: &mut Lexer) {
    println!("STATEMENT");
    match lexer.next_token() {
        Token::EOF => unreachable!(),
        Token::PRINT => match lexer.peek_token() {
            &Token::STRING(..) => {
                let _ = lexer.next_token();
            }
            _ => expression(lexer),
        },
        Token::IF => {
            comparison(lexer);
            match lexer.next_token() {
                Token::THEN => {}
                other => panic!("Parser error. Expected THEN but found {:?}", other),
            }

            newline(lexer);
            while *lexer.peek_token() != Token::ENDIF {
                statement(lexer);
            }
            lexer.next_token();
        }
        Token::WHILE => {
            comparison(lexer);
            match lexer.next_token() {
                Token::REPEAT => {}
                other => panic!("Parser error. Expected REPEAT but found {:?}", other),
            }
            newline(lexer);
            while *lexer.peek_token() != Token::ENDWHILE {
                statement(lexer);
            }
            lexer.next_token();
        }
        Token::LABEL | Token::GOTO | Token::INPUT => match lexer.next_token() {
            Token::IDENT(..) => {}
            other => panic!("Parser error. Expected identifier but found {:?}", other),
        },
        Token::LET => {
            match lexer.next_token() {
                Token::IDENT(..) => {}
                other => panic!("Parser error. Expected identifier but found {:?}", other),
            }
            match lexer.next_token() {
                Token::EQ => {}
                other => panic!("Parser error. Expected `=` but found {:?}", other),
            }
            expression(lexer);
        }
        other => panic!(
            "Parser error. Expected start of statement but found {:?}",
            other
        ),
    }

    newline(lexer);
}

fn program(mut lexer: Lexer) {
    println!("PROGRAM");
    newline_optional(&mut lexer);
    while !lexer.peek_token().is_eof() {
        statement(&mut lexer)
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

fn is_operator(ch: char) -> bool {
    matches!(ch, '+' | '-' | '*' | '/' | '=' | '>' | '<')
}

pub struct TokenIter {
    lexer: Lexer,
    seen_eof: bool,
}

impl std::iter::Iterator for TokenIter {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.seen_eof {
            None
        } else {
            let t = self.lexer.next_token();
            self.seen_eof = t.is_eof();
            Some(t)
        }
    }
}

pub struct Lexer {
    inner: Peekable<IntoChars>,
    peeked: Option<Token>,
}

impl Lexer {
    pub fn new(s: &str) -> Lexer {
        let s = s.to_string() + "\n";
        Lexer {
            inner: IntoChars::new(s).peekable(),
            peeked: None,
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.inner.peek(), Some(' ') | Some('\t') | Some('\r')) {
            self.inner.next();
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        self.inner.peek().copied()
    }

    fn next_char(&mut self) -> Option<char> {
        self.inner.next()
    }

    pub fn tokens(self) -> TokenIter {
        TokenIter {
            lexer: self,
            seen_eof: false,
        }
    }

    pub fn peek_token(&mut self) -> &Token {
        if let None = self.peeked {
            self.peeked = Some(self.next_token())
        }

        // SAFETY: a `None` variant for `self` would have been replaced by a `Some`
        // variant in the code above.
        unsafe { self.peeked.as_mut().unwrap_unchecked() }
    }

    // all remaining tokens are basically strings of text, delimited by newlines or spaces, that
    // need to be categorized
    pub fn next_token(&mut self) -> Token {
        if let Some(v) = self.peeked.take() {
            return v;
        }

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
                                other if other.is_digit(10) => State::InNumeric(false),
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
                            _ => State::InString,
                        }
                    }
                },
                State::InOperator(start_operator) => match self.peek_char() {
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
                State::InNumeric(seen_period) => match self.peek_char() {
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
                        ch if !ch.is_alphabetic() => {
                            State::Finished(Token::NUMBER(token_string.clone()))
                        }
                        other => {
                            panic!("Lexer error. Invalid character in numeric token: {}", other)
                        }
                    },
                },
                State::InAlpha => match self.peek_char() {
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
                        ch if is_operator(ch) => {
                            State::Finished(Token::try_keyword_or_ident(token_string.clone()))
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

fn main() {
    let s = r#"
LET X = 3
PRINT X
IF X >= 2 THEN
    LET Y = 5
    WHILE Y*2 > 3 REPEAT 
        PRINT Y
        LET Y = Y - 1
    ENDWHILE 
    PRINT "LARGE"
ENDIF"#;

    let lexer = Lexer::new(&s);
    program(lexer);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn test_invalid_char_in_alpha_token() {
        let input = r#"LET MY_FUN_👍_VAR = 3.23"#;
        let _: Vec<_> = Lexer::new(input).tokens().collect();
    }

    #[test]
    #[should_panic]
    fn test_invalid_alpha_in_number() {
        let input = r#"LET X = 3.23A"#;
        let _: Vec<_> = Lexer::new(input).tokens().collect();
    }

    #[test]
    #[should_panic]
    fn test_invalid_period_start_token() {
        let input = r#"LET X = .3"#;
        let _: Vec<_> = Lexer::new(input).tokens().collect();
    }

    #[test]
    #[should_panic]
    fn test_double_period_in_number() {
        let input = r#"LET X = 4.3.2"#;
        let _: Vec<_> = Lexer::new(input).tokens().collect();
    }

    #[test]
    #[should_panic]
    fn test_missing_string_end() {
        let input = r#"LET S = "this does not end"#;
        let _: Vec<_> = Lexer::new(input).tokens().collect();
    }

    #[test]
    #[should_panic]
    fn test_unknown_token_start() {
        let input = r#"LET 👍 = 3"#;
        let _: Vec<_> = Lexer::new(input).tokens().collect();
    }

    #[test]
    fn test_edge_case_operator_delimited_things() {
        let inputs = vec![r#"LET X = 3+4"#, r#"LET X=Y>=3"#];

        let answers = {
            use Token::*;
            vec![
                vec![
                    LET,
                    IDENT("X".to_string()),
                    EQ,
                    NUMBER("3".to_string()),
                    PLUS,
                    NUMBER("4".to_string()),
                    EOF,
                ],
                vec![
                    LET,
                    IDENT("X".to_string()),
                    EQ,
                    IDENT("Y".to_string()),
                    GTEQ,
                    NUMBER("3".to_string()),
                    EOF,
                ],
            ]
        };

        for (input, expected) in std::iter::zip(inputs, answers) {
            let tokens: Vec<_> = Lexer::new(input).tokens().collect();
            assert_eq!(tokens, expected);
        }
    }

    #[test]
    fn test_lexing_success() {
        let inputs = vec![
            r#"LET XΔ = 6"#,
            r#"LET X = -4.6
LET Y2 = 3.2
LET X_PLUS_Y = X + Y
IF X_PLUS_Y >= 3 THEN
    PRINT "WORKED"
ENDIF"#,
        ];

        let answers = {
            use Token::*;
            vec![
                vec![
                    LET,
                    IDENT("XΔ".to_string()),
                    EQ,
                    NUMBER("6".to_string()),
                    EOF,
                ],
                vec![
                    LET,
                    IDENT("X".to_string()),
                    EQ,
                    MINUS,
                    NUMBER("4.6".to_string()),
                    NEWLINE,
                    LET,
                    IDENT("Y2".to_string()),
                    EQ,
                    NUMBER("3.2".to_string()),
                    NEWLINE,
                    LET,
                    IDENT("X_PLUS_Y".to_string()),
                    EQ,
                    IDENT("X".to_string()),
                    PLUS,
                    IDENT("Y".to_string()),
                    NEWLINE,
                    IF,
                    IDENT("X_PLUS_Y".to_string()),
                    GTEQ,
                    NUMBER("3".to_string()),
                    THEN,
                    NEWLINE,
                    PRINT,
                    STRING("\"WORKED\"".to_string()),
                    NEWLINE,
                    ENDIF,
                    EOF,
                ],
            ]
        };

        for (input, expected) in std::iter::zip(inputs, answers) {
            let tokens: Vec<_> = Lexer::new(input).tokens().collect();
            assert_eq!(tokens, expected);
        }
    }
}
