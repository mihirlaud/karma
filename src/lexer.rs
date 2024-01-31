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
    "struct" => Token::Struct,
    "int" => Token::Int, "float" => Token::FloatKW, "bool" => Token::Bool, "char" => Token::Char,
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
    Float(f32),
    Character(char),
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
    Struct,
    Int,
    FloatKW,
    Bool,
    Char,
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

    pub fn next_token(&mut self) -> Result<Option<Token>, String> {
        let mut forward = self.curr;
        let mut state = 0;

        while forward < self.chars.len() {
            let c = self.chars[forward];

            match state {
                0 => {
                    match c {
                        ' ' | '\t' | '\n' | '\r' | '{' | '}' | '(' | ')' | '[' | ']' | ';'
                        | '.' | ',' => {
                            self.curr = forward + 1;
                        }
                        _ => {}
                    }

                    match c {
                        '{' => {
                            return Ok(Some(Token::LeftBrace));
                        }
                        '}' => {
                            return Ok(Some(Token::RightBrace));
                        }
                        '(' => {
                            return Ok(Some(Token::LeftParen));
                        }
                        ')' => {
                            return Ok(Some(Token::RightParen));
                        }
                        '[' => {
                            return Ok(Some(Token::LeftBracket));
                        }
                        ']' => {
                            return Ok(Some(Token::RightBracket));
                        }
                        ';' => {
                            return Ok(Some(Token::Semicolon));
                        }
                        '.' => {
                            return Ok(Some(Token::Dot));
                        }
                        ',' => {
                            return Ok(Some(Token::Comma));
                        }
                        _ => {}
                    }

                    state = match c {
                        '+' | '*' | '=' | '!' | '<' | '>' => 1,
                        ':' => 2,
                        '-' => 3,
                        '_' => 4,
                        '"' => 7,
                        '&' => 8,
                        '|' => 9,
                        '/' => 10,
                        '\'' => 12,
                        _ => 0,
                    };

                    if c.is_ascii_alphabetic() {
                        state = 4;
                    }

                    if c.is_ascii_digit() {
                        state = 5;
                    }
                }
                1 => {
                    if c == '=' {
                        let attr = self.chars[self.curr..forward + 1]
                            .into_iter()
                            .collect::<String>();
                        self.curr = forward + 1;
                        return Ok(Some(SYMBOLS[attr.as_str()].clone()));
                    } else {
                        let attr = String::from(self.chars[forward - 1]);
                        self.curr = forward;
                        return Ok(Some(SYMBOLS[attr.as_str()].clone()));
                    }
                }
                2 => {
                    self.curr = forward + if c == ':' { 1 } else { 0 };
                    return Ok(Some(if c == ':' {
                        Token::DoubleColon
                    } else {
                        Token::Colon
                    }));
                }
                3 => {
                    self.curr = forward
                        + match c {
                            '>' | '=' => 1,
                            _ => 0,
                        };

                    return Ok(Some(match c {
                        '>' => Token::Arrow,
                        '=' => Token::SubAssign,
                        _ => Token::Sub,
                    }));
                }
                4 => {
                    if !(c.is_ascii_alphanumeric() || c == '_') {
                        let attr = self.chars[self.curr..forward]
                            .into_iter()
                            .collect::<String>();
                        self.curr = forward;

                        if KEYWORDS.contains_key(attr.as_str()) {
                            return Ok(Some(KEYWORDS[attr.as_str()].clone()));
                        } else {
                            return Ok(Some(Token::ID(attr)));
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
                        return Ok(Some(Token::Integer(val)));
                    }
                }
                6 => {
                    if !(c.is_ascii_digit()) {
                        let attr = self.chars[self.curr..forward]
                            .into_iter()
                            .collect::<String>();
                        let val: f32 = attr.parse().expect("Failed to convert to float");
                        self.curr = forward;
                        return Ok(Some(Token::Float(val)));
                    }
                }
                7 => {
                    if c == '"' {
                        let attr = self.chars[self.curr..forward + 1]
                            .into_iter()
                            .collect::<String>();
                        self.curr = forward + 1;
                        return Ok(Some(Token::StringLiteral(attr)));
                    }
                }
                8 => {
                    self.curr = forward
                        + match c {
                            '&' => 1,
                            _ => 0,
                        };

                    return Ok(Some(match c {
                        '&' => Token::LogicalAnd,
                        _ => Token::BitwiseAnd,
                    }));
                }
                9 => {
                    self.curr = forward
                        + match c {
                            '|' => 1,
                            _ => 0,
                        };

                    return Ok(Some(match c {
                        '|' => Token::LogicalOr,
                        _ => Token::BitwiseOr,
                    }));
                }
                10 => {
                    if c == '=' {
                        let attr = self.chars[self.curr..forward + 1]
                            .into_iter()
                            .collect::<String>();
                        self.curr = forward + 1;
                        return Ok(Some(SYMBOLS[attr.as_str()].clone()));
                    } else if c == '/' {
                        state = 11;
                    } else {
                        let attr = String::from(self.chars[forward - 1]);
                        self.curr = forward;
                        return Ok(Some(SYMBOLS[attr.as_str()].clone()));
                    }
                }
                11 => {
                    if c == '\n' {
                        self.curr = forward + 1;
                        state = 0;
                    }
                }
                12 => {
                    if c == '\'' {
                        if forward - self.curr != 2 {
                            return Err(String::from(
                                "Error: characters must be one character length",
                            ));
                        }
                        let attr = Token::Character(self.chars[self.curr + 1]);
                        self.curr = forward + 1;
                        return Ok(Some(attr));
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
                    Ok(Some(KEYWORDS[attr.as_str()].clone()))
                } else {
                    Ok(Some(Token::ID(attr)))
                }
            }
            5 => {
                let attr = self.chars[self.curr..self.chars.len()]
                    .into_iter()
                    .collect::<String>();
                let val: i32 = attr.parse().expect("Failed to convert to float");
                self.curr = self.chars.len();
                Ok(Some(Token::Integer(val)))
            }
            6 => {
                let attr = self.chars[self.curr..self.chars.len()]
                    .into_iter()
                    .collect::<String>();
                let val: f32 = attr.parse().expect("Failed to convert to float");
                self.curr = self.chars.len();
                Ok(Some(Token::Float(val)))
            }
            7 => {
                let attr = self.chars[self.curr..self.chars.len()]
                    .into_iter()
                    .collect::<String>();
                self.curr = self.chars.len();
                Ok(Some(Token::StringLiteral(attr)))
            }
            _ => Ok(None),
        }
    }
}
