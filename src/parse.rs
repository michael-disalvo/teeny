use crate::Token;
use crate::lex::Lexer;

use crate::token::{BinaryOp, UnaryOp};
use std::io::Read;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Number(String),
    Identifier(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfBranch {
    pub condition: Expr,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub first_branch: IfBranch,
    pub other_branches: Vec<IfBranch>,
    pub else_body: Option<Vec<Stmt>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrintValue {
    Str(String),
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Print(PrintValue),
    If(IfStmt),
    While(WhileStmt),
    Label(String),
    Goto(String),
    Input(String),
    Let(String, Expr),
}

pub struct Parser<R: Read> {
    lexer: Lexer<R>,
}

impl Parser<std::io::Cursor<String>> {
    pub fn from_str(s: &str) -> Self {
        Self {
            lexer: Lexer::new(s),
        }
    }
}

impl<R: Read> Parser<R> {
    pub fn new(lexer: Lexer<R>) -> Self {
        Self { lexer }
    }
    fn newline_optional(&mut self) {
        while *self.lexer.peek_token() == Token::NEWLINE {
            let _ = self.lexer.next_token();
        }
    }

    fn single_newline(&mut self) {
        match self.lexer.next_token() {
            Token::NEWLINE => {}
            other => panic!("Parse error. Expected newline but found {:?}", other),
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
            let _ = self.lexer.next_token();
            let rhs = self.not_expr();
            lhs = Expr::Binary(BinaryOp::And, Box::new(lhs), Box::new(rhs));
        }
        lhs
    }

    fn expression(&mut self) -> Expr {
        let mut lhs = self.and_expr();
        while matches!(self.lexer.peek_token(), Token::OR) {
            let _ = self.lexer.next_token();
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

    pub fn statement(&mut self) -> Stmt {
        self.newline_optional();
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

        self.single_newline();

        stmt
    }

    pub fn program(&mut self) -> Vec<Stmt> {
        self.newline_optional();
        let mut stmts = Vec::new();
        while !self.lexer.peek_token().is_eof() {
            stmts.push(self.statement());
            self.newline_optional();
        }

        stmts
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_string_to_ast() {
        let input = r#"LET X = 5
PRINT X"#;

        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        assert_eq!(
            ast[0],
            Stmt::Let("X".to_string(), Expr::Number("5".to_string())),
        );

        assert_eq!(
            ast[1],
            Stmt::Print(PrintValue::Expr(Expr::Identifier("X".to_string()))),
        );
    }

    #[test]
    fn print_string() {
        let input = "PRINT \"hello world\"\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        assert_eq!(
            ast[0],
            Stmt::Print(PrintValue::Str("hello world".to_string()))
        );
    }

    #[test]
    fn input_statement() {
        let input = "INPUT X\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        assert_eq!(ast[0], Stmt::Input("X".to_string()));
    }

    #[test]
    fn label_and_goto() {
        let input = "LABEL start\nGOTO start\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        assert_eq!(ast[0], Stmt::Label("start".to_string()));
        assert_eq!(ast[1], Stmt::Goto("start".to_string()));
    }

    #[test]
    fn binary_arithmetic() {
        let input = "LET X = 1 + 2\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        assert_eq!(
            ast[0],
            Stmt::Let(
                "X".to_string(),
                Expr::Binary(
                    BinaryOp::Plus,
                    Box::new(Expr::Number("1".to_string())),
                    Box::new(Expr::Number("2".to_string())),
                )
            )
        );
    }

    #[test]
    fn operator_precedence_mul_over_add() {
        // 1 + 2 * 3 should parse as 1 + (2 * 3)
        let input = "LET X = 1 + 2 * 3\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        let expected = Stmt::Let(
            "X".to_string(),
            Expr::Binary(
                BinaryOp::Plus,
                Box::new(Expr::Number("1".to_string())),
                Box::new(Expr::Binary(
                    BinaryOp::Asterisk,
                    Box::new(Expr::Number("2".to_string())),
                    Box::new(Expr::Number("3".to_string())),
                )),
            ),
        );

        assert_eq!(ast[0], expected);
    }

    #[test]
    fn operator_precedence_div_over_sub() {
        // 10 - 6 / 2 should parse as 10 - (6 / 2)
        let input = "LET X = 10 - 6 / 2\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        let expected = Stmt::Let(
            "X".to_string(),
            Expr::Binary(
                BinaryOp::Minus,
                Box::new(Expr::Number("10".to_string())),
                Box::new(Expr::Binary(
                    BinaryOp::Slash,
                    Box::new(Expr::Number("6".to_string())),
                    Box::new(Expr::Number("2".to_string())),
                )),
            ),
        );

        assert_eq!(ast[0], expected);
    }

    #[test]
    fn left_associativity() {
        // 1 - 2 - 3 should parse as (1 - 2) - 3
        let input = "LET X = 1 - 2 - 3\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        let expected = Stmt::Let(
            "X".to_string(),
            Expr::Binary(
                BinaryOp::Minus,
                Box::new(Expr::Binary(
                    BinaryOp::Minus,
                    Box::new(Expr::Number("1".to_string())),
                    Box::new(Expr::Number("2".to_string())),
                )),
                Box::new(Expr::Number("3".to_string())),
            ),
        );

        assert_eq!(ast[0], expected);
    }

    #[test]
    fn parentheses_override_precedence() {
        // (1 + 2) * 3 should parse as (1 + 2) * 3
        let input = "LET X = (1 + 2) * 3\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        let expected = Stmt::Let(
            "X".to_string(),
            Expr::Binary(
                BinaryOp::Asterisk,
                Box::new(Expr::Binary(
                    BinaryOp::Plus,
                    Box::new(Expr::Number("1".to_string())),
                    Box::new(Expr::Number("2".to_string())),
                )),
                Box::new(Expr::Number("3".to_string())),
            ),
        );

        assert_eq!(ast[0], expected);
    }

    #[test]
    fn unary_minus() {
        let input = "LET X = -5\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        assert_eq!(
            ast[0],
            Stmt::Let(
                "X".to_string(),
                Expr::Unary(UnaryOp::Minus, Box::new(Expr::Number("5".to_string())))
            )
        );
    }

    #[test]
    fn unary_plus() {
        let input = "LET X = +5\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        assert_eq!(
            ast[0],
            Stmt::Let(
                "X".to_string(),
                Expr::Unary(UnaryOp::Plus, Box::new(Expr::Number("5".to_string())))
            )
        );
    }

    #[test]
    fn comparison_operators() {
        let cases = [
            ("LET X = 1 == 2\n", BinaryOp::EqEq),
            ("LET X = 1 != 2\n", BinaryOp::NotEq),
            ("LET X = 1 < 2\n", BinaryOp::Lt),
            ("LET X = 1 > 2\n", BinaryOp::Gt),
            ("LET X = 1 <= 2\n", BinaryOp::LtEq),
            ("LET X = 1 >= 2\n", BinaryOp::GtEq),
        ];

        for (input, expected_op) in cases {
            let mut parser = Parser::from_str(input);
            let ast = parser.program();

            assert_eq!(
                ast[0],
                Stmt::Let(
                    "X".to_string(),
                    Expr::Binary(
                        expected_op,
                        Box::new(Expr::Number("1".to_string())),
                        Box::new(Expr::Number("2".to_string())),
                    )
                ),
                "Failed for input: {}",
                input
            );
        }
    }

    #[test]
    fn not_operator() {
        let input = "LET X = ! 1 == 2\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        // NOT applies to the whole comparison: !(1 == 2)
        let expected = Stmt::Let(
            "X".to_string(),
            Expr::Unary(
                UnaryOp::Not,
                Box::new(Expr::Binary(
                    BinaryOp::EqEq,
                    Box::new(Expr::Number("1".to_string())),
                    Box::new(Expr::Number("2".to_string())),
                )),
            ),
        );

        assert_eq!(ast[0], expected);
    }

    #[test]
    fn and_operator() {
        let input = "LET X = 1 && 2\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        assert_eq!(
            ast[0],
            Stmt::Let(
                "X".to_string(),
                Expr::Binary(
                    BinaryOp::And,
                    Box::new(Expr::Number("1".to_string())),
                    Box::new(Expr::Number("2".to_string())),
                )
            )
        );
    }

    #[test]
    fn or_operator() {
        let input = "LET X = 1 || 2\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        assert_eq!(
            ast[0],
            Stmt::Let(
                "X".to_string(),
                Expr::Binary(
                    BinaryOp::Or,
                    Box::new(Expr::Number("1".to_string())),
                    Box::new(Expr::Number("2".to_string())),
                )
            )
        );
    }

    #[test]
    fn and_or_precedence() {
        // 1 || 2 && 3 should parse as 1 || (2 && 3)
        let input = "LET X = 1 || 2 && 3\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        let expected = Stmt::Let(
            "X".to_string(),
            Expr::Binary(
                BinaryOp::Or,
                Box::new(Expr::Number("1".to_string())),
                Box::new(Expr::Binary(
                    BinaryOp::And,
                    Box::new(Expr::Number("2".to_string())),
                    Box::new(Expr::Number("3".to_string())),
                )),
            ),
        );

        assert_eq!(ast[0], expected);
    }

    #[test]
    fn simple_if() {
        let input = "IF 1 THEN\nPRINT 2\nENDIF\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        let expected = Stmt::If(IfStmt {
            first_branch: IfBranch {
                condition: Expr::Number("1".to_string()),
                body: vec![Stmt::Print(PrintValue::Expr(Expr::Number("2".to_string())))],
            },
            other_branches: vec![],
            else_body: None,
        });

        assert_eq!(ast[0], expected);
    }

    #[test]
    fn if_else() {
        let input = "IF 1 THEN\nPRINT 2\nELSE\nPRINT 3\nENDIF\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        let expected = Stmt::If(IfStmt {
            first_branch: IfBranch {
                condition: Expr::Number("1".to_string()),
                body: vec![Stmt::Print(PrintValue::Expr(Expr::Number("2".to_string())))],
            },
            other_branches: vec![],
            else_body: Some(vec![Stmt::Print(PrintValue::Expr(Expr::Number(
                "3".to_string(),
            )))]),
        });

        assert_eq!(ast[0], expected);
    }

    #[test]
    fn if_elseif() {
        let input = "IF 1 THEN\nPRINT 2\nELSEIF 3 THEN\nPRINT 4\nENDIF\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        let expected = Stmt::If(IfStmt {
            first_branch: IfBranch {
                condition: Expr::Number("1".to_string()),
                body: vec![Stmt::Print(PrintValue::Expr(Expr::Number("2".to_string())))],
            },
            other_branches: vec![IfBranch {
                condition: Expr::Number("3".to_string()),
                body: vec![Stmt::Print(PrintValue::Expr(Expr::Number("4".to_string())))],
            }],
            else_body: None,
        });

        assert_eq!(ast[0], expected);
    }

    #[test]
    fn if_elseif_else() {
        let input = "IF 1 THEN\nPRINT 2\nELSEIF 3 THEN\nPRINT 4\nELSE\nPRINT 5\nENDIF\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        let expected = Stmt::If(IfStmt {
            first_branch: IfBranch {
                condition: Expr::Number("1".to_string()),
                body: vec![Stmt::Print(PrintValue::Expr(Expr::Number("2".to_string())))],
            },
            other_branches: vec![IfBranch {
                condition: Expr::Number("3".to_string()),
                body: vec![Stmt::Print(PrintValue::Expr(Expr::Number("4".to_string())))],
            }],
            else_body: Some(vec![Stmt::Print(PrintValue::Expr(Expr::Number(
                "5".to_string(),
            )))]),
        });

        assert_eq!(ast[0], expected);
    }

    #[test]
    fn multiple_elseif() {
        let input = "IF 1 THEN\nPRINT 1\nELSEIF 2 THEN\nPRINT 2\nELSEIF 3 THEN\nPRINT 3\nENDIF\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        if let Stmt::If(if_stmt) = &ast[0] {
            assert_eq!(if_stmt.other_branches.len(), 2);
        } else {
            panic!("Expected If statement");
        }
    }

    #[test]
    fn while_loop() {
        let input = "WHILE 1 REPEAT\nPRINT 2\nENDWHILE\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        let expected = Stmt::While(WhileStmt {
            condition: Expr::Number("1".to_string()),
            body: vec![Stmt::Print(PrintValue::Expr(Expr::Number("2".to_string())))],
        });

        assert_eq!(ast[0], expected);
    }

    #[test]
    fn nested_if_in_while() {
        let input = "WHILE 1 REPEAT\nIF 2 THEN\nPRINT 3\nENDIF\nENDWHILE\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        if let Stmt::While(while_stmt) = &ast[0] {
            assert_eq!(while_stmt.body.len(), 1);
            assert!(matches!(while_stmt.body[0], Stmt::If(_)));
        } else {
            panic!("Expected While statement");
        }
    }

    #[test]
    fn nested_while_in_if() {
        let input = "IF 1 THEN\nWHILE 2 REPEAT\nPRINT 3\nENDWHILE\nENDIF\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        if let Stmt::If(if_stmt) = &ast[0] {
            assert_eq!(if_stmt.first_branch.body.len(), 1);
            assert!(matches!(if_stmt.first_branch.body[0], Stmt::While(_)));
        } else {
            panic!("Expected If statement");
        }
    }

    #[test]
    fn multiple_newlines_between_statements() {
        let input = "\n\n\nLET X = 1\n\n\nPRINT X\n\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        assert_eq!(ast.len(), 2);
    }

    #[test]
    fn complex_expression() {
        // (A + B) * C - D / E
        let input = "LET X = (A + B) * C - D / E\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        // Should parse as ((A + B) * C) - (D / E)
        if let Stmt::Let(_, expr) = &ast[0] {
            if let Expr::Binary(op, _, _) = expr {
                assert_eq!(*op, BinaryOp::Minus);
            } else {
                panic!("Expected Binary at top level");
            }
        } else {
            panic!("Expected Let statement");
        }
    }

    #[test]
    fn empty_program() {
        let input = "";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        assert_eq!(ast.len(), 0);
    }

    #[test]
    fn only_newlines() {
        let input = "\n\n\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        assert_eq!(ast.len(), 0);
    }

    #[test]
    fn expression_with_identifier() {
        let input = "LET Y = X + 1\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        assert_eq!(
            ast[0],
            Stmt::Let(
                "Y".to_string(),
                Expr::Binary(
                    BinaryOp::Plus,
                    Box::new(Expr::Identifier("X".to_string())),
                    Box::new(Expr::Number("1".to_string())),
                )
            )
        );
    }

    #[test]
    fn while_with_comparison() {
        let input = "WHILE X < 10 REPEAT\nLET X = X + 1\nENDWHILE\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        if let Stmt::While(while_stmt) = &ast[0] {
            assert!(matches!(
                while_stmt.condition,
                Expr::Binary(BinaryOp::Lt, _, _)
            ));
        } else {
            panic!("Expected While statement");
        }
    }

    #[test]
    fn if_with_boolean_expression() {
        let input = "IF X > 0 && X < 10 THEN\nPRINT X\nENDIF\n";
        let mut parser = Parser::from_str(input);
        let ast = parser.program();

        if let Stmt::If(if_stmt) = &ast[0] {
            // Top level should be AND
            assert!(matches!(
                if_stmt.first_branch.condition,
                Expr::Binary(BinaryOp::And, _, _)
            ));
        } else {
            panic!("Expected If statement");
        }
    }
}
