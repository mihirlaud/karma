mod lexer;
mod parser;

use crate::lexer::Lexer;
use crate::parser::Parser;

fn main() {
    let lexer = Lexer::new("code/current_snippet.krm");

    let mut parser = Parser::new(lexer);

    match parser.parse() {
        Ok(_) => {
            parser.generate_ast();
            println!("{:?}", parser.ast);
        }
        Err(s) => println!("{s}"),
    }
}
