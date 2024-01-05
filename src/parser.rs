use crate::lexer::{Lexer, Token};
use std::collections::{HashMap, LinkedList};

#[derive(Clone, Debug, PartialEq)]
pub enum SyntaxTreeNode {
    NodeSeq,
    DeclareNode,
    NodeHeader,
    NodeList,
    TLStmtSeq,
    DeclareFunc,
    ParamList,
    Param,
    ReturnType,
    Void,
    NoReturn,
    StmtSeq,
    DeclareVar,
    DeclareConst,
    ReturnValue,
    WhileLoop,
    IfStmt,
    Assign,
    FnCall,
    InputList,
    AddOp,
    SubOp,
    MulOp,
    DivOp,
    OrOp,
    AndOp,
    CompEq,
    CompNeq,
    CompLess,
    CompGreater,
    CompLeq,
    CompGeq,
    Integer(i32),
    Float(f64),
    Identifier(String),
    Null,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AbstractSyntaxTree {
    pub node: SyntaxTreeNode,
    pub children: Vec<AbstractSyntaxTree>,
}

impl AbstractSyntaxTree {
    pub fn new() -> Self {
        Self {
            node: SyntaxTreeNode::Null,
            children: vec![],
        }
    }
}

#[derive(Debug)]
pub struct ParseTree {
    node_list: Vec<GrammarSymbol>,
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
            self.node_list.push(sym);
        } else {
            self.node_list[0] = sym;
        }
    }

    pub fn get_node(&self, idx: usize) -> GrammarSymbol {
        self.node_list[idx].clone()
    }

    pub fn get_children(&self, idx: usize) -> Vec<usize> {
        match self.adj_list.get(&idx) {
            Some(children) => children.clone(),
            None => vec![],
        }
    }

    pub fn add_child(&mut self, idx: usize, sym: GrammarSymbol) -> usize {
        let new_idx = self.node_list.len();
        let new_node = sym;
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
                    match self.node_list[sibling] {
                        GrammarSymbol::Terminal(_) | GrammarSymbol::Empty | GrammarSymbol::End => {}
                        _ => {
                            return sibling;
                        }
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
    pub ast: AbstractSyntaxTree,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GrammarSymbol {
    Terminal(Token),
    Empty,
    End,
    AssignOrFnCall,
    Block,
    BoolExpr,
    BoolTerm,
    BoolTerm1,
    Comparison,
    Conditional,
    Conditional1,
    Expression,
    Expression1,
    Factor,
    Func,
    ID,
    IDOrFn,
    InputList,
    InputRest,
    NodeBlock,
    NodeHeader,
    NodeList,
    NodeNT,
    NodeRest,
    OptElse,
    OptIDList,
    Param,
    ParamList,
    ParamRest,
    Program,
    ReturnType,
    Stmt,
    StmtList,
    Term,
    Term1,
    TLStmt,
    TLStmtList,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self {
            lexer,
            parse_tree: ParseTree::new(),
            ast: AbstractSyntaxTree::new(),
        }
    }

    pub fn parse(&mut self) -> Result<(), String> {
        let mut stack: LinkedList<GrammarSymbol> = LinkedList::new();

        stack.push_back(GrammarSymbol::End);
        stack.push_back(GrammarSymbol::Program);

        self.parse_tree.set_root(GrammarSymbol::Program);
        let mut idx = 0;

        loop {
            let token = self.lexer.next_token();

            while !stack.is_empty() {
                let top = stack.pop_back();
                //// println!("{:?}", top);

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
                    GrammarSymbol::Empty => {}
                    GrammarSymbol::End => {
                        if token == None {
                            break;
                        } else {
                            // println!("{}", token.unwrap().clone(),);
                            return Err("syntax error 3".to_string());
                        }
                    }
                    nt => {
                        let production: Vec<GrammarSymbol> = match nt {
                            GrammarSymbol::Terminal(_)
                            | GrammarSymbol::Empty
                            | GrammarSymbol::End => {
                                return Err(format!("syntax error: {:?} {:?}", nt, token));
                            }
                            GrammarSymbol::AssignOrFnCall => match token.clone() {
                                Some(Token::Assign) => {
                                    vec![
                                        GrammarSymbol::Terminal(Token::Assign),
                                        GrammarSymbol::Expression,
                                        GrammarSymbol::Terminal(Token::Semicolon),
                                    ]
                                }
                                Some(Token::LeftParen) => {
                                    vec![
                                        GrammarSymbol::Terminal(Token::LeftParen),
                                        GrammarSymbol::InputList,
                                        GrammarSymbol::Terminal(Token::RightParen),
                                        GrammarSymbol::Terminal(Token::Semicolon),
                                    ]
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::Block => match token.clone() {
                                Some(Token::LeftBrace) => vec![
                                    GrammarSymbol::Terminal(Token::LeftBrace),
                                    GrammarSymbol::StmtList,
                                    GrammarSymbol::Terminal(Token::RightBrace),
                                ],
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::BoolExpr => match token.clone() {
                                Some(Token::ID(_))
                                | Some(Token::Integer(_))
                                | Some(Token::Float(_))
                                | Some(Token::LeftParen) => {
                                    vec![
                                        GrammarSymbol::Expression,
                                        GrammarSymbol::Comparison,
                                        GrammarSymbol::Expression,
                                    ]
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::BoolTerm => match token.clone() {
                                Some(Token::ID(_))
                                | Some(Token::LeftParen)
                                | Some(Token::Integer(_))
                                | Some(Token::Float(_)) => {
                                    vec![GrammarSymbol::BoolExpr, GrammarSymbol::BoolTerm1]
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::BoolTerm1 => match token.clone() {
                                Some(Token::LogicalAnd) => {
                                    vec![
                                        GrammarSymbol::Terminal(Token::LogicalAnd),
                                        GrammarSymbol::BoolExpr,
                                        GrammarSymbol::Conditional1,
                                    ]
                                }
                                Some(Token::LeftBrace) | Some(Token::LogicalOr) => {
                                    vec![]
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::Comparison => match token.clone() {
                                Some(Token::Equals) => vec![GrammarSymbol::Terminal(Token::Equals)],
                                Some(Token::Neq) => vec![GrammarSymbol::Terminal(Token::Neq)],
                                Some(Token::Less) => vec![GrammarSymbol::Terminal(Token::Less)],
                                Some(Token::Greater) => {
                                    vec![GrammarSymbol::Terminal(Token::Greater)]
                                }
                                Some(Token::Leq) => vec![GrammarSymbol::Terminal(Token::Leq)],
                                Some(Token::Geq) => vec![GrammarSymbol::Terminal(Token::Geq)],
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::Conditional => match token.clone() {
                                Some(Token::ID(_))
                                | Some(Token::LeftParen)
                                | Some(Token::Integer(_))
                                | Some(Token::Float(_)) => {
                                    vec![GrammarSymbol::BoolTerm, GrammarSymbol::Conditional1]
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::Conditional1 => match token.clone() {
                                Some(Token::LogicalOr) => {
                                    vec![
                                        GrammarSymbol::Terminal(Token::LogicalOr),
                                        GrammarSymbol::BoolTerm,
                                        GrammarSymbol::Conditional1,
                                    ]
                                }
                                Some(Token::LeftBrace) => {
                                    vec![]
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::Expression => match token.clone() {
                                Some(Token::ID(_))
                                | Some(Token::Integer(_))
                                | Some(Token::Float(_))
                                | Some(Token::LeftParen) => {
                                    vec![GrammarSymbol::Term, GrammarSymbol::Expression1]
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::Expression1 => match token.clone() {
                                Some(Token::RightParen)
                                | Some(Token::Semicolon)
                                | Some(Token::Equals)
                                | Some(Token::Neq)
                                | Some(Token::Less)
                                | Some(Token::Greater)
                                | Some(Token::Leq)
                                | Some(Token::Geq)
                                | Some(Token::LeftBrace)
                                | Some(Token::LogicalAnd)
                                | Some(Token::LogicalOr) => {
                                    vec![]
                                }
                                Some(Token::Sub) => {
                                    vec![
                                        GrammarSymbol::Terminal(Token::Sub),
                                        GrammarSymbol::Term,
                                        GrammarSymbol::Expression1,
                                    ]
                                }
                                Some(Token::Add) => {
                                    vec![
                                        GrammarSymbol::Terminal(Token::Add),
                                        GrammarSymbol::Term,
                                        GrammarSymbol::Expression1,
                                    ]
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::Factor => match token.clone() {
                                Some(Token::LeftParen) => {
                                    vec![
                                        GrammarSymbol::Terminal(Token::LeftParen),
                                        GrammarSymbol::Expression,
                                        GrammarSymbol::Terminal(Token::RightParen),
                                    ]
                                }
                                Some(Token::ID(_)) => {
                                    vec![GrammarSymbol::ID, GrammarSymbol::IDOrFn]
                                }
                                Some(Token::Integer(num)) => {
                                    vec![GrammarSymbol::Terminal(Token::Integer(num))]
                                }
                                Some(Token::Float(num)) => {
                                    vec![GrammarSymbol::Terminal(Token::Float(num))]
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::Func => match token.clone() {
                                Some(Token::Fn) => {
                                    vec![
                                        GrammarSymbol::Terminal(Token::Fn),
                                        GrammarSymbol::ID,
                                        GrammarSymbol::Terminal(Token::LeftParen),
                                        GrammarSymbol::ParamList,
                                        GrammarSymbol::Terminal(Token::RightParen),
                                        GrammarSymbol::Terminal(Token::Arrow),
                                        GrammarSymbol::ReturnType,
                                        GrammarSymbol::Block,
                                    ]
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::ID => match token.clone() {
                                Some(Token::ID(id)) => {
                                    vec![GrammarSymbol::Terminal(Token::ID(id.clone()))]
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::IDOrFn => match token.clone() {
                                Some(Token::LeftParen) => {
                                    vec![
                                        GrammarSymbol::Terminal(Token::LeftParen),
                                        GrammarSymbol::InputList,
                                        GrammarSymbol::Terminal(Token::RightParen),
                                    ]
                                }
                                Some(Token::Mul)
                                | Some(Token::Div)
                                | Some(Token::Add)
                                | Some(Token::Sub)
                                | Some(Token::RightParen)
                                | Some(Token::Semicolon)
                                | Some(Token::Equals)
                                | Some(Token::Neq)
                                | Some(Token::Less)
                                | Some(Token::Greater)
                                | Some(Token::Leq)
                                | Some(Token::Geq)
                                | Some(Token::LeftBrace)
                                | Some(Token::LogicalAnd)
                                | Some(Token::LogicalOr) => {
                                    // println!("T' -> `");
                                    vec![]
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::InputList => match token.clone() {
                                Some(Token::ID(_))
                                | Some(Token::Integer(_))
                                | Some(Token::Float(_))
                                | Some(Token::LeftParen) => {
                                    vec![GrammarSymbol::Expression, GrammarSymbol::InputRest]
                                }
                                Some(Token::RightParen) => {
                                    vec![]
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::InputRest => match token.clone() {
                                Some(Token::Comma) => {
                                    vec![
                                        GrammarSymbol::Terminal(Token::Comma),
                                        GrammarSymbol::InputList,
                                    ]
                                }
                                Some(Token::RightParen) => {
                                    // println!("input_rest -> `")
                                    vec![]
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::NodeBlock => match token.clone() {
                                Some(Token::LeftBrace) => vec![
                                    GrammarSymbol::Terminal(Token::LeftBrace),
                                    GrammarSymbol::TLStmtList,
                                    GrammarSymbol::Terminal(Token::RightBrace),
                                ],
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::NodeHeader => match token.clone() {
                                Some(Token::ID(_)) => {
                                    vec![GrammarSymbol::ID, GrammarSymbol::OptIDList]
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::NodeList => match token.clone() {
                                Some(Token::ID(_)) => {
                                    vec![GrammarSymbol::ID, GrammarSymbol::NodeRest]
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::NodeNT => match token.clone() {
                                Some(Token::Node) => {
                                    vec![
                                        GrammarSymbol::Terminal(Token::Node),
                                        GrammarSymbol::NodeHeader,
                                        GrammarSymbol::NodeBlock,
                                    ]
                                    // println!("N -> node Nh NB");
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::NodeRest => match token.clone() {
                                Some(Token::Comma) => vec![
                                    GrammarSymbol::Terminal(Token::Comma),
                                    GrammarSymbol::NodeList,
                                ],
                                Some(Token::LeftBrace) => vec![],
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::OptElse => match token.clone() {
                                Some(Token::Else) => {
                                    vec![GrammarSymbol::Terminal(Token::Else), GrammarSymbol::Block]
                                }
                                Some(Token::Var)
                                | Some(Token::Const)
                                | Some(Token::ID(_))
                                | Some(Token::While)
                                | Some(Token::If)
                                | Some(Token::RightBrace) => {
                                    vec![]
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::OptIDList => match token.clone() {
                                Some(Token::Colon) => vec![
                                    GrammarSymbol::Terminal(Token::Colon),
                                    GrammarSymbol::NodeList,
                                ],
                                Some(Token::LeftBrace) => vec![],
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::Param => match token.clone() {
                                Some(Token::ID(_)) => vec![
                                    GrammarSymbol::ID,
                                    GrammarSymbol::Terminal(Token::Colon),
                                    GrammarSymbol::ID,
                                ],
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::ParamList => match token.clone() {
                                Some(Token::ID(_)) => {
                                    vec![GrammarSymbol::Param, GrammarSymbol::ParamRest]
                                }
                                Some(Token::RightParen) => vec![],
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::ParamRest => match token.clone() {
                                Some(Token::Comma) => vec![
                                    GrammarSymbol::Terminal(Token::Comma),
                                    GrammarSymbol::ParamList,
                                ],
                                Some(Token::RightParen) => vec![],
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::Program => match token.clone() {
                                Some(Token::Node) => {
                                    vec![GrammarSymbol::NodeNT, GrammarSymbol::Program]
                                    // println!("P -> N P");
                                }
                                None => {
                                    vec![]
                                    // println!("P -> `");
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::ReturnType => match token.clone() {
                                Some(Token::ID(_)) => vec![GrammarSymbol::ID],
                                Some(Token::LeftParen) => vec![
                                    GrammarSymbol::Terminal(Token::LeftParen),
                                    GrammarSymbol::Terminal(Token::RightParen),
                                ],
                                Some(Token::Not) => vec![GrammarSymbol::Terminal(Token::Not)],
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::Stmt => match token.clone() {
                                Some(Token::Var) => {
                                    vec![
                                        GrammarSymbol::Terminal(Token::Var),
                                        GrammarSymbol::ID,
                                        GrammarSymbol::Terminal(Token::Colon),
                                        GrammarSymbol::ID,
                                        GrammarSymbol::Terminal(Token::Assign),
                                        GrammarSymbol::Expression,
                                        GrammarSymbol::Terminal(Token::Semicolon),
                                    ]
                                }
                                Some(Token::Const) => {
                                    vec![
                                        GrammarSymbol::Terminal(Token::Const),
                                        GrammarSymbol::ID,
                                        GrammarSymbol::Terminal(Token::Colon),
                                        GrammarSymbol::ID,
                                        GrammarSymbol::Terminal(Token::Assign),
                                        GrammarSymbol::Expression,
                                        GrammarSymbol::Terminal(Token::Semicolon),
                                    ]
                                }
                                Some(Token::ID(_)) => {
                                    vec![GrammarSymbol::ID, GrammarSymbol::AssignOrFnCall]
                                }
                                Some(Token::While) => {
                                    vec![
                                        GrammarSymbol::Terminal(Token::While),
                                        GrammarSymbol::Conditional,
                                        GrammarSymbol::Block,
                                    ]
                                }
                                Some(Token::If) => {
                                    vec![
                                        GrammarSymbol::Terminal(Token::If),
                                        GrammarSymbol::Conditional,
                                        GrammarSymbol::Block,
                                        GrammarSymbol::OptElse,
                                    ]
                                }
                                Some(Token::Return) => {
                                    vec![
                                        GrammarSymbol::Terminal(Token::Return),
                                        GrammarSymbol::Expression,
                                        GrammarSymbol::Terminal(Token::Semicolon),
                                    ]
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::StmtList => match token.clone() {
                                Some(Token::Var) | Some(Token::Const) | Some(Token::While)
                                | Some(Token::If) | Some(Token::Return) | Some(Token::ID(_)) => {
                                    vec![GrammarSymbol::Stmt, GrammarSymbol::StmtList]
                                }
                                Some(Token::RightBrace) => {
                                    vec![]
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::Term => match token.clone() {
                                Some(Token::ID(_))
                                | Some(Token::Integer(_))
                                | Some(Token::Float(_))
                                | Some(Token::LeftParen) => {
                                    vec![GrammarSymbol::Factor, GrammarSymbol::Term1]
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::Term1 => match token.clone() {
                                Some(Token::Mul) => {
                                    vec![
                                        GrammarSymbol::Terminal(Token::Mul),
                                        GrammarSymbol::Factor,
                                        GrammarSymbol::Term1,
                                    ]
                                }
                                Some(Token::Div) => {
                                    vec![
                                        GrammarSymbol::Terminal(Token::Div),
                                        GrammarSymbol::Factor,
                                        GrammarSymbol::Term1,
                                    ]
                                }
                                Some(Token::Add)
                                | Some(Token::Sub)
                                | Some(Token::RightParen)
                                | Some(Token::Semicolon)
                                | Some(Token::Equals)
                                | Some(Token::Neq)
                                | Some(Token::Less)
                                | Some(Token::Greater)
                                | Some(Token::Leq)
                                | Some(Token::Geq)
                                | Some(Token::LeftBrace)
                                | Some(Token::LogicalAnd)
                                | Some(Token::LogicalOr) => {
                                    vec![]
                                }
                                _ => {
                                    println!("{:?}", token.unwrap().clone());
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::TLStmt => match token.clone() {
                                Some(Token::Fn) => {
                                    vec![GrammarSymbol::Func]
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                            GrammarSymbol::TLStmtList => match token.clone() {
                                Some(Token::Fn) => {
                                    vec![GrammarSymbol::TLStmt, GrammarSymbol::TLStmtList]
                                }
                                Some(Token::RightBrace) => {
                                    vec![]
                                }
                                _ => {
                                    return Err("syntax error".to_string());
                                }
                            },
                        };

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
                                GrammarSymbol::Terminal(_)
                                | GrammarSymbol::Empty
                                | GrammarSymbol::End => {}
                                _ => {
                                    if !has_nt {
                                        has_nt = true;
                                        next_idx = i;
                                    }
                                }
                            }
                        });

                        if has_nt {
                            idx = next_idx as usize;
                        } else {
                            idx = self.parse_tree.get_next_nt_sibling(idx);
                        }
                    }
                }
            }

            if stack.is_empty() && token == None {
                return Ok(());
            }
        }
    }

    pub fn generate_ast(&mut self) {
        self.ast = self.build_ast_from_parse_node(0);
    }

    pub fn build_ast_from_parse_node(&self, idx: usize) -> AbstractSyntaxTree {
        let mut tree = AbstractSyntaxTree::new();
        let node = self.parse_tree.get_node(idx);
        let children = self.parse_tree.get_children(idx);

        match node {
            GrammarSymbol::ID => {
                tree = self.build_ast_from_parse_node(children[0]);
            }
            GrammarSymbol::Terminal(Token::ID(id)) => {
                tree.node = SyntaxTreeNode::Identifier(id);
            }
            GrammarSymbol::NodeHeader => {
                tree.node = SyntaxTreeNode::NodeHeader;

                tree.children = vec![
                    self.build_ast_from_parse_node(children[0]),
                    self.build_ast_from_parse_node(children[1]),
                ];
            }
            GrammarSymbol::OptIDList => {
                if children.len() == 1 {
                    tree.node = SyntaxTreeNode::Null;
                } else {
                    tree = self.build_ast_from_parse_node(children[1]);
                }
            }
            GrammarSymbol::NodeList => {
                tree.node = SyntaxTreeNode::NodeList;

                tree.children = vec![
                    self.build_ast_from_parse_node(children[0]),
                    self.build_ast_from_parse_node(children[1]),
                ];
            }
            GrammarSymbol::NodeRest => match self.parse_tree.get_node(children[0]) {
                GrammarSymbol::Terminal(Token::Comma) => {
                    tree = self.build_ast_from_parse_node(children[1]);
                }
                GrammarSymbol::Empty => {}
                _ => {}
            },
            GrammarSymbol::NodeNT => {
                tree.node = SyntaxTreeNode::DeclareNode;

                tree.children = vec![
                    self.build_ast_from_parse_node(children[1]),
                    self.build_ast_from_parse_node(children[2]),
                ];
            }
            GrammarSymbol::Program => match self.parse_tree.get_node(children[0]) {
                GrammarSymbol::NodeNT => {
                    tree.node = SyntaxTreeNode::NodeSeq;

                    tree.children = vec![
                        self.build_ast_from_parse_node(children[0]),
                        self.build_ast_from_parse_node(children[1]),
                    ];
                }
                GrammarSymbol::Empty => {}
                _ => {}
            },
            GrammarSymbol::NodeBlock => {
                tree = self.build_ast_from_parse_node(children[1]);
            }
            GrammarSymbol::TLStmtList => match self.parse_tree.get_node(children[0]) {
                GrammarSymbol::TLStmt => {
                    tree.node = SyntaxTreeNode::TLStmtSeq;

                    tree.children = vec![
                        self.build_ast_from_parse_node(children[0]),
                        self.build_ast_from_parse_node(children[1]),
                    ];
                }
                GrammarSymbol::Empty => {}
                _ => {}
            },
            GrammarSymbol::TLStmt => match self.parse_tree.get_node(children[0]) {
                GrammarSymbol::Func => {
                    tree = self.build_ast_from_parse_node(children[0]);
                }
                _ => {}
            },
            GrammarSymbol::Func => {
                tree.node = SyntaxTreeNode::DeclareFunc;

                tree.children = vec![
                    self.build_ast_from_parse_node(children[1]),
                    self.build_ast_from_parse_node(children[3]),
                    self.build_ast_from_parse_node(children[6]),
                    self.build_ast_from_parse_node(children[7]),
                ];
            }
            GrammarSymbol::ParamList => match self.parse_tree.get_node(children[0]) {
                GrammarSymbol::Param => {
                    tree.node = SyntaxTreeNode::ParamList;

                    tree.children = vec![
                        self.build_ast_from_parse_node(children[0]),
                        self.build_ast_from_parse_node(children[1]),
                    ];
                }
                GrammarSymbol::Empty => {}
                _ => {}
            },
            GrammarSymbol::Param => {
                tree.node = SyntaxTreeNode::Param;

                tree.children = vec![
                    self.build_ast_from_parse_node(children[0]),
                    self.build_ast_from_parse_node(children[2]),
                ];
            }
            GrammarSymbol::ParamRest => match self.parse_tree.get_node(children[0]) {
                GrammarSymbol::Terminal(Token::Comma) => {
                    tree = self.build_ast_from_parse_node(children[1]);
                }
                GrammarSymbol::Empty => {}
                _ => {}
            },
            GrammarSymbol::ReturnType => match self.parse_tree.get_node(children[0]) {
                GrammarSymbol::ID => {
                    tree.node = SyntaxTreeNode::ReturnType;

                    tree.children = vec![self.build_ast_from_parse_node(children[0])];
                }
                GrammarSymbol::Terminal(Token::LeftParen) => {
                    tree.node = SyntaxTreeNode::ReturnType;

                    let mut child = AbstractSyntaxTree::new();
                    child.node = SyntaxTreeNode::Void;

                    tree.children = vec![child];
                }
                GrammarSymbol::Terminal(Token::Not) => {
                    tree.node = SyntaxTreeNode::ReturnType;

                    let mut child = AbstractSyntaxTree::new();
                    child.node = SyntaxTreeNode::NoReturn;

                    tree.children = vec![child];
                }
                _ => {
                    todo!("implement return types");
                }
            },
            GrammarSymbol::Block => {
                tree = self.build_ast_from_parse_node(children[1]);
            }
            GrammarSymbol::StmtList => match self.parse_tree.get_node(children[0]) {
                GrammarSymbol::Empty => {}
                _ => {
                    tree.node = SyntaxTreeNode::StmtSeq;

                    tree.children = vec![
                        self.build_ast_from_parse_node(children[0]),
                        self.build_ast_from_parse_node(children[1]),
                    ];
                }
            },
            GrammarSymbol::Stmt => match self.parse_tree.get_node(children[0]) {
                GrammarSymbol::Terminal(Token::If) => {
                    tree.node = SyntaxTreeNode::IfStmt;

                    tree.children = vec![
                        self.build_ast_from_parse_node(children[1]),
                        self.build_ast_from_parse_node(children[2]),
                        self.build_ast_from_parse_node(children[3]),
                    ];
                }
                GrammarSymbol::Terminal(Token::Const) => {
                    tree.node = SyntaxTreeNode::DeclareConst;

                    tree.children = vec![
                        self.build_ast_from_parse_node(children[1]),
                        self.build_ast_from_parse_node(children[3]),
                        self.build_ast_from_parse_node(children[5]),
                    ];
                }
                GrammarSymbol::Terminal(Token::Var) => {
                    tree.node = SyntaxTreeNode::DeclareVar;

                    tree.children = vec![
                        self.build_ast_from_parse_node(children[1]),
                        self.build_ast_from_parse_node(children[3]),
                        self.build_ast_from_parse_node(children[5]),
                    ];
                }
                GrammarSymbol::Terminal(Token::While) => {
                    tree.node = SyntaxTreeNode::WhileLoop;

                    tree.children = vec![
                        self.build_ast_from_parse_node(children[1]),
                        self.build_ast_from_parse_node(children[2]),
                    ];
                }
                GrammarSymbol::ID => {
                    tree = self.build_ast_from_parse_node(children[1]);

                    tree.children
                        .insert(0, self.build_ast_from_parse_node(children[0]));
                }
                GrammarSymbol::Terminal(Token::Return) => {
                    tree.node = SyntaxTreeNode::ReturnValue;

                    tree.children = vec![self.build_ast_from_parse_node(children[1])];
                }
                _ => {}
            },
            GrammarSymbol::OptElse => match self.parse_tree.get_node(children[0]) {
                GrammarSymbol::Terminal(Token::Else) => {
                    tree = self.build_ast_from_parse_node(children[1]);
                }
                GrammarSymbol::Empty => {
                    tree.node = SyntaxTreeNode::Null;
                }
                _ => {}
            },
            GrammarSymbol::AssignOrFnCall => match self.parse_tree.get_node(children[0]) {
                GrammarSymbol::Terminal(Token::Assign) => {
                    tree.node = SyntaxTreeNode::Assign;

                    tree.children = vec![self.build_ast_from_parse_node(children[1])];
                }
                GrammarSymbol::Terminal(Token::LeftParen) => {
                    tree.node = SyntaxTreeNode::FnCall;

                    tree.children = vec![self.build_ast_from_parse_node(children[1])];
                }
                _ => {}
            },
            GrammarSymbol::Expression => {
                let subtree = self.build_ast_from_parse_node(children[1]);

                match subtree.node {
                    SyntaxTreeNode::AddOp | SyntaxTreeNode::SubOp => {
                        tree = subtree;
                        tree.children
                            .insert(0, self.build_ast_from_parse_node(children[0]));

                        tree = Self::rebalance_expression_tree(tree);
                    }
                    _ => {
                        tree = self.build_ast_from_parse_node(children[0]);
                    }
                }
            }
            GrammarSymbol::Expression1 => match self.parse_tree.get_node(children[0]) {
                GrammarSymbol::Terminal(Token::Add) => {
                    let mut subtree = self.build_ast_from_parse_node(children[2]);

                    match subtree.node {
                        SyntaxTreeNode::AddOp | SyntaxTreeNode::SubOp => {
                            tree.node = SyntaxTreeNode::AddOp;

                            subtree
                                .children
                                .insert(0, self.build_ast_from_parse_node(children[1]));

                            tree.children = vec![subtree];
                        }
                        _ => {
                            tree.node = SyntaxTreeNode::AddOp;

                            tree.children = vec![self.build_ast_from_parse_node(children[1])];
                        }
                    }
                }
                GrammarSymbol::Terminal(Token::Sub) => {
                    let mut subtree = self.build_ast_from_parse_node(children[2]);

                    match subtree.node {
                        SyntaxTreeNode::AddOp | SyntaxTreeNode::SubOp => {
                            tree.node = SyntaxTreeNode::SubOp;

                            subtree
                                .children
                                .insert(0, self.build_ast_from_parse_node(children[1]));

                            tree.children = vec![subtree];
                        }
                        _ => {
                            tree.node = SyntaxTreeNode::SubOp;

                            tree.children = vec![self.build_ast_from_parse_node(children[1])];
                        }
                    }
                }
                _ => {
                    tree.node = SyntaxTreeNode::Null;
                }
            },
            GrammarSymbol::Term => {
                let subtree = self.build_ast_from_parse_node(children[1]);

                match subtree.node {
                    SyntaxTreeNode::MulOp | SyntaxTreeNode::DivOp => {
                        tree = subtree;
                        tree.children
                            .insert(0, self.build_ast_from_parse_node(children[0]));

                        tree = Self::rebalance_term_tree(tree);
                    }
                    _ => {
                        tree = self.build_ast_from_parse_node(children[0]);
                    }
                }
            }
            GrammarSymbol::Term1 => match self.parse_tree.get_node(children[0]) {
                GrammarSymbol::Terminal(Token::Mul) => {
                    let mut subtree = self.build_ast_from_parse_node(children[2]);

                    match subtree.node {
                        SyntaxTreeNode::MulOp | SyntaxTreeNode::DivOp => {
                            tree.node = SyntaxTreeNode::MulOp;

                            subtree
                                .children
                                .insert(0, self.build_ast_from_parse_node(children[1]));

                            tree.children = vec![subtree];
                        }
                        _ => {
                            tree.node = SyntaxTreeNode::MulOp;

                            tree.children = vec![self.build_ast_from_parse_node(children[1])];
                        }
                    }
                }
                GrammarSymbol::Terminal(Token::Div) => {
                    let mut subtree = self.build_ast_from_parse_node(children[2]);

                    match subtree.node {
                        SyntaxTreeNode::MulOp | SyntaxTreeNode::DivOp => {
                            tree.node = SyntaxTreeNode::DivOp;

                            subtree
                                .children
                                .insert(0, self.build_ast_from_parse_node(children[1]));

                            tree.children = vec![subtree];
                        }
                        _ => {
                            tree.node = SyntaxTreeNode::DivOp;

                            tree.children = vec![self.build_ast_from_parse_node(children[1])];
                        }
                    }
                }
                _ => {
                    tree.node = SyntaxTreeNode::Null;
                }
            },
            GrammarSymbol::Factor => match self.parse_tree.get_node(children[0]) {
                GrammarSymbol::Terminal(Token::Integer(num)) => {
                    tree.node = SyntaxTreeNode::Integer(num);
                }
                GrammarSymbol::Terminal(Token::Float(num)) => {
                    tree.node = SyntaxTreeNode::Float(num);
                }
                GrammarSymbol::ID => {
                    let subtree = self.build_ast_from_parse_node(children[1]);

                    match subtree.node {
                        SyntaxTreeNode::Null => {
                            tree = self.build_ast_from_parse_node(children[0]);
                        }
                        SyntaxTreeNode::InputList => {
                            tree.node = SyntaxTreeNode::FnCall;

                            tree.children =
                                vec![self.build_ast_from_parse_node(children[0]), subtree];
                        }
                        _ => {}
                    }
                }
                GrammarSymbol::Terminal(Token::LeftParen) => {
                    tree = self.build_ast_from_parse_node(children[1]);
                }
                _ => {}
            },
            GrammarSymbol::IDOrFn => {
                if children.len() == 1 {
                    tree.node = SyntaxTreeNode::Null;
                } else {
                    tree = self.build_ast_from_parse_node(children[1]);
                }
            }
            GrammarSymbol::Conditional => {
                let subtree = self.build_ast_from_parse_node(children[1]);

                match subtree.node {
                    SyntaxTreeNode::OrOp => {
                        tree = subtree;
                        tree.children
                            .insert(0, self.build_ast_from_parse_node(children[0]));

                        tree = Self::rebalance_comparison_tree(tree);
                    }
                    _ => {
                        tree = self.build_ast_from_parse_node(children[0]);
                    }
                }
            }
            GrammarSymbol::Conditional1 => match self.parse_tree.get_node(children[0]) {
                GrammarSymbol::Terminal(Token::LogicalOr) => {
                    let mut subtree = self.build_ast_from_parse_node(children[2]);

                    match subtree.node {
                        SyntaxTreeNode::OrOp => {
                            tree.node = SyntaxTreeNode::OrOp;

                            subtree
                                .children
                                .insert(0, self.build_ast_from_parse_node(children[1]));

                            tree.children = vec![subtree];
                        }
                        _ => {
                            tree.node = SyntaxTreeNode::OrOp;

                            tree.children = vec![self.build_ast_from_parse_node(children[1])];
                        }
                    }
                }
                _ => {
                    tree.node = SyntaxTreeNode::Null;
                }
            },
            GrammarSymbol::BoolTerm => {
                let subtree = self.build_ast_from_parse_node(children[1]);

                match subtree.node {
                    SyntaxTreeNode::AndOp => {
                        tree = subtree;
                        tree.children
                            .insert(0, self.build_ast_from_parse_node(children[0]));

                        tree = Self::rebalance_bool_term_tree(tree);
                    }
                    _ => {
                        tree = self.build_ast_from_parse_node(children[0]);
                    }
                }
            }
            GrammarSymbol::BoolTerm1 => match self.parse_tree.get_node(children[0]) {
                GrammarSymbol::Terminal(Token::LogicalAnd) => {
                    let mut subtree = self.build_ast_from_parse_node(children[2]);

                    match subtree.node {
                        SyntaxTreeNode::AndOp => {
                            tree.node = SyntaxTreeNode::AndOp;

                            subtree
                                .children
                                .insert(0, self.build_ast_from_parse_node(children[1]));

                            tree.children = vec![subtree];
                        }
                        _ => {
                            tree.node = SyntaxTreeNode::AndOp;

                            tree.children = vec![self.build_ast_from_parse_node(children[1])];
                        }
                    }
                }
                _ => {
                    tree.node = SyntaxTreeNode::Null;
                }
            },
            GrammarSymbol::BoolExpr => {
                let comp = self.build_ast_from_parse_node(children[1]);

                tree.node = comp.node;

                tree.children = vec![
                    self.build_ast_from_parse_node(children[0]),
                    self.build_ast_from_parse_node(children[2]),
                ];
            }
            GrammarSymbol::Comparison => {
                tree = self.build_ast_from_parse_node(children[0]);
            }
            GrammarSymbol::Terminal(Token::Equals) => {
                tree.node = SyntaxTreeNode::CompEq;
            }
            GrammarSymbol::Terminal(Token::Neq) => {
                tree.node = SyntaxTreeNode::CompNeq;
            }
            GrammarSymbol::Terminal(Token::Less) => {
                tree.node = SyntaxTreeNode::CompLess;
            }
            GrammarSymbol::Terminal(Token::Greater) => {
                tree.node = SyntaxTreeNode::CompGreater;
            }
            GrammarSymbol::Terminal(Token::Leq) => {
                tree.node = SyntaxTreeNode::CompLeq;
            }
            GrammarSymbol::Terminal(Token::Geq) => {
                tree.node = SyntaxTreeNode::CompGeq;
            }
            GrammarSymbol::InputList => match self.parse_tree.get_node(children[0]) {
                GrammarSymbol::Empty => {
                    tree.node = SyntaxTreeNode::Null;
                }
                GrammarSymbol::Expression => {
                    tree.node = SyntaxTreeNode::InputList;

                    tree.children = vec![
                        self.build_ast_from_parse_node(children[0]),
                        self.build_ast_from_parse_node(children[1]),
                    ];
                }
                _ => {}
            },
            GrammarSymbol::InputRest => match self.parse_tree.get_node(children[0]) {
                GrammarSymbol::Empty => {
                    tree.node = SyntaxTreeNode::Null;
                }
                GrammarSymbol::Terminal(Token::Comma) => {
                    tree = self.build_ast_from_parse_node(children[1]);
                }
                _ => {}
            },
            e => {
                println!("unresolved {:?}", e);
            }
        };

        tree
    }

    fn rebalance_expression_tree(mut tree: AbstractSyntaxTree) -> AbstractSyntaxTree {
        let mut stack = LinkedList::new();

        let mut root = tree.clone();

        while root.children.len() > 1 {
            if root.node != SyntaxTreeNode::AddOp && root.node != SyntaxTreeNode::SubOp {
                break;
            }

            let mut subtree = root.clone();
            subtree.children.remove(1);
            stack.push_back(subtree);

            root = root.children[1].clone();
        }

        stack.push_back(root.clone());

        tree = stack.pop_front().unwrap();
        while !stack.is_empty() {
            let mut front = stack.pop_front().unwrap();

            if front.children.len() > 0
                && (front.node == SyntaxTreeNode::AddOp || front.node == SyntaxTreeNode::SubOp)
            {
                tree.children.push(front.children[0].clone());

                front.children = vec![tree];

                tree = front;
            } else {
                tree.children.push(front);
            }
        }

        tree
    }

    fn rebalance_term_tree(mut tree: AbstractSyntaxTree) -> AbstractSyntaxTree {
        let mut stack = LinkedList::new();

        let mut root = tree.clone();

        while root.children.len() > 1 {
            if root.node != SyntaxTreeNode::MulOp && root.node != SyntaxTreeNode::DivOp {
                break;
            }

            let mut subtree = root.clone();
            subtree.children.remove(1);
            stack.push_back(subtree);

            root = root.children[1].clone();
        }

        stack.push_back(root.clone());

        tree = stack.pop_front().unwrap();
        while !stack.is_empty() {
            let mut front = stack.pop_front().unwrap();

            if front.children.len() > 0
                && (front.node == SyntaxTreeNode::MulOp || front.node == SyntaxTreeNode::DivOp)
            {
                tree.children.push(front.children[0].clone());

                front.children = vec![tree];

                tree = front;
            } else {
                tree.children.push(front);
            }
        }

        tree
    }

    fn rebalance_comparison_tree(mut tree: AbstractSyntaxTree) -> AbstractSyntaxTree {
        let mut stack = LinkedList::new();

        let mut root = tree.clone();

        while root.children.len() > 1 {
            if root.node != SyntaxTreeNode::OrOp {
                break;
            }

            let mut subtree = root.clone();
            subtree.children.remove(1);
            stack.push_back(subtree);

            root = root.children[1].clone();
        }

        stack.push_back(root.clone());

        tree = stack.pop_front().unwrap();
        while !stack.is_empty() {
            let mut front = stack.pop_front().unwrap();

            if front.children.len() > 0 && front.node == SyntaxTreeNode::OrOp {
                tree.children.push(front.children[0].clone());

                front.children = vec![tree];

                tree = front;
            } else {
                tree.children.push(front);
            }
        }

        tree
    }

    fn rebalance_bool_term_tree(mut tree: AbstractSyntaxTree) -> AbstractSyntaxTree {
        let mut stack = LinkedList::new();

        let mut root = tree.clone();

        while root.children.len() > 1 {
            if root.node != SyntaxTreeNode::AndOp {
                break;
            }

            let mut subtree = root.clone();
            subtree.children.remove(1);
            stack.push_back(subtree);

            root = root.children[1].clone();
        }

        stack.push_back(root.clone());

        tree = stack.pop_front().unwrap();
        while !stack.is_empty() {
            let mut front = stack.pop_front().unwrap();

            if front.children.len() > 0 && front.node == SyntaxTreeNode::AndOp {
                tree.children.push(front.children[0].clone());

                front.children = vec![tree];

                tree = front;
            } else {
                tree.children.push(front);
            }
        }

        tree
    }
}
