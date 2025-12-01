use lex::Lexer;
use parse::Parser;
pub use token::Token;

mod lex;
mod parse;
mod token;

#[derive(clap::Parser)]
#[command(version, about)]
struct Args {
    /// Input file of teeny code to compile
    input_file: String,
}

#[derive(Debug, Clone)]
struct Emitter {
    header: String,
    code: String,
}

impl Emitter {
    pub fn new() -> Self {
        let mut slf = Emitter {
            header: String::new(),
            code: String::new(),
        };

        slf.header_line("#include <stdio.h>");
        slf.emit_line("int main(void) {");
        slf
    }

    pub fn emit(&mut self, code: &str) {
        self.code += code;
    }

    pub fn emit_line(&mut self, code: &str) {
        self.emit(code);
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

fn main() {
    let args = <Args as clap::Parser>::parse();

    let s = std::fs::read_to_string(args.input_file).expect("failed to read input file");

    let lexer = Lexer::new(&s);
    let mut parser = Parser::new(lexer, Emitter::new());
    parser.program();
    parser.emitter.write_out();
}
