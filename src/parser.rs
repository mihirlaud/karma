use crate::lexer::{Lexer, Token};
use std::collections::{HashMap, LinkedList};

#[derive(Debug)]
pub struct Node {
    sym: GrammarSymbol,
}

impl Node {
    fn new(sym: GrammarSymbol) -> Self {
        Self { sym }
    }
}

#[derive(Debug)]
pub struct ParseTree {
    node_list: Vec<Node>,
    adj_list: HashMap<usize, Vec<usize>>,
    parents_list: HashMap<usize, usize>,
}

impl ParseTree {
    pub fn new() -> Self {
        Self {
            node_list: vec![],
            adj_list: HashMap::new(),
            parents_list: HashMap::new(),
        }
    }

    pub fn set_root(&mut self, sym: GrammarSymbol) {
        if self.node_list.len() == 0 {
            self.node_list.push(Node::new(sym));
        } else {
            self.node_list[0] = Node::new(sym);
        }
    }

    pub fn add_child(&mut self, idx: usize, sym: GrammarSymbol) -> usize {
        let new_idx = self.node_list.len();
        let new_node = Node::new(sym);
        self.node_list.push(new_node);

        let mut neighbors = match self.adj_list.get(&idx) {
            Some(vec) => vec.clone(),
            None => vec![],
        };
        neighbors.push(new_idx);
        self.adj_list.insert(idx, neighbors);

        self.parents_list.insert(new_idx, idx);

        new_idx
    }

    pub fn get_next_nt_sibling(&self, idx: usize) -> usize {
        let mut idx = idx;
        loop {
            if idx >= self.node_list.len() || idx == 0 {
                return 0;
            }

            let parent = self.parents_list[&idx];
            let siblings = self.adj_list[&parent].clone();

            let mut flag = false;
            for sibling in siblings {
                if flag {
                    match self.node_list[sibling].sym {
                        GrammarSymbol::Nonterminal(_) => {
                            return sibling;
                        }
                        _ => {}
                    }
                }

                if sibling == idx {
                    flag = true;
                }
            }

            idx = parent;
        }
    }
}

pub struct Parser {
    lexer: Lexer,
    pub parse_tree: ParseTree,
}

#[derive(Clone, Debug)]
pub enum GrammarSymbol {
    Terminal(Token),
    Nonterminal(String),
    Empty,
    End,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self {
            lexer,
            parse_tree: ParseTree::new(),
        }
    }

    pub fn parse(&mut self) -> Result<(), String> {
        let mut stack: LinkedList<GrammarSymbol> = LinkedList::new();

        stack.push_back(GrammarSymbol::End);
        stack.push_back(GrammarSymbol::Nonterminal("block".to_string()));

        self.parse_tree
            .set_root(GrammarSymbol::Nonterminal("block".to_string()));
        let mut idx = 0;

        loop {
            let token = self.lexer.next_token();

            while !stack.is_empty() {
                let top = stack.pop_back();
                //println!("{:?}", top);

                match top.unwrap() {
                    GrammarSymbol::Terminal(t) => {
                        if std::mem::discriminant(&token.clone())
                            == std::mem::discriminant(&Some(t.clone()))
                        {
                            break;
                        } else {
                            return Err("syntax error 1".to_string());
                        }
                    }
                    GrammarSymbol::Nonterminal(nt) => {
                        let mut production: Vec<GrammarSymbol> = vec![];

                        match (nt.as_str(), token.clone()) {
                            ("block", Some(Token::LeftBrace)) => {
                                production = vec![
                                    GrammarSymbol::Terminal(Token::LeftBrace),
                                    GrammarSymbol::Nonterminal("stmt_list".to_string()),
                                    GrammarSymbol::Terminal(Token::RightBrace),
                                ];
                                println!("B -> {{ SL }}");
                            }
                            ("stmt_list", Some(Token::Var))
                            | ("stmt_list", Some(Token::Const))
                            | ("stmt_list", Some(Token::While))
                            | ("stmt_list", Some(Token::If))
                            | ("stmt_list", Some(Token::ID(_))) => {
                                production = vec![
                                    GrammarSymbol::Nonterminal("stmt".to_string()),
                                    GrammarSymbol::Nonterminal("stmt_list".to_string()),
                                ];
                                println!("SL -> S SL");
                            }
                            ("stmt_list", Some(Token::RightBrace)) => {
                                println!("SL -> `");
                            }
                            ("stmt", Some(Token::Var)) => {
                                production = vec![
                                    GrammarSymbol::Terminal(Token::Var),
                                    GrammarSymbol::Nonterminal("id".to_string()),
                                    GrammarSymbol::Terminal(Token::Colon),
                                    GrammarSymbol::Nonterminal("id".to_string()),
                                    GrammarSymbol::Terminal(Token::Assign),
                                    GrammarSymbol::Nonterminal("expression".to_string()),
                                    GrammarSymbol::Terminal(Token::Semicolon),
                                ];
                                println!("S -> var IDENTIFIER : IDENTIFIER = E ;");
                            }
                            ("stmt", Some(Token::Const)) => {
                                production = vec![
                                    GrammarSymbol::Terminal(Token::Const),
                                    GrammarSymbol::Nonterminal("id".to_string()),
                                    GrammarSymbol::Terminal(Token::Colon),
                                    GrammarSymbol::Nonterminal("id".to_string()),
                                    GrammarSymbol::Terminal(Token::Assign),
                                    GrammarSymbol::Nonterminal("expression".to_string()),
                                    GrammarSymbol::Terminal(Token::Semicolon),
                                ];
                                println!("S -> const IDENTIFIER : IDENTIFIER = E ;");
                            }
                            ("stmt", Some(Token::ID(_))) => {
                                production = vec![
                                    GrammarSymbol::Nonterminal("id".to_string()),
                                    GrammarSymbol::Terminal(Token::Assign),
                                    GrammarSymbol::Nonterminal("expression".to_string()),
                                    GrammarSymbol::Terminal(Token::Semicolon),
                                ];
                                println!("S -> id = E ;");
                            }
                            ("stmt", Some(Token::While)) => {
                                production = vec![
                                    GrammarSymbol::Terminal(Token::While),
                                    GrammarSymbol::Nonterminal("conditional".to_string()),
                                    GrammarSymbol::Nonterminal("block".to_string()),
                                ];

                                println!("S -> while C B");
                            }
                            ("stmt", Some(Token::If)) => {
                                production = vec![
                                    GrammarSymbol::Terminal(Token::If),
                                    GrammarSymbol::Nonterminal("conditional".to_string()),
                                    GrammarSymbol::Nonterminal("block".to_string()),
                                    GrammarSymbol::Nonterminal("optelse".to_string()),
                                ];
                                println!("S -> if C B optelse");
                            }
                            ("optelse", Some(Token::Else)) => {
                                production = vec![
                                    GrammarSymbol::Terminal(Token::Else),
                                    GrammarSymbol::Nonterminal("block".to_string()),
                                ];
                                println!("optelse -> else B");
                            }
                            ("optelse", Some(Token::Var))
                            | ("optelse", Some(Token::Const))
                            | ("optelse", Some(Token::ID(_)))
                            | ("optelse", Some(Token::While))
                            | ("optelse", Some(Token::If))
                            | ("optelse", Some(Token::RightBrace)) => {
                                println!("optelse -> `");
                            }
                            ("conditional", Some(Token::ID(_)))
                            | ("conditional", Some(Token::LeftParen))
                            | ("conditional", Some(Token::Number(_))) => {
                                production = vec![
                                    GrammarSymbol::Nonterminal("bool_expr".to_string()),
                                    GrammarSymbol::Nonterminal("conditional1".to_string()),
                                ];
                                println!("C -> bE C'");
                            }
                            ("bool_expr", Some(Token::ID(_)))
                            | ("bool_expr", Some(Token::Number(_)))
                            | ("bool_expr", Some(Token::LeftParen)) => {
                                production = vec![
                                    GrammarSymbol::Nonterminal("expression".to_string()),
                                    GrammarSymbol::Nonterminal("comparison".to_string()),
                                    GrammarSymbol::Nonterminal("expression".to_string()),
                                ];

                                println!("bE -> E comp E");
                            }
                            ("comparison", Some(Token::Equals)) => {
                                production = vec![GrammarSymbol::Terminal(Token::Equals)];
                                println!("comp -> ==");
                            }
                            ("comparison", Some(Token::Neq)) => {
                                production = vec![GrammarSymbol::Terminal(Token::Neq)];
                                println!("comp -> !=");
                            }
                            ("comparison", Some(Token::Less)) => {
                                production = vec![GrammarSymbol::Terminal(Token::Less)];
                                println!("comp -> <");
                            }
                            ("comparison", Some(Token::Greater)) => {
                                production = vec![GrammarSymbol::Terminal(Token::Greater)];
                                println!("comp -> >");
                            }
                            ("comparison", Some(Token::Leq)) => {
                                production = vec![GrammarSymbol::Terminal(Token::Leq)];
                                println!("comp -> <=");
                            }
                            ("comparison", Some(Token::Geq)) => {
                                production = vec![GrammarSymbol::Terminal(Token::Geq)];
                                println!("comp -> >=");
                            }
                            ("conditional1", Some(Token::LogicalAnd)) => {
                                production = vec![
                                    GrammarSymbol::Terminal(Token::LogicalAnd),
                                    GrammarSymbol::Nonterminal("bool_expr".to_string()),
                                    GrammarSymbol::Nonterminal("conditional1".to_string()),
                                ];
                                println!("C' -> && bE C'");
                            }
                            ("conditional1", Some(Token::LogicalOr)) => {
                                production = vec![
                                    GrammarSymbol::Terminal(Token::LogicalOr),
                                    GrammarSymbol::Nonterminal("bool_expr".to_string()),
                                    GrammarSymbol::Nonterminal("conditional1".to_string()),
                                ];
                                println!("C' -> || bE C'");
                            }
                            ("conditional1", Some(Token::LeftBrace)) => {
                                println!("C' -> `");
                            }
                            ("expression", Some(Token::ID(_)))
                            | ("expression", Some(Token::Number(_)))
                            | ("expression", Some(Token::LeftParen)) => {
                                production = vec![
                                    GrammarSymbol::Nonterminal("term".to_string()),
                                    GrammarSymbol::Nonterminal("expression1".to_string()),
                                ];
                                println!("E -> T E'");
                            }
                            ("expression1", Some(Token::Add)) => {
                                production = vec![
                                    GrammarSymbol::Terminal(Token::Add),
                                    GrammarSymbol::Nonterminal("term".to_string()),
                                    GrammarSymbol::Nonterminal("expression1".to_string()),
                                ];
                                println!("E' -> + T E'");
                            }
                            ("expression1", Some(Token::Sub)) => {
                                production = vec![
                                    GrammarSymbol::Terminal(Token::Sub),
                                    GrammarSymbol::Nonterminal("term".to_string()),
                                    GrammarSymbol::Nonterminal("expression1".to_string()),
                                ];
                                println!("E' -> - T E'");
                            }
                            ("expression1", Some(Token::RightParen))
                            | ("expression1", Some(Token::Semicolon))
                            | ("expression1", Some(Token::Equals))
                            | ("expression1", Some(Token::Neq))
                            | ("expression1", Some(Token::Less))
                            | ("expression1", Some(Token::Greater))
                            | ("expression1", Some(Token::Leq))
                            | ("expression1", Some(Token::Geq))
                            | ("expression1", Some(Token::LeftBrace)) => {
                                println!("E' -> `");
                            }
                            ("term", Some(Token::ID(_)))
                            | ("term", Some(Token::Number(_)))
                            | ("term", Some(Token::LeftParen)) => {
                                production = vec![
                                    GrammarSymbol::Nonterminal("factor".to_string()),
                                    GrammarSymbol::Nonterminal("term1".to_string()),
                                ];
                                println!("T -> F T'");
                            }
                            ("term1", Some(Token::Mul)) => {
                                production = vec![
                                    GrammarSymbol::Terminal(Token::Mul),
                                    GrammarSymbol::Nonterminal("factor".to_string()),
                                    GrammarSymbol::Nonterminal("term1".to_string()),
                                ];
                                println!("T' -> * F T'");
                            }
                            ("term1", Some(Token::Div)) => {
                                production = vec![
                                    GrammarSymbol::Terminal(Token::Div),
                                    GrammarSymbol::Nonterminal("factor".to_string()),
                                    GrammarSymbol::Nonterminal("term1".to_string()),
                                ];
                                println!("T' -> / F T'");
                            }
                            ("term1", Some(Token::Add))
                            | ("term1", Some(Token::Sub))
                            | ("term1", Some(Token::RightParen))
                            | ("term1", Some(Token::Semicolon))
                            | ("term1", Some(Token::Equals))
                            | ("term1", Some(Token::Neq))
                            | ("term1", Some(Token::Less))
                            | ("term1", Some(Token::Greater))
                            | ("term1", Some(Token::Leq))
                            | ("term1", Some(Token::Geq))
                            | ("term1", Some(Token::LeftBrace)) => {
                                println!("T' -> `");
                            }
                            ("factor", Some(Token::LeftParen)) => {
                                production = vec![
                                    GrammarSymbol::Terminal(Token::LeftParen),
                                    GrammarSymbol::Nonterminal("expression".to_string()),
                                    GrammarSymbol::Terminal(Token::RightParen),
                                ];
                                println!("F -> ( E )");
                            }
                            ("factor", Some(Token::ID(_))) => {
                                production = vec![GrammarSymbol::Nonterminal("id".to_string())];
                                println!("F -> id");
                            }
                            ("factor", Some(Token::Number(num))) => {
                                production = vec![GrammarSymbol::Terminal(Token::Number(num))];
                                println!("F -> NUMBER");
                            }
                            ("id", Some(Token::ID(id))) => {
                                production = vec![GrammarSymbol::Terminal(Token::ID(id.clone()))];
                                println!("id -> IDENTIFIER");
                            }
                            _ => {
                                return Err(format!("syntax error: {:?} {:?}", nt, token));
                            }
                        }

                        if production.len() == 0 {
                            self.parse_tree.add_child(idx, GrammarSymbol::Empty);
                            idx = self.parse_tree.get_next_nt_sibling(idx);
                            continue;
                        }

                        production.iter().rev().for_each(|symbol| {
                            stack.push_back(symbol.clone());
                        });

                        let mut has_nt = false;
                        let mut next_idx = 0;
                        production.iter().for_each(|symbol| {
                            let i = self.parse_tree.add_child(idx, symbol.clone());
                            match symbol.clone() {
                                GrammarSymbol::Nonterminal(_) => {
                                    if !has_nt {
                                        has_nt = true;
                                        next_idx = i;
                                    }
                                }
                                _ => {}
                            }
                        });

                        if has_nt {
                            idx = next_idx as usize;
                        } else {
                            idx = self.parse_tree.get_next_nt_sibling(idx);
                        }
                    }
                    GrammarSymbol::Empty => {}
                    GrammarSymbol::End => {
                        if token == None {
                            break;
                        } else {
                            println!("{}", token.unwrap().clone(),);
                            return Err("syntax error 3".to_string());
                        }
                    }
                }
            }

            if stack.is_empty() && token == None {
                return Ok(());
            }
        }
    }

    pub fn gen(&self) {
        let mut stack: LinkedList<usize> = LinkedList::new();

        stack.push_back(0);
        while !stack.is_empty() {
            let curr = stack.pop_back().unwrap();

            match self.parse_tree.adj_list.get(&curr) {
                Some(children) => {
                    children.iter().rev().for_each(|child| {
                        stack.push_back(*child);
                    });
                }
                None => {
                    println!("{:?}", self.parse_tree.node_list[curr].sym);
                }
            }
        }
    }
}
