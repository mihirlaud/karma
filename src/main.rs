use std::fmt::Display;
use std::io::BufReader;
use std::fs::File;
use utf8_chars::BufReadCharsExt;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
enum Keyword {
    Node,
    Export,
    Var,
    Const,
    Fn,
    While,
    True,
    False,
}

enum Token {
    ID(String),
    Number(u32),
    ReservedKeyword(Keyword),
    Symbol(String),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::ID(name) => write!(f, "ID: {name}"),
            Token::Number(num) => write!(f, "Number: {num}"),
            Token::ReservedKeyword(kw) => write!(f, "Keyword: {kw:?}"),
            Token::Symbol(sym) => write!(f, "Symbol: {sym}"),
        }
    }
}

fn main() {

    let mut f = BufReader::new(
        File::open("code/current_snippet.krm")
        .expect("Could not open file")
    );

    let mut tokens = vec![];

    let keywords = HashMap::from([
        ("node", Keyword::Node), ("export", Keyword::Export),
        ("var", Keyword::Var), ("const", Keyword::Const),
        ("fn", Keyword::Fn), ("while", Keyword::While),
        ("true", Keyword::True), ("false", Keyword::False),
    ]);

    let char_stream: Vec<char> = f.chars().map(|x| x.unwrap()).collect();

    let mut ptr = 0;
    let mut c;
    while ptr < char_stream.len() {
        c = char_stream[ptr];

        loop {
            if c.is_ascii_whitespace() {
                ptr += 1;
                c = char_stream[ptr];
                continue;
            }
            break;
        };

        if c.is_ascii_digit() {
            let mut v = c.to_digit(10).unwrap();
            ptr += 1;
            c = char_stream[ptr];
            
            while c.is_ascii_digit() {
                v = v * 10 + c.to_digit(10).unwrap();
                ptr += 1;
                if ptr < char_stream.len() {
                    c = char_stream[ptr];
                } else {
                    break;
                }
            }

            tokens.push(Token::Number(v));
            continue;
        }
        
        if c.is_ascii_alphabetic() {
            let mut b = String::new();
            b.push(c);
            ptr += 1;
            c = char_stream[ptr];

            while c.is_ascii_alphanumeric() {
                b.push(c);
                ptr += 1;
                if ptr < char_stream.len() {
                    c = char_stream[ptr];
                } else {
                    break;
                }
            }

            if keywords.contains_key(b.as_str()) {
                tokens.push(Token::ReservedKeyword(keywords[b.as_str()]))
            } else {
                tokens.push(Token::ID(b));
            }
            continue;
        }

        
        let mut b = String::new();
        b.push(c);
        if c == '-' {
            if ptr < char_stream.len() - 1 && char_stream[ptr + 1] == '>' {
                ptr += 1;
                c = char_stream[ptr];
                b.push(c);
            }
        }

        if c == '-' || c == '+' || c == '*' || c == '/' || c == '=' || c == '!' {
            if ptr < char_stream.len() - 1 && char_stream[ptr + 1] == '=' {
                ptr += 1;
                c = char_stream[ptr];
                b.push(c);
            }
        }

        if c == ':' {
            if ptr < char_stream.len() - 1 && char_stream[ptr + 1] == ':' {
                ptr += 1;
                c = char_stream[ptr];
                b.push(c);
            }
        }

        if c == '/' {
            if ptr < char_stream.len() - 1 && char_stream[ptr + 1] == '/' {
                loop {
                    if c != '\n' {
                        ptr += 1;
                        c = char_stream[ptr];
                        continue;
                    }
                    break;
                };
                continue;
            }

        }

        tokens.push(Token::Symbol(b));

        ptr += 1;
    }

    for token in tokens {
        println!("{token}");
    }

}