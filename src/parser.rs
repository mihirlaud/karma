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
                                stack.push_back(GrammarSymbol::Terminal(Token::RightBrace));
                                stack
                                    .push_back(GrammarSymbol::Nonterminal("stmt_list".to_string()));
                                stack.push_back(GrammarSymbol::Terminal(Token::LeftBrace));
                                println!("B -> {{ SL }}");

                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::LeftBrace));
                                let next_idx = self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("stmt_list".to_string()),
                                );
                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::RightBrace));

                                idx = next_idx;
                            }
                            ("stmt_list", Some(Token::Var))
                            | ("stmt_list", Some(Token::Const))
                            | ("stmt_list", Some(Token::While))
                            | ("stmt_list", Some(Token::If))
                            | ("stmt_list", Some(Token::ID(_))) => {
                                stack
                                    .push_back(GrammarSymbol::Nonterminal("stmt_list".to_string()));
                                stack.push_back(GrammarSymbol::Nonterminal("stmt".to_string()));
                                println!("SL -> S SL");

                                let next_idx = self
                                    .parse_tree
                                    .add_child(idx, GrammarSymbol::Nonterminal("stmt".to_string()));
                                self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("stmt_list".to_string()),
                                );

                                idx = next_idx;
                            }
                            ("stmt_list", Some(Token::RightBrace)) => {
                                println!("SL -> `");

                                self.parse_tree.add_child(idx, GrammarSymbol::Empty);
                                idx = self.parse_tree.get_next_nt_sibling(idx);
                            }
                            ("stmt", Some(Token::Var)) => {
                                stack.push_back(GrammarSymbol::Terminal(Token::Semicolon));
                                stack.push_back(GrammarSymbol::Nonterminal(
                                    "expression".to_string(),
                                ));
                                stack.push_back(GrammarSymbol::Terminal(Token::Assign));
                                stack.push_back(GrammarSymbol::Nonterminal("id".to_string()));
                                stack.push_back(GrammarSymbol::Terminal(Token::Colon));
                                stack.push_back(GrammarSymbol::Nonterminal("id".to_string()));
                                stack.push_back(GrammarSymbol::Terminal(Token::Var));
                                println!("S -> var IDENTIFIER : IDENTIFIER = E ;");

                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::Var));
                                let next_idx = self
                                    .parse_tree
                                    .add_child(idx, GrammarSymbol::Nonterminal("id".to_string()));
                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::Colon));
                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Nonterminal("id".to_string()));
                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::Assign));
                                self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("expression".to_string()),
                                );
                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::Semicolon));

                                idx = next_idx;
                            }
                            ("stmt", Some(Token::Const)) => {
                                stack.push_back(GrammarSymbol::Terminal(Token::Semicolon));
                                stack.push_back(GrammarSymbol::Nonterminal(
                                    "expression".to_string(),
                                ));
                                stack.push_back(GrammarSymbol::Terminal(Token::Assign));
                                stack.push_back(GrammarSymbol::Nonterminal("id".to_string()));
                                stack.push_back(GrammarSymbol::Terminal(Token::Colon));
                                stack.push_back(GrammarSymbol::Nonterminal("id".to_string()));
                                stack.push_back(GrammarSymbol::Terminal(Token::Const));
                                println!("S -> const IDENTIFIER : IDENTIFIER = E ;");

                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::Const));
                                let next_idx = self
                                    .parse_tree
                                    .add_child(idx, GrammarSymbol::Nonterminal("id".to_string()));
                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::Colon));
                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Nonterminal("id".to_string()));
                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::Assign));
                                self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("expression".to_string()),
                                );
                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::Semicolon));

                                idx = next_idx;
                            }
                            ("stmt", Some(Token::ID(_))) => {
                                stack.push_back(GrammarSymbol::Terminal(Token::Semicolon));
                                stack.push_back(GrammarSymbol::Nonterminal(
                                    "expression".to_string(),
                                ));
                                stack.push_back(GrammarSymbol::Terminal(Token::Assign));
                                stack.push_back(GrammarSymbol::Nonterminal("id".to_string()));
                                println!("S -> id = E ;");

                                let next_idx = self
                                    .parse_tree
                                    .add_child(idx, GrammarSymbol::Nonterminal("id".to_string()));
                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::Assign));
                                self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("expression".to_string()),
                                );
                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::Semicolon));

                                idx = next_idx;
                            }
                            ("stmt", Some(Token::While)) => {
                                stack.push_back(GrammarSymbol::Nonterminal("block".to_string()));
                                stack.push_back(GrammarSymbol::Nonterminal(
                                    "conditional".to_string(),
                                ));
                                stack.push_back(GrammarSymbol::Terminal(Token::While));
                                println!("S -> while C B");

                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::While));
                                let next_idx = self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("conditional".to_string()),
                                );
                                self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("block".to_string()),
                                );

                                idx = next_idx;
                            }
                            ("stmt", Some(Token::If)) => {
                                stack.push_back(GrammarSymbol::Nonterminal("optelse".to_string()));
                                stack.push_back(GrammarSymbol::Nonterminal("block".to_string()));
                                stack.push_back(GrammarSymbol::Nonterminal(
                                    "conditional".to_string(),
                                ));
                                stack.push_back(GrammarSymbol::Terminal(Token::If));
                                println!("S -> if C B optelse");

                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::If));
                                let next_idx = self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("conditional".to_string()),
                                );
                                self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("block".to_string()),
                                );
                                self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("optelse".to_string()),
                                );

                                idx = next_idx;
                            }
                            ("optelse", Some(Token::Else)) => {
                                stack.push_back(GrammarSymbol::Nonterminal("block".to_string()));
                                stack.push_back(GrammarSymbol::Terminal(Token::Else));
                                println!("optelse -> else B");

                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::Else));
                                let next_idx = self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("block".to_string()),
                                );

                                idx = next_idx;
                            }
                            ("optelse", Some(Token::Var))
                            | ("optelse", Some(Token::Const))
                            | ("optelse", Some(Token::ID(_)))
                            | ("optelse", Some(Token::While))
                            | ("optelse", Some(Token::If))
                            | ("optelse", Some(Token::RightBrace)) => {
                                println!("optelse -> `");

                                self.parse_tree.add_child(idx, GrammarSymbol::Empty);
                                idx = self.parse_tree.get_next_nt_sibling(idx);
                            }
                            ("conditional", Some(Token::ID(_)))
                            | ("conditional", Some(Token::LeftParen))
                            | ("conditional", Some(Token::Number(_))) => {
                                stack.push_back(GrammarSymbol::Nonterminal(
                                    "conditional1".to_string(),
                                ));
                                stack
                                    .push_back(GrammarSymbol::Nonterminal("bool_expr".to_string()));
                                println!("C -> bE C'");

                                let next_idx = self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("bool_expr".to_string()),
                                );
                                self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("conditional1".to_string()),
                                );

                                idx = next_idx;
                            }
                            ("bool_expr", Some(Token::ID(_)))
                            | ("bool_expr", Some(Token::Number(_)))
                            | ("bool_expr", Some(Token::LeftParen)) => {
                                stack.push_back(GrammarSymbol::Nonterminal(
                                    "expression".to_string(),
                                ));
                                stack.push_back(GrammarSymbol::Nonterminal(
                                    "comparison".to_string(),
                                ));
                                stack.push_back(GrammarSymbol::Nonterminal(
                                    "expression".to_string(),
                                ));
                                println!("bE -> E comp E");

                                let next_idx = self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("expression".to_string()),
                                );
                                self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("comparison".to_string()),
                                );

                                self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("expression".to_string()),
                                );

                                idx = next_idx;
                            }
                            ("comparison", Some(Token::Equals)) => {
                                stack.push_back(GrammarSymbol::Terminal(Token::Equals));
                                println!("comp -> ==");

                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::Equals));

                                idx = self.parse_tree.get_next_nt_sibling(idx);
                            }
                            ("comparison", Some(Token::Neq)) => {
                                stack.push_back(GrammarSymbol::Terminal(Token::Neq));
                                println!("comp -> !=");

                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::Neq));
                                idx = self.parse_tree.get_next_nt_sibling(idx);
                            }
                            ("comparison", Some(Token::Less)) => {
                                stack.push_back(GrammarSymbol::Terminal(Token::Less));
                                println!("comp -> <");

                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::Less));
                                idx = self.parse_tree.get_next_nt_sibling(idx);
                            }
                            ("comparison", Some(Token::Greater)) => {
                                stack.push_back(GrammarSymbol::Terminal(Token::Greater));
                                println!("comp -> >");

                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::Greater));
                                idx = self.parse_tree.get_next_nt_sibling(idx);
                            }
                            ("comparison", Some(Token::Leq)) => {
                                stack.push_back(GrammarSymbol::Terminal(Token::Leq));
                                println!("comp -> <=");

                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::Leq));
                                idx = self.parse_tree.get_next_nt_sibling(idx);
                            }
                            ("comparison", Some(Token::Geq)) => {
                                stack.push_back(GrammarSymbol::Terminal(Token::Geq));
                                println!("comp -> >=");

                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::Geq));
                                idx = self.parse_tree.get_next_nt_sibling(idx);
                            }
                            ("conditional1", Some(Token::LogicalAnd)) => {
                                stack.push_back(GrammarSymbol::Nonterminal(
                                    "conditional1".to_string(),
                                ));
                                stack
                                    .push_back(GrammarSymbol::Nonterminal("bool_expr".to_string()));
                                stack.push_back(GrammarSymbol::Terminal(Token::LogicalAnd));
                                println!("C' -> && bE C'");

                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::LogicalAnd));
                                let next_idx = self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("bool_expr".to_string()),
                                );
                                self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("conditional1".to_string()),
                                );

                                idx = next_idx;
                            }
                            ("conditional1", Some(Token::LogicalOr)) => {
                                stack.push_back(GrammarSymbol::Nonterminal(
                                    "conditional1".to_string(),
                                ));
                                stack
                                    .push_back(GrammarSymbol::Nonterminal("bool_expr".to_string()));
                                stack.push_back(GrammarSymbol::Terminal(Token::LogicalOr));
                                println!("C' -> || bE C'");

                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::LogicalOr));
                                let next_idx = self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("bool_expr".to_string()),
                                );
                                self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("conditional1".to_string()),
                                );

                                idx = next_idx;
                            }
                            ("conditional1", Some(Token::LeftBrace)) => {
                                println!("C' -> `");

                                self.parse_tree.add_child(idx, GrammarSymbol::Empty);
                                idx = self.parse_tree.get_next_nt_sibling(idx);
                            }
                            ("expression", Some(Token::ID(_)))
                            | ("expression", Some(Token::Number(_)))
                            | ("expression", Some(Token::LeftParen)) => {
                                stack.push_back(GrammarSymbol::Nonterminal(
                                    "expression1".to_string(),
                                ));
                                stack.push_back(GrammarSymbol::Nonterminal("term".to_string()));
                                println!("E -> T E'");

                                let next_idx = self
                                    .parse_tree
                                    .add_child(idx, GrammarSymbol::Nonterminal("term".to_string()));
                                self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("expression1".to_string()),
                                );

                                idx = next_idx;
                            }
                            ("expression1", Some(Token::Add)) => {
                                stack.push_back(GrammarSymbol::Nonterminal(
                                    "expression1".to_string(),
                                ));
                                stack.push_back(GrammarSymbol::Nonterminal("term".to_string()));
                                stack.push_back(GrammarSymbol::Terminal(Token::Add));
                                println!("E' -> + T E'");

                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::Add));
                                let next_idx = self
                                    .parse_tree
                                    .add_child(idx, GrammarSymbol::Nonterminal("term".to_string()));
                                self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("expression1".to_string()),
                                );

                                idx = next_idx;
                            }
                            ("expression1", Some(Token::Sub)) => {
                                stack.push_back(GrammarSymbol::Nonterminal(
                                    "expression1".to_string(),
                                ));
                                stack.push_back(GrammarSymbol::Nonterminal("term".to_string()));
                                stack.push_back(GrammarSymbol::Terminal(Token::Sub));
                                println!("E' -> - T E'");

                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::Sub));
                                let next_idx = self
                                    .parse_tree
                                    .add_child(idx, GrammarSymbol::Nonterminal("term".to_string()));
                                self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("expression1".to_string()),
                                );

                                idx = next_idx;
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

                                self.parse_tree.add_child(idx, GrammarSymbol::Empty);
                                idx = self.parse_tree.get_next_nt_sibling(idx);
                            }
                            ("term", Some(Token::ID(_)))
                            | ("term", Some(Token::Number(_)))
                            | ("term", Some(Token::LeftParen)) => {
                                stack.push_back(GrammarSymbol::Nonterminal("term1".to_string()));
                                stack.push_back(GrammarSymbol::Nonterminal("factor".to_string()));
                                println!("T -> F T'");

                                let next_idx = self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("factor".to_string()),
                                );
                                self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("term1".to_string()),
                                );

                                idx = next_idx;
                            }
                            ("term1", Some(Token::Mul)) => {
                                stack.push_back(GrammarSymbol::Nonterminal("term1".to_string()));
                                stack.push_back(GrammarSymbol::Nonterminal("factor".to_string()));
                                stack.push_back(GrammarSymbol::Terminal(Token::Mul));
                                println!("T' -> * F T'");

                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::Mul));
                                let next_idx = self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("factor".to_string()),
                                );
                                self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("term1".to_string()),
                                );

                                idx = next_idx;
                            }
                            ("term1", Some(Token::Div)) => {
                                stack.push_back(GrammarSymbol::Nonterminal("term1".to_string()));
                                stack.push_back(GrammarSymbol::Nonterminal("factor".to_string()));
                                stack.push_back(GrammarSymbol::Terminal(Token::Div));
                                println!("T' -> / F T'");

                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::Div));
                                let next_idx = self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("factor".to_string()),
                                );
                                self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("term1".to_string()),
                                );

                                idx = next_idx;
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

                                self.parse_tree.add_child(idx, GrammarSymbol::Empty);
                                idx = self.parse_tree.get_next_nt_sibling(idx);
                            }
                            ("factor", Some(Token::LeftParen)) => {
                                stack.push_back(GrammarSymbol::Terminal(Token::RightParen));
                                stack.push_back(GrammarSymbol::Nonterminal(
                                    "expression".to_string(),
                                ));
                                stack.push_back(GrammarSymbol::Terminal(Token::LeftParen));
                                println!("F -> ( E )");

                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::LeftParen));
                                let next_idx = self.parse_tree.add_child(
                                    idx,
                                    GrammarSymbol::Nonterminal("expression".to_string()),
                                );
                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::RightParen));

                                idx = next_idx;
                            }
                            ("factor", Some(Token::ID(_))) => {
                                stack.push_back(GrammarSymbol::Nonterminal("id".to_string()));
                                println!("F -> id");

                                idx = self
                                    .parse_tree
                                    .add_child(idx, GrammarSymbol::Nonterminal("id".to_string()));
                            }
                            ("factor", Some(Token::Number(num))) => {
                                stack.push_back(GrammarSymbol::Terminal(Token::Number(num)));
                                println!("F -> NUMBER");

                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::Number(num)));

                                idx = self.parse_tree.get_next_nt_sibling(idx);
                            }
                            ("id", Some(Token::ID(id))) => {
                                stack.push_back(GrammarSymbol::Terminal(Token::ID(id.clone())));
                                println!("id -> IDENTIFIER");

                                self.parse_tree
                                    .add_child(idx, GrammarSymbol::Terminal(Token::ID(id)));
                                idx = self.parse_tree.get_next_nt_sibling(idx);
                            }
                            _ => {
                                return Err(format!("syntax error: {:?} {:?}", nt, token));
                            }
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
