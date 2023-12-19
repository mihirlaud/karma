mod lexer;
mod parser;

use crate::lexer::Lexer;

fn main() {
    let mut lexer = Lexer::new("code/current_snippet.krm");

    while let Some(token) = lexer.next_token() {
        println!("{token}");
    }

    println!("Lexing finished");
}