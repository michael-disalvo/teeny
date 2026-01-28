use crate::parse::{Expr, IfBranch, IfStmt, PrintValue, Stmt, WhileStmt};

use std::collections::HashSet;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct Emitter {
    header: String,
    code: String,
    declared_variables: HashSet<String>,
}

impl Emitter {
    pub fn new() -> Self {
        let mut slf = Emitter {
            header: String::new(),
            code: String::new(),
            declared_variables: Default::default(),
        };

        slf.header_line("#include <stdio.h>");
        slf.emit_line("int main(void) {");
        slf
    }

    fn emit(&mut self, code: impl AsRef<str>) {
        self.code += code.as_ref();
    }

    fn emit_line(&mut self, code: impl AsRef<str>) {
        self.emit(code.as_ref());
        self.code += "\n";
    }

    fn header_line(&mut self, code: &str) {
        self.header += code;
        self.header += "\n";
    }

    pub fn write_out(&mut self, w: &mut impl Write) {
        self.emit_line("return 0;");
        self.emit_line("}");

        writeln!(w, "{}", self.header).expect("writing in emitter");
        writeln!(w, "{}", self.code).expect("writing in emitter");
    }
}

impl Emitter {
    pub fn emit_expr(&mut self, expr: &Expr) {
        self.emit("(");
        match expr {
            Expr::Identifier(id) => self.emit(id),
            Expr::Number(n) => self.emit(n),
            Expr::Unary(op, expr) => {
                self.emit(op.text());
                self.emit_expr(expr);
            }
            Expr::Binary(op, lhs, rhs) => {
                self.emit_expr(lhs);
                self.emit(op.text());
                self.emit_expr(rhs);
            }
        }
        self.emit(")");
    }

    pub fn emit_if_branch(&mut self, if_branch: &IfBranch) {
        self.emit("(");
        self.emit_expr(&if_branch.condition);
        self.emit_line(") {");
        for stmt in &if_branch.body {
            self.emit_stmt(stmt);
        }

        self.emit("}");
    }

    pub fn emit_if_stmt(&mut self, if_stmt: &IfStmt) {
        self.emit("if ");
        self.emit_if_branch(&if_stmt.first_branch);
        for branch in &if_stmt.other_branches {
            self.emit(" else if ");
            self.emit_if_branch(branch);
        }

        if let Some(else_body) = &if_stmt.else_body {
            self.emit_line(" else {");
            for stmt in else_body {
                self.emit_stmt(stmt);
            }
            self.emit("}");
        }

        self.emit_line("");
    }

    pub fn emit_while_stmt(&mut self, while_stmt: &WhileStmt) {
        self.emit("while (");
        self.emit_expr(&while_stmt.condition);
        self.emit_line(") {");

        for stmt in &while_stmt.body {
            self.emit_stmt(stmt);
        }

        self.emit_line("}");
    }

    pub fn emit_print_stmt(&mut self, print_value: &PrintValue) {
        self.emit("printf(");

        match print_value {
            PrintValue::Str(s) => self.emit_line(format!("\"{}\\n\");", s)),
            PrintValue::Expr(expr) => {
                self.emit("\"%.2f\\n\", (float)");
                self.emit_expr(expr);
                self.emit_line(");");
            }
        }
    }

    pub fn emit_input(&mut self, ident: &str) {
        if !self.declared_variables.contains(ident) {
            self.emit_line(format!("float {};", ident));
            self.declared_variables.insert(ident.to_owned());
        }
        self.emit_line(format!("if(0 == scanf(\"%f\", &{})) {{", ident));
        self.emit_line(format!("{} = 0;", ident));
        self.emit_line("scanf(\"%*s\");");
        self.emit_line("}");
    }

    pub fn emit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Print(pv) => self.emit_print_stmt(pv),
            Stmt::If(if_stmt) => self.emit_if_stmt(if_stmt),
            Stmt::While(while_stmt) => self.emit_while_stmt(while_stmt),
            Stmt::Label(s) => self.emit_line(format!("{}:", s)),
            Stmt::Goto(s) => self.emit_line(format!("goto {};", s)),
            Stmt::Input(ident) => self.emit_input(ident),
            Stmt::Let(ident, expr) => {
                if !self.declared_variables.contains(ident) {
                    self.emit(format!("float {} = ", ident));
                    self.declared_variables.insert(ident.clone());
                } else {
                    self.emit(format!("{} = ", ident));
                }
                self.emit_expr(expr);
                self.emit_line(";");
            }
        }
    }

    pub fn emit_tree(&mut self, ast: &[Stmt]) {
        for stmt in ast {
            self.emit_stmt(stmt)
        }
    }
}
