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

#[derive(Clone, Debug, PartialEq)]
enum SymbolTableEntry {
    Variable(String),
    Const(String),
    Function(String, Vec<String>),
}

pub struct Source {
    graph: HashMap<String, Vec<String>>,
    symbol_table: HashMap<String, SymbolTableEntry>,
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

        let mut symbol_table = HashMap::new();
        Self::generate_symbol_table(&mut symbol_table, parser.ast.clone());

        println!("{:?}", symbol_table);

        match Self::check_types(&symbol_table, parser.ast.clone()) {
            Ok(_) => println!("type check passed"),
            Err(_) => println!("TYPE CHECK FAILED"),
        }

        match Self::check_return(&symbol_table, parser.ast.clone()) {
            Ok(_) => println!("return type check passed"),
            Err(_) => println!("RETURN TYPE CHECK FAILED"),
        }

        Self {
            graph,
            symbol_table,
        }
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

    fn generate_symbol_table(
        table: &mut HashMap<String, SymbolTableEntry>,
        ast: AbstractSyntaxTree,
    ) {
        let children = ast.children.clone();

        match ast.node {
            SyntaxTreeNode::DeclareConst => {
                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                let t = match children[1].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                table.insert(id, SymbolTableEntry::Const(t));

                Self::generate_symbol_table(table, children[2].clone());
            }
            SyntaxTreeNode::DeclareVar => {
                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                let t = match children[1].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                table.insert(id, SymbolTableEntry::Variable(t));

                Self::generate_symbol_table(table, children[2].clone());
            }
            SyntaxTreeNode::DeclareFunc => {
                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                let params = Self::get_params(children[1].clone());
                let t = match children[2].clone().node {
                    SyntaxTreeNode::ReturnType => match children[2].children[0].clone().node {
                        SyntaxTreeNode::Identifier(id) => id,
                        _ => "".to_string(),
                    },
                    _ => "".to_string(),
                };
                table.insert(id, SymbolTableEntry::Function(t, params));

                Self::generate_symbol_table(table, children[3].clone());
            }
            SyntaxTreeNode::Integer(num) => {
                table.insert(
                    format!("{}", num),
                    SymbolTableEntry::Const("int".to_string()),
                );
            }
            SyntaxTreeNode::Float(num) => {
                table.insert(
                    format!("{}", num),
                    SymbolTableEntry::Const("float".to_string()),
                );
            }
            _ => {
                for child in children {
                    Self::generate_symbol_table(table, child);
                }
            }
        }
    }

    fn get_params(ast: AbstractSyntaxTree) -> Vec<String> {
        if ast.node == SyntaxTreeNode::Null {
            vec![]
        } else {
            let param = ast.children[0].clone();
            let param_type = match param.children[1].clone().node {
                SyntaxTreeNode::Identifier(id) => id,
                _ => "".to_string(),
            };

            let mut ret = vec![param_type];
            let mut rest = Self::get_params(ast.children[1].clone());
            ret.append(&mut rest);

            ret
        }
    }

    fn check_types(
        table: &HashMap<String, SymbolTableEntry>,
        ast: AbstractSyntaxTree,
    ) -> Result<(), ()> {
        let children = ast.children.clone();

        match ast.node {
            SyntaxTreeNode::DeclareConst => {
                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                match table.get(&id) {
                    Some(SymbolTableEntry::Const(l_value)) => {
                        if l_value.clone() != Self::get_type(table, children[2].clone())? {
                            return Err(());
                        }
                    }
                    _ => {}
                }
            }
            SyntaxTreeNode::DeclareVar => {
                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                match table.get(&id) {
                    Some(SymbolTableEntry::Variable(l_value)) => {
                        if l_value.clone() != Self::get_type(table, children[2].clone())? {
                            return Err(());
                        }
                    }
                    _ => {}
                }
            }
            SyntaxTreeNode::Assign => {
                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                match table.get(&id) {
                    Some(SymbolTableEntry::Variable(l_value)) => {
                        if l_value.clone() != Self::get_type(table, children[1].clone())? {
                            return Err(());
                        }
                    }
                    _ => {}
                }
            }
            SyntaxTreeNode::AndOp
            | SyntaxTreeNode::OrOp
            | SyntaxTreeNode::CompEq
            | SyntaxTreeNode::CompNeq
            | SyntaxTreeNode::CompLeq
            | SyntaxTreeNode::CompGeq
            | SyntaxTreeNode::CompLess
            | SyntaxTreeNode::CompGreater => {
                let l_value = Self::get_type(table, children[0].clone())?;
                let r_value = Self::get_type(table, children[1].clone())?;

                if l_value != r_value {
                    return Err(());
                }
            }
            _ => {
                for child in children {
                    Self::check_types(table, child)?;
                }
            }
        }

        Ok(())
    }

    fn get_type(
        table: &HashMap<String, SymbolTableEntry>,
        ast: AbstractSyntaxTree,
    ) -> Result<String, ()> {
        println!("{:?}", ast.node);
        match ast.node {
            SyntaxTreeNode::AddOp
            | SyntaxTreeNode::SubOp
            | SyntaxTreeNode::DivOp
            | SyntaxTreeNode::MulOp => {
                let l_value = Self::get_type(table, ast.children[0].clone());
                let r_value = Self::get_type(table, ast.children[1].clone());

                if l_value == r_value {
                    l_value
                } else {
                    Err(())
                }
            }
            SyntaxTreeNode::Integer(_) => Ok(String::from("int")),
            SyntaxTreeNode::Float(_) => Ok(String::from("float")),
            SyntaxTreeNode::Identifier(id) => match table.get(&id).unwrap() {
                SymbolTableEntry::Variable(t) => Ok(t.clone()),
                SymbolTableEntry::Const(t) => Ok(t.clone()),
                _ => Ok("".to_string()),
            },
            SyntaxTreeNode::FnCall => {
                let params = Self::get_inputs(table, ast.children[1].clone());

                match ast.children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => match table.get(&id).unwrap() {
                        SymbolTableEntry::Function(t, p) => {
                            if params != p.clone() {
                                println!("{:?} {:?}", params, p);
                                return Err(());
                            }
                            Ok(t.clone())
                        }
                        _ => Ok("".to_string()),
                    },
                    _ => Ok("".to_string()),
                }
            }
            _ => Ok("".to_string()),
        }
    }

    fn get_inputs(
        table: &HashMap<String, SymbolTableEntry>,
        ast: AbstractSyntaxTree,
    ) -> Vec<String> {
        if ast.node == SyntaxTreeNode::Null {
            vec![]
        } else {
            let mut ret = match ast.children[0].clone().node {
                SyntaxTreeNode::Integer(num) => {
                    let id_type = match table.get(&format!("{num}")).unwrap() {
                        SymbolTableEntry::Const(t) => t.clone(),
                        _ => "".to_string(),
                    };
                    vec![id_type]
                }
                SyntaxTreeNode::Float(num) => {
                    let id_type = match table.get(&format!("{num}")).unwrap() {
                        SymbolTableEntry::Const(t) => t.clone(),
                        _ => "".to_string(),
                    };
                    vec![id_type]
                }
                SyntaxTreeNode::Identifier(id) => {
                    let id_type = match table.get(&id).unwrap() {
                        SymbolTableEntry::Const(t) => t.clone(),
                        SymbolTableEntry::Variable(t) => t.clone(),
                        _ => "".to_string(),
                    };
                    vec![id_type]
                }
                SyntaxTreeNode::AddOp
                | SyntaxTreeNode::SubOp
                | SyntaxTreeNode::MulOp
                | SyntaxTreeNode::DivOp => {
                    let t =
                        Self::get_type(table, ast.children[0].clone()).expect("failed type check");
                    vec![t]
                }
                _ => vec![],
            };

            let mut rest = Self::get_inputs(table, ast.children[1].clone());
            ret.append(&mut rest);

            ret
        }
    }

    fn check_return(
        table: &HashMap<String, SymbolTableEntry>,
        ast: AbstractSyntaxTree,
    ) -> Result<(), ()> {
        let children = ast.children.clone();
        match ast.node {
            SyntaxTreeNode::DeclareFunc => match children[2].children[0].clone().node {
                SyntaxTreeNode::Void | SyntaxTreeNode::NoReturn => {
                    Self::check_return_helper_1(children[3].clone())?;
                }
                SyntaxTreeNode::Identifier(id) => {
                    Self::check_return_helper_2(table, children[3].clone(), &id)?;
                }
                _ => {
                    return Err(());
                }
            },
            _ => {
                for child in children {
                    Self::check_return(table, child)?;
                }
            }
        }

        Ok(())
    }

    fn check_return_helper_1(ast: AbstractSyntaxTree) -> Result<(), ()> {
        match ast.node {
            SyntaxTreeNode::ReturnValue => {
                return Err(());
            }
            _ => {
                for child in ast.children.clone() {
                    Self::check_return_helper_1(child)?;
                }
            }
        }

        Ok(())
    }

    fn check_return_helper_2(
        table: &HashMap<String, SymbolTableEntry>,
        ast: AbstractSyntaxTree,
        id: &String,
    ) -> Result<(), ()> {
        match ast.node {
            SyntaxTreeNode::ReturnValue => {
                let return_type = Self::get_type(table, ast.children[0].clone())?;
                if id.clone() != return_type {
                    return Err(());
                }
            }
            _ => {
                for child in ast.children.clone() {
                    Self::check_return_helper_2(table, child, id)?;
                }
            }
        }
        Ok(())
    }

    pub fn generate_assembly(&self) {}
}
