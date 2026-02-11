use crate::parse::{Expr, IfStmt, PrintValue, Stmt, WhileStmt};

use std::collections::HashMap;
use std::iter;
use std::str::FromStr;

#[derive(Debug, Default)]
pub struct Runtime {
    variables: HashMap<String, f32>,
}

impl Runtime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn condition_is_true(&self, expr: &Expr) -> bool {
        crate::token::to_bool(self.eval_expr(expr))
    }

    pub fn eval_let(&mut self, ident: &str, expr: &Expr) {
        let value = self.eval_expr(expr);
        self.variables.insert(ident.to_string(), value);
    }

    pub fn eval_label(&mut self, _label: &str) {
        panic!("Runtime error: labels not allowed in interpreter");
    }

    pub fn eval_goto(&mut self, _label: &str) {
        panic!("Runtime error: gotos not allowed in interpreter");
    }

    pub fn eval_if(&mut self, if_stmt: &IfStmt) {
        let Some(body) = iter::once(&if_stmt.first_branch)
            .chain(&if_stmt.other_branches)
            .find(|branch| self.condition_is_true(&branch.condition))
            .map(|true_branch| true_branch.body.as_slice())
            .or(if_stmt.else_body.as_deref())
        else {
            return;
        };

        // evaluate stmts
        for stmt in body {
            self.eval_stmt(stmt)
        }
    }

    pub fn eval_while(&mut self, while_stmt: &WhileStmt) {
        while self.condition_is_true(&while_stmt.condition) {
            // evalutate while_stmt.body
            for stmt in &while_stmt.body {
                self.eval_stmt(stmt)
            }
        }
    }

    pub fn eval_print(&self, print_value: &PrintValue) {
        match print_value {
            PrintValue::Str(s) => println!("{}", s),
            PrintValue::Expr(expr) => {
                let f = self.eval_expr(expr);
                println!("{:.2}", f);
            }
        }
    }

    pub fn eval_expr(&self, expr: &Expr) -> f32 {
        match expr {
            Expr::Number(num_str) => {
                f32::from_str(num_str).expect("should have validated number token")
            }
            Expr::Identifier(ident) => {
                let Some(val) = self.variables.get(ident) else {
                    panic!("Runtime error: no value for identifier {}", ident);
                };
                *val
            }
            Expr::Binary(op, lhs, rhs) => {
                let lhs = self.eval_expr(lhs);
                let rhs = self.eval_expr(rhs);
                op.eval(lhs, rhs)
            }
            Expr::Unary(op, expr) => {
                let operand = self.eval_expr(expr);
                op.eval(operand)
            }
        }
    }

    pub fn eval_input(&mut self, ident: &str) {
        use std::io::BufRead;
        let mut line = String::new();
        std::io::stdin().lock().read_line(&mut line).unwrap();

        let input = f32::from_str(line.trim()).unwrap_or(0.0);
        self.variables.insert(ident.to_string(), input);
    }

    pub fn eval_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Print(pv) => self.eval_print(pv),
            Stmt::If(if_stmt) => self.eval_if(if_stmt),
            Stmt::While(while_stmt) => self.eval_while(while_stmt),
            Stmt::Label(label) => self.eval_label(label),
            Stmt::Goto(label) => self.eval_goto(label),
            Stmt::Input(ident) => self.eval_input(ident),
            Stmt::Let(ident, expr) => self.eval_let(ident, expr),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::parse::Parser;
    use crate::token::{BinaryOp, UnaryOp, to_bool};

    macro_rules! assert_feq {
        ($a:expr, $b:expr, $tolerance:expr) => {{
            let diff = ($a - $b).abs();
            assert!(
                diff <= $tolerance,
                "assertion failed: `(left !== right)` \
                 (left: `{:?}`, right: `{:?}`, tolerance: `{:?}`, difference: `{:?}`)",
                $a,
                $b,
                $tolerance,
                diff
            );
        }};
    }

    #[test]
    fn test_eval_stmt() {
        let input = r#"
LET X = 4
LET Y = 0
WHILE X > 0 REPEAT 
    LET Y = Y + 1
    LET X = X - 1
ENDWHILE
"#;

        let ast = Parser::from_str(input).program().unwrap();

        let mut runtime = Runtime::new();

        for stmt in ast {
            runtime.eval_stmt(&stmt);
        }

        assert_feq!(runtime.variables.get("X").unwrap(), 0.0, 0.000001);
        assert_feq!(runtime.variables.get("Y").unwrap(), 4.0, 0.000001);
    }

    #[test]
    fn test_eval_let_stmt() {
        let mut runtime = Runtime::new();

        let input = "LET X = 3 * (5 - 1)";

        let ast = Parser::from_str(input).program().unwrap();

        let Stmt::Let(ident, expr) = &ast[0] else {
            panic!("expected Stmt::Let but found {:?}", ast[0]);
        };

        runtime.eval_let(ident, expr);

        assert_feq!(runtime.variables.get("X").unwrap(), 12.0, 0.00001);
    }

    fn number_expr(f: f32) -> Box<Expr> {
        Box::new(Expr::Number(format!("{}", f)))
    }

    fn ident_expr(s: &str) -> Box<Expr> {
        Box::new(Expr::Identifier(s.to_string()))
    }

    #[test]
    fn test_eval_expr() {
        let runtime = Runtime::new();

        // 1 + 2
        let expr = Expr::Binary(BinaryOp::Plus, number_expr(1.0), number_expr(2.0));
        assert_feq!(runtime.eval_expr(&expr), 3.0, 0.00001);

        // 2 * (3 + 4) = 14
        let expr = Expr::Binary(
            BinaryOp::Asterisk,
            number_expr(2.0),
            Expr::Binary(BinaryOp::Plus, number_expr(3.0), number_expr(4.0)).into(),
        );
        assert_feq!(runtime.eval_expr(&expr), 14.0, 0.00001);

        let expr = Expr::Unary(UnaryOp::Not, number_expr(1.4));
        assert_feq!(runtime.eval_expr(&expr), 0.0, 0.00001);

        let mut runtime = Runtime::new();
        runtime.variables.insert("X".to_string(), 5.0);

        let expr = Expr::Binary(BinaryOp::Slash, ident_expr("X"), ident_expr("X"));
        assert_feq!(runtime.eval_expr(&expr), 1.0, 0.000001);
    }

    #[test]
    fn test_to_bool() {
        let a = 3.0;
        let b = 2.0;

        assert_eq!(to_bool(a - b), true);
        assert_eq!(to_bool(a - b - 1.0), false);
        assert_eq!(to_bool(a - b - 0.5), true);
        assert_eq!(to_bool(a - b - 0.1), true);
        assert_eq!(to_bool(a - b - 0.01), true);
        assert_eq!(to_bool(a - b - 0.001), true);
        assert_eq!(to_bool(a - b - 0.0001), true);
        assert_eq!(to_bool(a - b - 1.1), true);
    }

    #[test]
    fn test_if_false_no_else() {
        let input = r#"
LET Y = 0
IF 0 THEN
    LET Y = 1
ENDIF
"#;
        let ast = Parser::from_str(input).program().unwrap();
        let mut runtime = Runtime::new();
        for stmt in ast {
            runtime.eval_stmt(&stmt);
        }
        assert_feq!(runtime.variables.get("Y").unwrap(), 0.0, 0.000001);
    }

    #[test]
    fn test_if_false_with_else() {
        let input = r#"
IF 0 THEN
    LET Y = 1
ELSE
    LET Y = 2
ENDIF
"#;
        let ast = Parser::from_str(input).program().unwrap();
        let mut runtime = Runtime::new();
        for stmt in ast {
            runtime.eval_stmt(&stmt);
        }
        assert_feq!(runtime.variables.get("Y").unwrap(), 2.0, 0.000001);
    }

    #[test]
    fn test_if_true_skips_else() {
        let input = r#"
IF 1 THEN
    LET Y = 1
ELSE
    LET Y = 2
ENDIF
"#;
        let ast = Parser::from_str(input).program().unwrap();
        let mut runtime = Runtime::new();
        for stmt in ast {
            runtime.eval_stmt(&stmt);
        }
        assert_feq!(runtime.variables.get("Y").unwrap(), 1.0, 0.000001);
    }

    #[test]
    fn test_elseif_first_true_branch_wins() {
        let input = r#"
LET X = 5
IF X > 0 THEN
    LET Y = 1
ELSEIF X > 3 THEN
    LET Y = 2
ELSEIF X > 1 THEN
    LET Y = 3
ENDIF
"#;
        let ast = Parser::from_str(input).program().unwrap();
        let mut runtime = Runtime::new();
        for stmt in ast {
            runtime.eval_stmt(&stmt);
        }
        // All conditions are true, but first should win
        assert_feq!(runtime.variables.get("Y").unwrap(), 1.0, 0.000001);
    }

    #[test]
    fn test_elseif_second_branch() {
        let input = r#"
LET X = 2
IF X == 1 THEN
    LET Y = 1
ELSEIF X == 2 THEN
    LET Y = 2
ELSEIF X == 3 THEN
    LET Y = 3
ELSE
    LET Y = 4
ENDIF
"#;
        let ast = Parser::from_str(input).program().unwrap();
        let mut runtime = Runtime::new();
        for stmt in ast {
            runtime.eval_stmt(&stmt);
        }
        assert_feq!(runtime.variables.get("Y").unwrap(), 2.0, 0.000001);
    }

    #[test]
    fn test_elseif_falls_through_to_else() {
        let input = r#"
LET X = 99
IF X == 1 THEN
    LET Y = 1
ELSEIF X == 2 THEN
    LET Y = 2
ELSE
    LET Y = 3
ENDIF
"#;
        let ast = Parser::from_str(input).program().unwrap();
        let mut runtime = Runtime::new();
        for stmt in ast {
            runtime.eval_stmt(&stmt);
        }
        assert_feq!(runtime.variables.get("Y").unwrap(), 3.0, 0.000001);
    }

    #[test]
    fn test_while_never_executes() {
        let input = r#"
LET X = 0
LET Y = 0
WHILE X > 0 REPEAT
    LET Y = Y + 1
ENDWHILE
"#;
        let ast = Parser::from_str(input).program().unwrap();
        let mut runtime = Runtime::new();
        for stmt in ast {
            runtime.eval_stmt(&stmt);
        }
        assert_feq!(runtime.variables.get("Y").unwrap(), 0.0, 0.000001);
    }

    #[test]
    fn test_while_executes_once() {
        let input = r#"
LET X = 1
LET Y = 0
WHILE X > 0 REPEAT
    LET Y = Y + 1
    LET X = X - 1
ENDWHILE
"#;
        let ast = Parser::from_str(input).program().unwrap();
        let mut runtime = Runtime::new();
        for stmt in ast {
            runtime.eval_stmt(&stmt);
        }
        assert_feq!(runtime.variables.get("Y").unwrap(), 1.0, 0.000001);
    }

    #[test]
    fn test_nested_while() {
        let input = r#"
LET X = 2
LET Y = 0
WHILE X > 0 REPEAT
    LET Z = 3
    WHILE Z > 0 REPEAT
        LET Y = Y + 1
        LET Z = Z - 1
    ENDWHILE
    LET X = X - 1
ENDWHILE
"#;
        let ast = Parser::from_str(input).program().unwrap();
        let mut runtime = Runtime::new();
        for stmt in ast {
            runtime.eval_stmt(&stmt);
        }
        // 2 outer iterations * 3 inner iterations = 6
        assert_feq!(runtime.variables.get("Y").unwrap(), 6.0, 0.000001);
    }

    #[test]
    fn test_if_inside_while() {
        let input = r#"
LET X = 4
LET EVENS = 0
LET ODDS = 0
WHILE X > 0 REPEAT
    LET REMAINDER = X - (X / 2) * 2
    IF REMAINDER == 0 THEN
        LET EVENS = EVENS + 1
    ELSE
        LET ODDS = ODDS + 1
    ENDIF
    LET X = X - 1
ENDWHILE
"#;
        let ast = Parser::from_str(input).program().unwrap();
        let mut runtime = Runtime::new();
        for stmt in ast {
            runtime.eval_stmt(&stmt);
        }
        // X goes 4, 3, 2, 1 - but with float division this won't work as expected
        // 4: 4 - (4/2)*2 = 4 - 4 = 0 (even)
        // 3: 3 - (3/2)*2 = 3 - 3 = 0 (even??) - float division: 3/2 = 1.5, 1.5*2 = 3
        // 2: 2 - (2/2)*2 = 2 - 2 = 0 (even)
        // 1: 1 - (1/2)*2 = 1 - 1 = 0 (even??) - float division: 1/2 = 0.5, 0.5*2 = 1
        assert_feq!(runtime.variables.get("EVENS").unwrap(), 4.0, 0.000001);
    }

    #[test]
    fn test_while_inside_if() {
        let input = r#"
LET X = 1
LET Y = 0
IF X THEN
    WHILE X < 5 REPEAT
        LET Y = Y + X
        LET X = X + 1
    ENDWHILE
ENDIF
"#;
        let ast = Parser::from_str(input).program().unwrap();
        let mut runtime = Runtime::new();
        for stmt in ast {
            runtime.eval_stmt(&stmt);
        }
        // Y = 1 + 2 + 3 + 4 = 10
        assert_feq!(runtime.variables.get("Y").unwrap(), 10.0, 0.000001);
    }

    #[test]
    fn test_condition_uses_modified_variable() {
        let input = r#"
LET X = 1
IF X == 1 THEN
    LET X = 2
ELSEIF X == 2 THEN
    LET Y = 999
ENDIF
"#;
        let ast = Parser::from_str(input).program().unwrap();
        let mut runtime = Runtime::new();
        for stmt in ast {
            runtime.eval_stmt(&stmt);
        }
        // First branch runs (X was 1), sets X to 2
        // ELSEIF should NOT run even though X is now 2
        assert_feq!(runtime.variables.get("X").unwrap(), 2.0, 0.000001);
        assert!(runtime.variables.get("Y").is_none());
    }

    #[test]
    fn test_complex_while_condition() {
        let input = r#"
LET X = 5
LET Y = 0
WHILE X > 0 && X < 10 REPEAT
    LET Y = Y + 1
    LET X = X - 1
ENDWHILE
"#;
        let ast = Parser::from_str(input).program().unwrap();
        let mut runtime = Runtime::new();
        for stmt in ast {
            runtime.eval_stmt(&stmt);
        }
        assert_feq!(runtime.variables.get("Y").unwrap(), 5.0, 0.000001);
    }

    #[test]
    fn test_negative_numbers() {
        let input = r#"
LET X = -5
LET Y = 0
WHILE X < 0 REPEAT
    LET Y = Y + 1
    LET X = X + 1
ENDWHILE
"#;
        let ast = Parser::from_str(input).program().unwrap();
        let mut runtime = Runtime::new();
        for stmt in ast {
            runtime.eval_stmt(&stmt);
        }
        assert_feq!(runtime.variables.get("Y").unwrap(), 5.0, 0.000001);
    }

    #[test]
    fn test_deeply_nested_if() {
        let input = r#"
LET X = 3
LET Y = 0
IF X > 0 THEN
    IF X > 1 THEN
        IF X > 2 THEN
            LET Y = 1
        ENDIF
    ENDIF
ENDIF
"#;
        let ast = Parser::from_str(input).program().unwrap();
        let mut runtime = Runtime::new();
        for stmt in ast {
            runtime.eval_stmt(&stmt);
        }
        assert_feq!(runtime.variables.get("Y").unwrap(), 1.0, 0.000001);
    }

    #[test]
    fn test_or_short_circuit_not_needed() {
        // Tests that OR works correctly (both sides evaluated in this impl)
        let input = r#"
LET X = 0
LET Y = 1
IF X || Y THEN
    LET Z = 1
ELSE
    LET Z = 0
ENDIF
"#;
        let ast = Parser::from_str(input).program().unwrap();
        let mut runtime = Runtime::new();
        for stmt in ast {
            runtime.eval_stmt(&stmt);
        }
        assert_feq!(runtime.variables.get("Z").unwrap(), 1.0, 0.000001);
    }

    #[test]
    fn test_comparison_chain() {
        let input = r#"
LET A = 1
LET B = 2
LET C = 3
IF A < B && B < C THEN
    LET Y = 1
ELSE
    LET Y = 0
ENDIF
"#;
        let ast = Parser::from_str(input).program().unwrap();
        let mut runtime = Runtime::new();
        for stmt in ast {
            runtime.eval_stmt(&stmt);
        }
        assert_feq!(runtime.variables.get("Y").unwrap(), 1.0, 0.000001);
    }
}
