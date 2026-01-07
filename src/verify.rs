use crate::parse::{Expr, IfBranch, IfStmt, PrintValue, Stmt, WhileStmt};
use std::collections::HashSet;

#[derive(Default)]
struct Context {
    identifiers_declared: HashSet<String>,
    labels_declared: HashSet<String>,
    labels_gotoed: HashSet<String>,
}

pub fn walk_expr(expr: &Expr, context: &mut Context) {
    match expr {
        Expr::Binary(_, lhs, rhs) => {
            walk_expr(lhs, context);
            walk_expr(rhs, context);
        }
        Expr::Unary(_, inner) => walk_expr(inner, context),
        Expr::Number(_) => {}
        Expr::Identifier(ident) => {
            if !context.identifiers_declared.contains(ident) {
                panic!("Lexical error. Identifier `{ident}` used before declared");
            }
        }
    }
}

pub fn walk_if_branch(if_branch: &IfBranch, context: &mut Context) {
    let IfBranch { condition, body } = if_branch;

    walk_expr(condition, context);

    for stmt in body {
        walk_stmt(stmt, context);
    }
}

pub fn walk_if_stmt(if_stmt: &IfStmt, context: &mut Context) {
    let IfStmt {
        first_branch,
        other_branches,
        else_body,
    } = if_stmt;

    walk_if_branch(first_branch, context);
    for other_branch in other_branches {
        walk_if_branch(other_branch, context)
    }
    if let Some(body) = else_body {
        for stmt in body {
            walk_stmt(stmt, context);
        }
    }
}

pub fn walk_while_stmt(while_stmt: &WhileStmt, context: &mut Context) {
    let WhileStmt { condition, body } = while_stmt;

    walk_expr(condition, context);
    for stmt in body {
        walk_stmt(stmt, context)
    }
}

pub fn walk_stmt(stmt: &Stmt, context: &mut Context) {
    match stmt {
        Stmt::Print(_) => {}
        Stmt::If(if_stmt) => walk_if_stmt(if_stmt, context),
        Stmt::While(while_stmt) => walk_while_stmt(while_stmt, context),
        Stmt::Label(label) => {
            context.labels_declared.insert(label.clone());
        }
        Stmt::Goto(label) => {
            context.labels_gotoed.insert(label.clone());
        }
        Stmt::Input(ident) => {
            context.identifiers_declared.insert(ident.clone());
        }
        Stmt::Let(ident, expr) => {
            context.identifiers_declared.insert(ident.clone());
            walk_expr(expr, context);
        }
    }
}

pub fn walk_tree(ast: &[Stmt]) {
    let mut context = Context::default();
    for stmt in ast {
        walk_stmt(stmt, &mut context)
    }

    for label_gotoed in context.labels_gotoed {
        if !context.labels_declared.contains(&label_gotoed) {
            panic!("Lexer error. Label gotoed `{label_gotoed}` does not exist");
        }
    }
}
