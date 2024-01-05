use std::collections::{HashMap, LinkedList};

use crate::parser::{AbstractSyntaxTree, Parser, SyntaxTreeNode};

#[derive(Clone, Debug, PartialEq)]
enum ScopeElem {
    NodeScope(String),
    FuncScope(String),
    IfScope,
    WhileScope,
    ElseScope,
    Variable(String),
    Const(String),
    Func(String),
}

pub struct Source {
    graph: HashMap<String, Vec<String>>,
}

impl Source {
    pub fn new(parser: Parser) -> Self {
        let mut graph = HashMap::new();
        Self::create_node_graph(&mut graph, parser.ast.clone());

        let mut stack = LinkedList::new();

        match Self::check_semantics(&mut stack, parser.ast.clone()) {
            Ok(_) => println!("semantic check passed"),
            Err(_) => println!("SEMANTIC CHECK FAILED"),
        }

        Self { graph }
    }

    fn create_node_graph(graph: &mut HashMap<String, Vec<String>>, ast: AbstractSyntaxTree) {
        match ast.node {
            SyntaxTreeNode::NodeSeq => {
                Self::create_node_graph(graph, ast.children[0].clone());
                Self::create_node_graph(graph, ast.children[1].clone());
            }
            SyntaxTreeNode::DeclareNode => {
                Self::create_node_graph(graph, ast.children[0].clone());
            }
            SyntaxTreeNode::NodeHeader => {
                let children = ast.children.clone();

                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => {
                        graph.insert(id.clone(), vec![]);
                        id
                    }
                    _ => "".to_string(),
                };

                Self::add_dependencies(graph, &id, children[1].clone());
            }
            _ => {
                return;
            }
        }
    }

    fn add_dependencies(
        graph: &mut HashMap<String, Vec<String>>,
        id: &String,
        ast: AbstractSyntaxTree,
    ) {
        match ast.node {
            SyntaxTreeNode::Identifier(dependency) => {
                let mut dependencies = graph.get(id).unwrap().clone();
                dependencies.push(dependency);
                graph.insert(id.clone(), dependencies);
            }
            SyntaxTreeNode::Null => {
                return;
            }
            _ => {
                Self::add_dependencies(graph, id, ast.children[0].clone());
                Self::add_dependencies(graph, id, ast.children[1].clone());
            }
        }
    }

    fn check_semantics(
        stack: &mut LinkedList<ScopeElem>,
        ast: AbstractSyntaxTree,
    ) -> Result<(), ()> {
        let children = ast.children.clone();
        match ast.node {
            SyntaxTreeNode::NodeSeq => {
                let decl_node = children[0].clone();
                let node_header = decl_node.children[0].clone();
                let node_id = match node_header.children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                stack.push_back(ScopeElem::NodeScope(node_id.clone()));
                println!("{:?}", stack);

                Self::check_semantics(stack, decl_node.children[1].clone())?;

                while !stack.is_empty() {
                    let top = stack.pop_back().unwrap();

                    if top == ScopeElem::NodeScope(node_id.clone()) {
                        break;
                    }
                }
                println!("{:?}", stack);
            }
            SyntaxTreeNode::DeclareFunc => {
                let func_id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                stack.push_back(ScopeElem::FuncScope(func_id.clone()));
                println!("{:?}", stack);

                Self::check_semantics(stack, children[1].clone())?;
                println!("{:?}", stack);

                Self::check_semantics(stack, children[3].clone())?;

                while !stack.is_empty() {
                    let top = stack.pop_back().unwrap();

                    if top == ScopeElem::FuncScope(func_id.clone()) {
                        stack.push_back(ScopeElem::Func(func_id));
                        break;
                    }
                }
                println!("{:?}", stack);
            }
            SyntaxTreeNode::WhileLoop => {
                Self::check_semantics(stack, children[0].clone())?;

                stack.push_back(ScopeElem::WhileScope);
                println!("{:?}", stack);

                Self::check_semantics(stack, children[1].clone())?;

                while !stack.is_empty() {
                    let top = stack.pop_back().unwrap();

                    if top == ScopeElem::WhileScope {
                        break;
                    }
                }
                println!("{:?}", stack);
            }
            SyntaxTreeNode::IfStmt => {
                Self::check_semantics(stack, children[0].clone())?;

                stack.push_back(ScopeElem::IfScope);
                println!("{:?}", stack);

                Self::check_semantics(stack, children[1].clone())?;

                while !stack.is_empty() {
                    let top = stack.pop_back().unwrap();

                    if top == ScopeElem::IfScope {
                        break;
                    }
                }
                println!("{:?}", stack);

                if children[2].clone().node != SyntaxTreeNode::Null {
                    stack.push_back(ScopeElem::ElseScope);
                    println!("{:?}", stack);

                    Self::check_semantics(stack, children[2].clone())?;

                    while !stack.is_empty() {
                        let top = stack.pop_back().unwrap();

                        if top == ScopeElem::ElseScope {
                            break;
                        }
                    }
                    println!("{:?}", stack);
                }
            }
            SyntaxTreeNode::ParamList => {
                let param = children[0].clone();
                let id = match param.children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                stack.push_back(ScopeElem::Variable(id));

                Self::check_semantics(stack, children[1].clone())?;
            }
            SyntaxTreeNode::DeclareConst => {
                Self::check_semantics(stack, children[2].clone())?;

                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                stack.push_back(ScopeElem::Const(id));
                println!("{:?}", stack);
            }
            SyntaxTreeNode::DeclareVar => {
                Self::check_semantics(stack, children[2].clone())?;

                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                stack.push_back(ScopeElem::Variable(id));
                println!("{:?}", stack);
            }
            SyntaxTreeNode::Assign => {
                Self::check_semantics(stack, children[1].clone())?;

                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                for elem in stack {
                    if elem.clone() == ScopeElem::Variable(id.clone()) {
                        return Ok(());
                    }
                }

                println!("{:?}", id);
                return Err(());
            }
            SyntaxTreeNode::AddOp
            | SyntaxTreeNode::SubOp
            | SyntaxTreeNode::MulOp
            | SyntaxTreeNode::DivOp
            | SyntaxTreeNode::AndOp
            | SyntaxTreeNode::OrOp
            | SyntaxTreeNode::CompEq
            | SyntaxTreeNode::CompNeq
            | SyntaxTreeNode::CompLeq
            | SyntaxTreeNode::CompGeq
            | SyntaxTreeNode::CompLess
            | SyntaxTreeNode::CompGreater => {
                Self::check_semantics(stack, children[0].clone())?;
                Self::check_semantics(stack, children[1].clone())?;
            }
            SyntaxTreeNode::Identifier(id) => {
                for elem in stack {
                    if elem.clone() == ScopeElem::Variable(id.clone())
                        || elem.clone() == ScopeElem::Const(id.clone())
                    {
                        return Ok(());
                    }
                }
                println!("{:?}", id);
                return Err(());
            }
            SyntaxTreeNode::FnCall => {
                Self::check_semantics(stack, children[1].clone())?;

                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                for elem in stack {
                    if elem.clone() == ScopeElem::Func(id.clone()) {
                        return Ok(());
                    }
                }

                println!("{:?}", id);
                return Err(());
            }
            _ => {
                for child in children {
                    Self::check_semantics(stack, child)?;
                }
            }
        }

        Ok(())
    }

    pub fn generate_assembly(&self) {}
}
