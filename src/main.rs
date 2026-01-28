use emit::Emitter;
use lex::Lexer;
use parse::Parser;
pub use token::Token;

use std::io;

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
        emitter.emit_tree(&ast);
        emitter.write_out(&mut io::stdout());
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
