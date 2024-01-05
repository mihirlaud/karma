mod lexer;
mod parser;
mod source;

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::source::Source;

fn main() {
    let lexer = Lexer::new("code/current_snippet.krm");

    let mut parser = Parser::new(lexer);

    match parser.parse() {
        Ok(_) => {
            parser.generate_ast();
            let source = Source::new(parser);
            source.generate_assembly();
        }
        Err(s) => println!("{s}"),
    }
}
