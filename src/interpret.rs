use crate::parse::{Expr, IfBranch, IfStmt, Parser, PrintValue, Stmt, WhileStmt};
use crate::token::{BinaryOp, UnaryOp};

use std::collections::HashMap;
use std::str::FromStr;

type Runtime = HashMap<String, f32>;

pub fn eval(expr: &Expr, runtime: &Runtime) -> f32 {
    match expr {
        Expr::Number(num_str) => {
            f32::from_str(num_str).expect("should have validated number token")
        }
        Expr::Identifier(ident) => {
            let Some(val) = runtime.get(ident) else {
                panic!("Runtime error: no value for identifier {}", ident);
            };
            *val
        }
        Expr::Binary(op, lhs, rhs) => todo!(),
        Expr::Unary(op, expr) => {
            let operand = eval(expr, runtime);
            op.eval(operand)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
