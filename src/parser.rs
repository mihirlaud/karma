use crate::lexer::Token;


pub struct Parser {
    tokens: Vec<Token>,
    curr: usize,
}

impl Parser {

    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            curr: 0,
        }
    }

    pub fn parse(&mut self) -> Result<(), &str> {
        while self.curr < self.tokens.len() {
            self.curr += 1;
        }

        Ok(())
    }
}