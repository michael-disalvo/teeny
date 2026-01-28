use lex::Lexer;
use parse::{Expr, IfBranch, IfStmt, Parser, PrintValue, Stmt, WhileStmt};
use std::collections::HashSet;
pub use token::Token;

mod interpret;
mod lex;
mod parse;
mod token;
mod verify;

#[derive(clap::Parser)]
#[command(version, about)]
struct Args {
    /// If given a file, compile the teeny program into C code instead of interpret
    #[clap(short, long)]
    compile: bool,
    /// Input file of teeny code to either interpret or compile
    input_file: Option<String>,
}

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

    pub fn emit(&mut self, code: impl AsRef<str>) {
        self.code += code.as_ref();
    }

    pub fn emit_line(&mut self, code: impl AsRef<str>) {
        self.emit(code.as_ref());
        self.code += "\n";
    }

    pub fn header_line(&mut self, code: &str) {
        self.header += code;
        self.header += "\n";
    }

    pub fn write_out(&mut self) {
        self.emit_line("return 0;");
        self.emit_line("}");

        println!("{}", self.header);
        println!("{}", self.code);
    }
}

pub fn emit_expr(expr: &Expr, emitter: &mut Emitter) {
    emitter.emit("(");
    match expr {
        Expr::Identifier(id) => emitter.emit(id),
        Expr::Number(n) => emitter.emit(n),
        Expr::Unary(op, expr) => {
            emitter.emit(op.text());
            emit_expr(expr, emitter);
        }
        Expr::Binary(op, lhs, rhs) => {
            emit_expr(lhs, emitter);
            emitter.emit(op.text());
            emit_expr(rhs, emitter);
        }
    }
    emitter.emit(")");
}

pub fn emit_if_branch(if_branch: &IfBranch, emitter: &mut Emitter) {
    emitter.emit("(");
    emit_expr(&if_branch.condition, emitter);
    emitter.emit_line(") {");
    for stmt in &if_branch.body {
        emit_stmt(stmt, emitter);
    }

    emitter.emit("}");
}

pub fn emit_if_stmt(if_stmt: &IfStmt, emitter: &mut Emitter) {
    emitter.emit("if ");
    emit_if_branch(&if_stmt.first_branch, emitter);
    for branch in &if_stmt.other_branches {
        emitter.emit(" else if ");
        emit_if_branch(branch, emitter);
    }

    if let Some(else_body) = &if_stmt.else_body {
        emitter.emit_line(" else {");
        for stmt in else_body {
            emit_stmt(stmt, emitter);
        }
        emitter.emit("}");
    }

    emitter.emit_line("");
}

pub fn emit_while_stmt(while_stmt: &WhileStmt, emitter: &mut Emitter) {
    emitter.emit("while (");
    emit_expr(&while_stmt.condition, emitter);
    emitter.emit_line(") {");

    for stmt in &while_stmt.body {
        emit_stmt(stmt, emitter);
    }

    emitter.emit_line("}");
}

pub fn emit_print_stmt(print_value: &PrintValue, emitter: &mut Emitter) {
    emitter.emit("printf(");

    match print_value {
        PrintValue::Str(s) => emitter.emit_line(format!("\"{}\\n\");", s)),
        PrintValue::Expr(expr) => {
            emitter.emit("\"%.2f\\n\", (float)");
            emit_expr(expr, emitter);
            emitter.emit_line(");");
        }
    }
}

pub fn emit_input(ident: &str, emitter: &mut Emitter) {
    if !emitter.declared_variables.contains(ident) {
        emitter.emit_line(format!("float {};", ident));
        emitter.declared_variables.insert(ident.to_owned());
    }
    emitter.emit_line(format!("if(0 == scanf(\"%f\", &{})) {{", ident));
    emitter.emit_line(format!("{} = 0;", ident));
    emitter.emit_line("scanf(\"%*s\");");
    emitter.emit_line("}");
}

pub fn emit_stmt(stmt: &Stmt, emitter: &mut Emitter) {
    match stmt {
        Stmt::Print(pv) => emit_print_stmt(pv, emitter),
        Stmt::If(if_stmt) => emit_if_stmt(if_stmt, emitter),
        Stmt::While(while_stmt) => emit_while_stmt(while_stmt, emitter),
        Stmt::Label(s) => emitter.emit_line(format!("{}:", s)),
        Stmt::Goto(s) => emitter.emit_line(format!("goto {};", s)),
        Stmt::Input(ident) => emit_input(ident, emitter),
        Stmt::Let(ident, expr) => {
            if !emitter.declared_variables.contains(ident) {
                emitter.emit(format!("float {} = ", ident));
                emitter.declared_variables.insert(ident.clone());
            } else {
                emitter.emit(format!("{} = ", ident));
            }
            emit_expr(expr, emitter);
            emitter.emit_line(";");
        }
    }
}

pub fn emit_tree(ast: &[Stmt], emitter: &mut Emitter) {
    for stmt in ast {
        emit_stmt(stmt, emitter)
    }
}

fn do_repl() {
    use std::io;

    let stdin = io::stdin();

    let lexer = Lexer::from_reader(stdin);
    let mut parser = Parser::new(lexer);

    let mut runtime = interpret::Runtime::new();

    loop {
        let stmt = parser.statement();
        runtime.eval_stmt(&stmt);
    }
}

fn do_file(input_file: String, compile: bool) {
    let s = std::fs::read_to_string(input_file).expect("failed to read input file");
    let mut parser = Parser::from_str(&s);
    let ast = parser.program();

    verify::verify_tree(&ast);

    if compile {
        let mut emitter = Emitter::new();
        emit_tree(&ast, &mut emitter);
        emitter.write_out();
    } else {
        let mut runtime = interpret::Runtime::new();
        for stmt in ast {
            runtime.eval_stmt(&stmt);
        }
    }
}

fn main() {
    let args = <Args as clap::Parser>::parse();

    if let Some(file) = args.input_file {
        do_file(file, args.compile);
    } else {
        do_repl();
    }
}
