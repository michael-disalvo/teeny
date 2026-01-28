use crate::Token;
use std::io::{BufRead, BufReader, Read};
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
    NOT,
}

impl StartOperator {
    fn into_token_with_eq(self) -> Token {
        match self {
            StartOperator::EQ => Token::EQEQ,
            StartOperator::GT => Token::GTEQ,
            StartOperator::LT => Token::LTEQ,
            StartOperator::NOT => Token::NOTEQ,
        }
    }

    fn into_token(self) -> Token {
        match self {
            StartOperator::EQ => Token::EQ,
            StartOperator::GT => Token::GT,
            StartOperator::LT => Token::LT,
            StartOperator::NOT => Token::NOT,
        }
    }
}

fn can_delimit_identifier(ch: char) -> bool {
    matches!(ch, '+' | '-' | '*' | '/' | '=' | '>' | '<' | ')')
}

#[allow(dead_code)]
pub struct TokenIter<R> {
    lexer: Lexer<R>,
    seen_eof: bool,
}

impl<R: Read> std::iter::Iterator for TokenIter<R> {
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

pub struct Lexer<R> {
    inner: BufReader<R>,
    current_line: Option<Peekable<IntoChars>>,
    peeked: Option<Token>,
}

impl Lexer<std::io::Cursor<String>> {
    pub fn new(s: &str) -> Self {
        Self {
            inner: BufReader::new(std::io::Cursor::new(s.to_string())),
            current_line: None,
            peeked: None,
        }
    }
}

impl<R: Read> Lexer<R> {
    #[cfg(test)]
    pub fn tokens(self) -> TokenIter<R> {
        TokenIter {
            lexer: self,
            seen_eof: false,
        }
    }

    pub fn from_reader(rdr: R) -> Self {
        let inner = BufReader::new(rdr);
        Self {
            inner,
            current_line: None,
            peeked: None,
        }
    }

    fn next_line(&mut self) -> Option<String> {
        let mut line = String::new();
        match self.inner.read_line(&mut line) {
            Ok(0) => None,
            Ok(_) => {
                if !line.ends_with('\n') {
                    line.push('\n');
                }
                Some(line)
            }
            Err(e) => panic!("IO Error: {e:?}"),
        }
    }

    fn next_char(&mut self) -> Option<char> {
        loop {
            if let Some(current_line) = &mut self.current_line {
                match current_line.next() {
                    Some(ch) => return Some(ch),
                    None => self.current_line = None,
                }
            }

            match self.next_line() {
                Some(line) => self.current_line = Some(IntoChars::new(line).peekable()),
                None => return None,
            }
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        loop {
            if let Some(current_line) = &mut self.current_line {
                match current_line.peek() {
                    Some(ch) => return Some(*ch),
                    None => self.current_line = None,
                }
            }

            match self.next_line() {
                Some(line) => self.current_line = Some(IntoChars::new(line).peekable()),
                None => return None,
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_char(), Some(' ') | Some('\t') | Some('\r')) {
            let _ = self.next_char();
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
                                '(' => State::Finished(Token::OPENPAREN),
                                ')' => State::Finished(Token::CLOSEPAREN),
                                '\n' => State::Finished(Token::NEWLINE),
                                start if matches!(start, '&' | '|') => {
                                    let next_ch = self.next_char();
                                    if next_ch.is_none_or(|ch| ch != start) {
                                        panic!(
                                            "Lexer error. Saw {:?}, expected another {}",
                                            next_ch, start
                                        )
                                    }
                                    let token = if start == '&' { Token::AND } else { Token::OR };
                                    State::Finished(token)
                                }
                                '=' => State::InOperator(StartOperator::EQ),
                                '>' => State::InOperator(StartOperator::GT),
                                '<' => State::InOperator(StartOperator::LT),
                                '!' => State::InOperator(StartOperator::NOT),
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
                            '"' => {
                                let string = token_string.trim_matches('"').to_owned();
                                State::Finished(Token::STRING(string))
                            }
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
                        ch if can_delimit_identifier(ch) => {
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
                    NEWLINE,
                    EOF,
                ],
                vec![
                    LET,
                    IDENT("X".to_string()),
                    EQ,
                    IDENT("Y".to_string()),
                    GTEQ,
                    NUMBER("3".to_string()),
                    NEWLINE,
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
            "IF THEN ELSEIF ELSE",
            "IF THEN ELSE",
            r#"!(X && Y) || Z && !A"#,
            r#"LET Y = 5 * (3 + 2)"#,
            r#"IF X != 5"#,
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
                vec![IF, THEN, ELSEIF, ELSE, NEWLINE, EOF],
                vec![IF, THEN, ELSE, NEWLINE, EOF],
                vec![
                    NOT,
                    OPENPAREN,
                    IDENT("X".to_string()),
                    AND,
                    IDENT("Y".to_string()),
                    CLOSEPAREN,
                    OR,
                    IDENT("Z".to_string()),
                    AND,
                    NOT,
                    IDENT("A".to_string()),
                    NEWLINE,
                    EOF,
                ],
                vec![
                    LET,
                    IDENT("Y".to_string()),
                    EQ,
                    NUMBER("5".to_string()),
                    ASTERISK,
                    OPENPAREN,
                    NUMBER("3".to_string()),
                    PLUS,
                    NUMBER("2".to_string()),
                    CLOSEPAREN,
                    NEWLINE,
                    EOF,
                ],
                vec![
                    IF,
                    IDENT("X".to_string()),
                    NOTEQ,
                    NUMBER("5".to_string()),
                    NEWLINE,
                    EOF,
                ],
                vec![
                    LET,
                    IDENT("XΔ".to_string()),
                    EQ,
                    NUMBER("6".to_string()),
                    NEWLINE,
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
                    STRING("WORKED".to_string()),
                    NEWLINE,
                    ENDIF,
                    NEWLINE,
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
