mod lexer;
mod parser;
mod source;

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::source::Source;

fn main() {
    let filename = std::env::args().nth(1).expect("no path passed");
    let lexer = Lexer::new(&filename);

    let mut parser = Parser::new(lexer);

    match parser.parse() {
        Ok(_) => {
            parser.generate_ast();
            let source = Source::new(parser).expect("could not build source");
            source.compile().expect("could not compile");
        }
        Err(s) => println!("{s}"),
    }
}
