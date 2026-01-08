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
    ELSE = 112,
    ELSEIF = 113,
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
    OR = 212,
    AND = 213,
    NOT = 214,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BinaryOp {
    Plus,
    Minus,
    Slash,
    Asterisk,
    And,
    Or,
    Gt,
    Lt,
    GtEq,
    LtEq,
    EqEq,
    NotEq,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
}

impl UnaryOp {
    pub fn text(&self) -> &str {
        match self {
            UnaryOp::Plus => "+",
            UnaryOp::Minus => "-",
            UnaryOp::Not => "!",
        }
    }
}

impl BinaryOp {
    pub fn text(&self) -> &str {
        match self {
            BinaryOp::Plus => "+",
            BinaryOp::Minus => "-",
            BinaryOp::Slash => "/",
            BinaryOp::Asterisk => "*",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::Gt => ">",
            BinaryOp::Lt => "<",
            BinaryOp::GtEq => ">=",
            BinaryOp::LtEq => "<=",
            BinaryOp::EqEq => "==",
            BinaryOp::NotEq => "!=",
        }
    }
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
            NOT => "!",
            AND => "&&",
            OR => "||",
            other => panic!("Internal error, no text for {:?}", other),
        }
    }

    pub fn binary_op(&self) -> Option<BinaryOp> {
        match self {
            Token::PLUS => Some(BinaryOp::Plus),
            Token::MINUS => Some(BinaryOp::Minus),
            Token::SLASH => Some(BinaryOp::Slash),
            Token::ASTERISK => Some(BinaryOp::Asterisk),
            Token::EQEQ => Some(BinaryOp::EqEq),
            Token::NOTEQ => Some(BinaryOp::NotEq),
            Token::LT => Some(BinaryOp::Lt),
            Token::GT => Some(BinaryOp::Gt),
            Token::LTEQ => Some(BinaryOp::LtEq),
            Token::GTEQ => Some(BinaryOp::GtEq),
            Token::OR => Some(BinaryOp::Or),
            Token::AND => Some(BinaryOp::And),
            _ => None,
        }
    }

    pub fn unary_op(&self) -> Option<UnaryOp> {
        match self {
            Token::PLUS => Some(UnaryOp::Plus),
            Token::MINUS => Some(UnaryOp::Minus),
            Token::NOT => Some(UnaryOp::Not),
            _ => None,
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
            "ELSE" => Some(Token::ELSE),
            "ELSEIF" => Some(Token::ELSEIF),
            _ => None,
        }
    }

    pub fn try_keyword_or_ident(s: String) -> Self {
        Token::try_keyword(&s).unwrap_or(Token::IDENT(s))
    }
}
