use lex::Lexer;
use parse::Parser;
pub use token::Token;

mod lex;
mod parse;
mod token;

fn main() {
    let s = r#"
LET X = 4.2
PRINT X + 2
INPUT Y
PRINT Y
LABEL loop
PRINT "hello world!"
GOTO loop
"#;

    let lexer = Lexer::new(s);
    let mut parser = Parser::new(lexer);
    parser.program();
}
