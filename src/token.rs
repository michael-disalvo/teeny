#[repr(isize)]
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    EOF = -1,
    NEWLINE = 0,
    NUMBER(String) = 1,
    IDENT(String) = 2,
    STRING(String) = 3,
    OPENPAREN = 4,
    CLOSEPAREN = 5,
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
    pub fn text(&self) -> &str {
        use Token::*;
        match self {
            NUMBER(s) => s,
            IDENT(s) => s,
            STRING(s) => s,
            EQ => "=",
            PLUS => "+",
            MINUS => "-",
            ASTERISK => "*",
            SLASH => "/",
            EQEQ => "==",
            NOTEQ => "!=",
            LT => "<",
            LTEQ => "<=",
            GT => ">",
            GTEQ => ">=",
            _ => "",
        }
    }

    pub fn is_comparator(&self) -> bool {
        matches!(
            self,
            Token::EQEQ | Token::NOTEQ | Token::LT | Token::LTEQ | Token::GT | Token::GTEQ
        )
    }

    pub fn is_eof(&self) -> bool {
        matches!(self, Token::EOF)
    }

    pub fn try_keyword(s: &str) -> Option<Self> {
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

    pub fn try_keyword_or_ident(s: String) -> Self {
        Token::try_keyword(&s).unwrap_or(Token::IDENT(s))
    }
}
