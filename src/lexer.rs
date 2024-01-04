use phf::phf_map;
use std::fs::File;
use std::io::{BufRead, BufReader};

static KEYWORDS: phf::Map<&'static str, Token> = phf_map! {
    "node" => Token::Node,
    "export" => Token::Export,
    "var" => Token::Var, "const" => Token::Const,
    "fn" => Token::Fn,
    "while" => Token::While,
    "true" => Token::True, "false" => Token::False,
    "if" => Token::If, "else" => Token::Else,
    "return" => Token::Return,
};

static SYMBOLS: phf::Map<&'static str, Token> = phf_map! {
    "=" => Token::Assign,
    "+" => Token::Add, "-" => Token::Sub, "*" => Token::Mul, "/" => Token::Div,
    "+=" => Token::AddAssign, "-=" => Token::SubAssign, "*=" => Token::MulAssign, "/=" => Token::DivAssign,
    "(" => Token::LeftParen, ")" => Token::RightParen, "[" => Token::LeftBracket, "]" => Token::RightBracket, "{" => Token::LeftBrace, "}" => Token::RightBrace,
    ";" => Token::Semicolon, ":" => Token::Colon, "::" => Token::DoubleColon,
    "->" => Token::Arrow, "." => Token::Dot, "," => Token::Comma,
    "==" => Token::Equals, "!" => Token::Not, "<" => Token::Less, ">" => Token::Greater, "<=" => Token::Leq, ">=" => Token::Geq, "!=" => Token::Neq,
    "&&" => Token::LogicalAnd, "||" => Token::LogicalOr, "&" => Token::BitwiseAnd, "|" => Token::BitwiseOr,
};

#[derive(Clone, PartialEq, Debug)]
pub enum Token {
    ID(String),
    Integer(i32),
    Float(f64),
    StringLiteral(String),
    Node,
    Export,
    Var,
    Const,
    Fn,
    While,
    True,
    False,
    If,
    Else,
    Assign,
    Add,
    Mul,
    Sub,
    Div,
    AddAssign,
    MulAssign,
    SubAssign,
    DivAssign,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Semicolon,
    Colon,
    DoubleColon,
    Arrow,
    Dot,
    Comma,
    Equals,
    Not,
    Less,
    Greater,
    Leq,
    Geq,
    Neq,
    LogicalAnd,
    LogicalOr,
    BitwiseAnd,
    BitwiseOr,
    Return,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::ID(name) => write!(f, "ID: {name}"),
            Token::Integer(int) => write!(f, "Integer: {int}"),
            Token::Float(float) => write!(f, "Float: {float}"),
            Token::StringLiteral(s) => write!(f, "String literal: {s}"),
            Token::Node => write!(f, "node"),
            Token::Export => write!(f, "export"),
            Token::Var => write!(f, "var"),
            Token::Const => write!(f, "const"),
            Token::Fn => write!(f, "fn"),
            Token::While => write!(f, "while"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::Assign => write!(f, "="),
            Token::Add => write!(f, "+"),
            Token::Mul => write!(f, "*"),
            Token::Sub => write!(f, "-"),
            Token::Div => write!(f, "/"),
            Token::AddAssign => write!(f, "+="),
            Token::MulAssign => write!(f, "*="),
            Token::SubAssign => write!(f, "-="),
            Token::DivAssign => write!(f, "/="),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
            Token::Semicolon => write!(f, ";"),
            Token::Colon => write!(f, ":"),
            Token::DoubleColon => write!(f, "::"),
            Token::Arrow => write!(f, "->"),
            Token::Dot => write!(f, "."),
            Token::Comma => write!(f, ","),
            Token::Equals => write!(f, "=="),
            Token::Not => write!(f, "!"),
            Token::Less => write!(f, "<"),
            Token::Greater => write!(f, ">"),
            Token::Leq => write!(f, "<="),
            Token::Geq => write!(f, ">="),
            Token::Neq => write!(f, "!="),
            Token::LogicalAnd => write!(f, "&&"),
            Token::LogicalOr => write!(f, "||"),
            Token::BitwiseAnd => write!(f, "&"),
            Token::BitwiseOr => write!(f, "|"),
            Token::Return => write!(f, "return"),
        }
    }
}

pub struct Lexer {
    chars: Vec<char>,
    curr: usize,
}

impl Lexer {
    pub fn new(filename: &str) -> Self {
        let f = BufReader::new(File::open(filename).expect("Could not open file"));

        let mut chars = vec![];

        for line in f.lines() {
            for c in line.expect("lines failed").chars() {
                chars.push(c);
            }
        }

        Self { chars, curr: 0 }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        let mut forward = self.curr;
        let mut state = 0;

        while forward < self.chars.len() {
            let c = self.chars[forward];

            match state {
                0 => {
                    if c == ' ' || c == '\t' || c == '\n' || c == '\r' {
                        self.curr = forward + 1;
                    }

                    if c == '{' {
                        self.curr = forward + 1;
                        return Some(Token::LeftBrace);
                    }

                    if c == '}' {
                        self.curr = forward + 1;
                        return Some(Token::RightBrace);
                    }

                    if c == '(' {
                        self.curr = forward + 1;
                        return Some(Token::LeftParen);
                    }

                    if c == ')' {
                        self.curr = forward + 1;
                        return Some(Token::RightParen);
                    }

                    if c == '[' {
                        self.curr = forward + 1;
                        return Some(Token::LeftBracket);
                    }

                    if c == ']' {
                        self.curr = forward + 1;
                        return Some(Token::RightBracket);
                    }

                    if c == ';' {
                        self.curr = forward + 1;
                        return Some(Token::Semicolon);
                    }

                    if c == '.' {
                        self.curr = forward + 1;
                        return Some(Token::Dot);
                    }

                    if c == ',' {
                        self.curr = forward + 1;
                        return Some(Token::Comma);
                    }

                    if c == '+'
                        || c == '*'
                        || c == '/'
                        || c == '='
                        || c == '!'
                        || c == '<'
                        || c == '>'
                    {
                        state = 1;
                    }

                    if c == ':' {
                        state = 2;
                    }

                    if c == '-' {
                        state = 3;
                    }

                    if c.is_ascii_alphabetic() || c == '_' {
                        state = 4;
                    }

                    if c.is_ascii_digit() {
                        state = 5;
                    }

                    if c == '"' {
                        state = 7;
                    }

                    if c == '&' {
                        state = 8;
                    }

                    if c == '|' {
                        state = 9;
                    }
                }
                1 => {
                    if c == '=' {
                        let attr = self.chars[self.curr..forward + 1]
                            .into_iter()
                            .collect::<String>();
                        self.curr = forward + 1;
                        return Some(SYMBOLS[attr.as_str()].clone());
                    } else {
                        let attr = String::from(self.chars[forward - 1]);
                        self.curr = forward;
                        return Some(SYMBOLS[attr.as_str()].clone());
                    }
                }
                2 => {
                    if c == ':' {
                        self.curr = forward + 1;
                        return Some(Token::DoubleColon);
                    } else {
                        self.curr = forward;
                        return Some(Token::Colon);
                    }
                }
                3 => {
                    if c == '>' {
                        self.curr = forward + 1;
                        return Some(Token::Arrow);
                    } else if c == '=' {
                        self.curr = forward + 1;
                        return Some(Token::SubAssign);
                    } else {
                        self.curr = forward;
                        return Some(Token::Sub);
                    }
                }
                4 => {
                    if !(c.is_ascii_alphanumeric() || c == '_') {
                        let attr = self.chars[self.curr..forward]
                            .into_iter()
                            .collect::<String>();
                        self.curr = forward;

                        if KEYWORDS.contains_key(attr.as_str()) {
                            return Some(KEYWORDS[attr.as_str()].clone());
                        } else {
                            return Some(Token::ID(attr));
                        }
                    }
                }
                5 => {
                    if c == '.' {
                        state = 6;
                    } else if !(c.is_ascii_digit()) {
                        let attr = self.chars[self.curr..forward]
                            .into_iter()
                            .collect::<String>();
                        let val: i32 = attr.parse().expect("Failed to convert to float");
                        self.curr = forward;
                        return Some(Token::Integer(val));
                    }
                }
                6 => {
                    if !(c.is_ascii_digit()) {
                        let attr = self.chars[self.curr..forward]
                            .into_iter()
                            .collect::<String>();
                        let val: f64 = attr.parse().expect("Failed to convert to float");
                        self.curr = forward;
                        return Some(Token::Float(val));
                    }
                }
                7 => {
                    if c == '"' {
                        let attr = self.chars[self.curr..forward + 1]
                            .into_iter()
                            .collect::<String>();
                        self.curr = forward + 1;
                        return Some(Token::StringLiteral(attr));
                    }
                }
                8 => {
                    if c == '&' {
                        self.curr = forward + 1;
                        return Some(Token::LogicalAnd);
                    } else {
                        self.curr = forward;
                        return Some(Token::BitwiseAnd);
                    }
                }
                9 => {
                    if c == '|' {
                        self.curr = forward + 1;
                        return Some(Token::LogicalAnd);
                    } else {
                        self.curr = forward;
                        return Some(Token::BitwiseAnd);
                    }
                }
                _ => {}
            }

            forward += 1;
        }

        match state {
            4 => {
                let attr = self.chars[self.curr..self.chars.len()]
                    .into_iter()
                    .collect::<String>();
                self.curr = self.chars.len();

                if KEYWORDS.contains_key(attr.as_str()) {
                    Some(KEYWORDS[attr.as_str()].clone())
                } else {
                    Some(Token::ID(attr))
                }
            }
            5 => {
                let attr = self.chars[self.curr..self.chars.len()]
                    .into_iter()
                    .collect::<String>();
                let val: i32 = attr.parse().expect("Failed to convert to float");
                self.curr = self.chars.len();
                Some(Token::Integer(val))
            }
            6 => {
                let attr = self.chars[self.curr..self.chars.len()]
                    .into_iter()
                    .collect::<String>();
                let val: f64 = attr.parse().expect("Failed to convert to float");
                self.curr = self.chars.len();
                Some(Token::Float(val))
            }
            7 => {
                let attr = self.chars[self.curr..self.chars.len()]
                    .into_iter()
                    .collect::<String>();
                self.curr = self.chars.len();
                Some(Token::StringLiteral(attr))
            }
            _ => None,
        }
    }
}
