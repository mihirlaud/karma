use std::collections::{LinkedList, HashMap};
use crate::lexer::{Lexer, Token};

#[derive(Debug)]
pub struct Node {
    idx: usize,
    sym: GrammarSymbol,
}

impl Node {
    fn new(idx: usize, sym: GrammarSymbol) -> Self {
        Self {
            idx,
            sym,
        }
    }
}

#[derive(Debug)]
pub struct SyntaxTree {
    node_list: Vec<Node>,
    adj_list: HashMap<usize, Vec<usize>>,
    parents_list: HashMap<usize, usize>,
}

impl SyntaxTree {
    pub fn new() -> Self {
        Self {
            node_list: vec![],
            adj_list: HashMap::new(),
            parents_list: HashMap::new(),
        }
    }

    pub fn set_root(&mut self, sym: GrammarSymbol) {
        if self.node_list.len() == 0 {
            self.node_list.push(Node::new(0, sym));
        } else {
            self.node_list[0] = Node::new(0, sym);
        }
    } 

    pub fn add_child(&mut self, idx: usize, sym: GrammarSymbol) -> usize {
        let new_idx = self.node_list.len();
        let new_node = Node::new(new_idx, sym);
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
                        },
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
    pub tree: SyntaxTree,
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
            tree: SyntaxTree::new(),
        }
    }

    pub fn parse(&mut self) -> Result<(), String> {
        let mut stack: LinkedList<GrammarSymbol> = LinkedList::new();
        
        stack.push_back(GrammarSymbol::End);
        stack.push_back(GrammarSymbol::Nonterminal("stmt".to_string()));

        self.tree.set_root(GrammarSymbol::Nonterminal("stmt".to_string()));
        let mut idx = 0;

        loop {
            let token = self.lexer.next_token();
            
            while !stack.is_empty() {
                let top = stack.pop_back();
                //println!("{:?}", top);
    
                match top.unwrap() {
                    GrammarSymbol::Terminal(t) => {
                        if std::mem::discriminant(&token.clone()) == std::mem::discriminant(&Some(t.clone())) {
                            break;
                        } else {
                            return Err("syntax error 1".to_string());
                        }
                    }
                    GrammarSymbol::Nonterminal(nt) => {
                        match (nt.as_str(), token.clone()) {
                            ("stmt", Some(Token::Var)) => {
                                stack.push_back(GrammarSymbol::Terminal(Token::Semicolon));
                                stack.push_back(GrammarSymbol::Nonterminal("expression".to_string()));
                                stack.push_back(GrammarSymbol::Terminal(Token::Assign));
                                stack.push_back(GrammarSymbol::Terminal(Token::ID("".to_string())));
                                stack.push_back(GrammarSymbol::Terminal(Token::Colon));
                                stack.push_back(GrammarSymbol::Terminal(Token::ID("".to_string())));
                                stack.push_back(GrammarSymbol::Terminal(Token::Var));
                                println!("S -> var IDENTIFIER : IDENTIFIER = E ;");

                                
                                self.tree.add_child(idx, GrammarSymbol::Terminal(Token::Var));
                                self.tree.add_child(idx, GrammarSymbol::Terminal(Token::ID("".to_string())));
                                self.tree.add_child(idx, GrammarSymbol::Terminal(Token::Colon));
                                self.tree.add_child(idx, GrammarSymbol::Terminal(Token::ID("".to_string())));
                                self.tree.add_child(idx, GrammarSymbol::Terminal(Token::Assign));
                                let next_idx = self.tree.add_child(idx, GrammarSymbol::Nonterminal("expression".to_string()));
                                self.tree.add_child(idx, GrammarSymbol::Terminal(Token::Semicolon));

                                idx = next_idx;
                            }
                            ("expression", Some(Token::ID(_))) => {
                                stack.push_back(GrammarSymbol::Nonterminal("expression1".to_string()));
                                stack.push_back(GrammarSymbol::Nonterminal("term".to_string()));
                                println!("E -> T E'");

                                let next_idx = self.tree.add_child(idx, GrammarSymbol::Nonterminal("term".to_string()));
                                self.tree.add_child(idx, GrammarSymbol::Nonterminal("expression1".to_string()));

                                idx = next_idx;
                            }
                            ("expression", Some(Token::Number(_))) => {
                                stack.push_back(GrammarSymbol::Nonterminal("expression1".to_string()));
                                stack.push_back(GrammarSymbol::Nonterminal("term".to_string()));
                                println!("E -> T E'");

                                let next_idx = self.tree.add_child(idx, GrammarSymbol::Nonterminal("term".to_string()));
                                self.tree.add_child(idx, GrammarSymbol::Nonterminal("expression1".to_string()));

                                idx = next_idx;
                            }
                            ("expression", Some(Token::LeftParen)) => {
                                stack.push_back(GrammarSymbol::Nonterminal("expression1".to_string()));
                                stack.push_back(GrammarSymbol::Nonterminal("term".to_string()));
                                println!("E -> T E'");

                                let next_idx = self.tree.add_child(idx, GrammarSymbol::Nonterminal("term".to_string()));
                                self.tree.add_child(idx, GrammarSymbol::Nonterminal("expression1".to_string()));

                                idx = next_idx;
                            }
                            ("expression1", Some(Token::Add)) => {
                                stack.push_back(GrammarSymbol::Nonterminal("expression1".to_string()));
                                stack.push_back(GrammarSymbol::Nonterminal("term".to_string()));
                                stack.push_back(GrammarSymbol::Terminal(Token::Add));
                                println!("E' -> + T E'");

                                self.tree.add_child(idx, GrammarSymbol::Terminal(Token::Add));
                                let next_idx = self.tree.add_child(idx, GrammarSymbol::Nonterminal("term".to_string()));
                                self.tree.add_child(idx, GrammarSymbol::Nonterminal("expression1".to_string()));

                                idx = next_idx;
                            }
                            ("expression1", Some(Token::Sub)) => {
                                stack.push_back(GrammarSymbol::Nonterminal("expression1".to_string()));
                                stack.push_back(GrammarSymbol::Nonterminal("term".to_string()));
                                stack.push_back(GrammarSymbol::Terminal(Token::Sub));
                                println!("E' -> - T E'");

                                self.tree.add_child(idx, GrammarSymbol::Terminal(Token::Sub));
                                let next_idx = self.tree.add_child(idx, GrammarSymbol::Nonterminal("term".to_string()));
                                self.tree.add_child(idx, GrammarSymbol::Nonterminal("expression1".to_string()));

                                idx = next_idx;
                            }
                            ("expression1", Some(Token::RightParen)) => {
                                println!("E' -> `");

                                self.tree.add_child(idx, GrammarSymbol::Empty);
                                idx = self.tree.get_next_nt_sibling(idx);

                            }
                            ("expression1", Some(Token::Semicolon)) => {
                                println!("E' -> `");

                                self.tree.add_child(idx, GrammarSymbol::Empty);
                                idx = self.tree.get_next_nt_sibling(idx);
                            }
                            ("term", Some(Token::ID(_))) => {
                                stack.push_back(GrammarSymbol::Nonterminal("term1".to_string()));
                                stack.push_back(GrammarSymbol::Nonterminal("factor".to_string()));
                                println!("T -> F T'");

                                let next_idx = self.tree.add_child(idx, GrammarSymbol::Nonterminal("factor".to_string()));
                                self.tree.add_child(idx, GrammarSymbol::Nonterminal("term1".to_string()));

                                idx = next_idx;
                            }
                            ("term", Some(Token::Number(_))) => {
                                stack.push_back(GrammarSymbol::Nonterminal("term1".to_string()));
                                stack.push_back(GrammarSymbol::Nonterminal("factor".to_string()));
                                println!("T -> F T'");

                                let next_idx = self.tree.add_child(idx, GrammarSymbol::Nonterminal("factor".to_string()));
                                self.tree.add_child(idx, GrammarSymbol::Nonterminal("term1".to_string()));

                                idx = next_idx;
                            }
                            ("term", Some(Token::LeftParen)) => {
                                stack.push_back(GrammarSymbol::Nonterminal("term1".to_string()));
                                stack.push_back(GrammarSymbol::Nonterminal("factor".to_string()));
                                println!("T -> F T'");

                                let next_idx = self.tree.add_child(idx, GrammarSymbol::Nonterminal("factor".to_string()));
                                self.tree.add_child(idx, GrammarSymbol::Nonterminal("term1".to_string()));

                                idx = next_idx;
                            }
                            ("term1", Some(Token::Mul)) => {
                                stack.push_back(GrammarSymbol::Nonterminal("term1".to_string()));
                                stack.push_back(GrammarSymbol::Nonterminal("factor".to_string()));
                                stack.push_back(GrammarSymbol::Terminal(Token::Mul));
                                println!("T' -> * F T'");
                                
                                self.tree.add_child(idx, GrammarSymbol::Terminal(Token::Mul));
                                let next_idx = self.tree.add_child(idx, GrammarSymbol::Nonterminal("factor".to_string()));
                                self.tree.add_child(idx, GrammarSymbol::Nonterminal("term1".to_string()));

                                idx = next_idx;
                            }
                            ("term1", Some(Token::Div)) => {
                                stack.push_back(GrammarSymbol::Nonterminal("term1".to_string()));
                                stack.push_back(GrammarSymbol::Nonterminal("factor".to_string()));
                                stack.push_back(GrammarSymbol::Terminal(Token::Div));
                                println!("T' -> / F T'");

                                self.tree.add_child(idx, GrammarSymbol::Terminal(Token::Div));
                                let next_idx = self.tree.add_child(idx, GrammarSymbol::Nonterminal("factor".to_string()));
                                self.tree.add_child(idx, GrammarSymbol::Nonterminal("term1".to_string()));

                                idx = next_idx;
                            }
                            ("term1", Some(Token::Add)) => {
                                println!("T' -> `");

                                self.tree.add_child(idx, GrammarSymbol::Empty);
                                idx = self.tree.get_next_nt_sibling(idx);
                            }
                            ("term1", Some(Token::Sub)) => {
                                println!("T' -> `");

                                self.tree.add_child(idx, GrammarSymbol::Empty);
                                idx = self.tree.get_next_nt_sibling(idx);
                            }
                            ("term1", Some(Token::RightParen)) => {
                                println!("T' -> `");

                                self.tree.add_child(idx, GrammarSymbol::Empty);
                                idx = self.tree.get_next_nt_sibling(idx);
                            }
                            ("term1", Some(Token::Semicolon)) => {
                                println!("T' -> `");

                                self.tree.add_child(idx, GrammarSymbol::Empty);
                                idx = self.tree.get_next_nt_sibling(idx);
                            }
                            ("factor", Some(Token::LeftParen)) => {
                                stack.push_back(GrammarSymbol::Terminal(Token::RightParen));
                                stack.push_back(GrammarSymbol::Nonterminal("expression".to_string()));
                                stack.push_back(GrammarSymbol::Terminal(Token::LeftParen));
                                println!("F -> ( E )");
                                
                                self.tree.add_child(idx, GrammarSymbol::Terminal(Token::LeftParen));
                                let next_idx = self.tree.add_child(idx, GrammarSymbol::Nonterminal("expression".to_string()));
                                self.tree.add_child(idx, GrammarSymbol::Terminal(Token::RightParen));

                                idx = next_idx;
                            }
                            ("factor", Some(Token::ID(id))) => {
                                stack.push_back(GrammarSymbol::Terminal(Token::ID(id.clone())));
                                println!("F -> IDENTIFIER");
                                
                                self.tree.add_child(idx, GrammarSymbol::Terminal(Token::ID(id)));

                                self.tree.add_child(idx, GrammarSymbol::Empty);
                                idx = self.tree.get_next_nt_sibling(idx);
                            }
                            ("factor", Some(Token::Number(num))) => {
                                stack.push_back(GrammarSymbol::Terminal(Token::Number(num)));
                                println!("F -> NUMBER");

                                self.tree.add_child(idx, GrammarSymbol::Terminal(Token::Number(num)));

                                self.tree.add_child(idx, GrammarSymbol::Empty);
                                idx = self.tree.get_next_nt_sibling(idx);
                            }
                            _ => {
                                return Err(format!("syntax error: {:?} {:?}", nt, token));
                            }
                        }
                    }
                    GrammarSymbol::Empty => {},
                    GrammarSymbol::End => {
                        if token == None {
                            break;
                        } else {
                            return Err("syntax error 3".to_string());
                        }
                    },
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

            match self.tree.adj_list.get(&curr) {
                Some(children) => {
                    children.iter().rev().for_each(|child| {
                        stack.push_back(*child);
                    });
                }
                None => {
                    println!("{:?}", self.tree.node_list[curr].sym);
                },
            }
        }

    }
}