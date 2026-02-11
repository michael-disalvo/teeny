use emit::Emitter;
use lex::Lexer;
use parse::Parser;
pub use token::Token;

use std::io::{self, Write};

mod emit;
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

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Lexer error. {0}")]
    Lexer(String),
    #[error("IO error. {0}")]
    Io(#[from] io::Error),
    #[error("Parser error. {0}")]
    Parse(String),
    #[error("Runtime error. {0}")]
    Runtime(String),
}

#[macro_export]
macro_rules! lexer_err {
    ($($arg:tt)*) => {
        crate::Error::Lexer(format!($($arg)*).into())
    };
}

#[macro_export]
macro_rules! parse_err {
    ($($arg:tt)*) => {
        crate::Error::Parse(format!($($arg)*).into())
    };
}

#[macro_export]
macro_rules! runtime_err {
    ($($arg:tt)*) => {
        crate::Error::Runtime(format!($($arg)*).into())
    };
}

type Result<T> = std::result::Result<T, Error>;

fn do_repl() -> Result<()> {
    use std::io;

    let stdin = io::stdin();

    let lexer = Lexer::from_reader(stdin);
    let mut parser = Parser::new(lexer);

    let mut runtime = interpret::Runtime::new();

    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();
        let stmt = parser.statement()?;
        if let Err(e) = runtime.eval_stmt(&stmt) {
            println!("{e:?}");
        }
    }
}

fn do_file(
    input_file: impl AsRef<std::path::Path>,
    compile: bool,
    w: &mut impl Write,
) -> Result<()> {
    let s = std::fs::read_to_string(input_file).expect("failed to read input file");
    let mut parser = Parser::from_str(&s);
    let ast = parser.program()?;

    verify::verify_tree(&ast);

    if compile {
        let mut emitter = Emitter::new();
        emitter.emit_tree(&ast);
        emitter.write_out(w);
    } else {
        let mut runtime = interpret::Runtime::new();
        for stmt in ast {
            runtime.eval_stmt(&stmt)?;
        }
    }
    Ok(())
}

fn main() {
    let args = <Args as clap::Parser>::parse();

    let res = if let Some(file) = args.input_file {
        do_file(file, args.compile, &mut io::stdout())
    } else {
        do_repl()
    };

    if let Err(e) = res {
        eprintln!("{e:?}");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn e2e_while_loop() {
        let input_file = "test_files/while_loop.teeny";

        let expected_file = "test_files/while_loop.c";
        let expected = std::fs::read_to_string(expected_file).unwrap();

        let mut got = Vec::new();
        do_file(input_file, true, &mut got).unwrap();
        assert_eq!(String::from_utf8(got).unwrap(), expected);
    }
}
