use lex::Lexer;
use parse::Parser;
pub use token::Token;

mod lex;
mod parse;
mod token;

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
    let s = r#"
LET iter = 10
WHILE iter > 0 REPEAT
    PRINT iter
    LET iter = iter - 1
    IF iter < 4 THEN
        GOTO DONE
    ENDIF
ENDWHILE 
LABEL DONE
PRINT "Done."
"#;

    let lexer = Lexer::new(s);
    let mut parser = Parser::new(lexer, Emitter::new());
    parser.program();
    parser.emitter.write_out();
}
