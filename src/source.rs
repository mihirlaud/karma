use std::collections::{HashMap, LinkedList};

use crate::parser::{AbstractSyntaxTree, Parser, SyntaxTreeNode};

pub struct Source {
    graph: HashMap<String, Vec<String>>,
}

impl Source {
    pub fn new(parser: Parser) -> Self {
        let mut graph = HashMap::new();
        Self::create_node_graph(&mut graph, parser.ast.clone());

        let mut stack = LinkedList::new();
        Self::check_semantics(&mut stack, parser.ast.clone());

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

    fn check_semantics(stack: &mut LinkedList<(String, String)>, ast: AbstractSyntaxTree) {
        match ast.node {
            _ => {}
        }

        for child in ast.children.clone() {
            Self::check_semantics(stack, child);
        }
    }
}
