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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum SymbolTableKey {
    ID(String),
    Int(String),
    Float(String),
}

#[derive(Clone, Debug, PartialEq)]
enum SymbolTableEntry {
    Variable(String),
    Const(String),
    Function(String, Vec<String>),
}

pub struct Source {
    graph: HashMap<String, Vec<String>>,
    symbol_table: HashMap<SymbolTableKey, SymbolTableEntry>,
    ast: AbstractSyntaxTree,
}

impl Source {
    pub fn new(parser: Parser) -> Result<Self, ()> {
        let mut graph = HashMap::new();
        Self::create_node_graph(&mut graph, parser.ast.clone());

        let mut stack = LinkedList::new();

        Self::check_semantics(&mut stack, parser.ast.clone())?;

        let mut symbol_table = HashMap::new();
        Self::generate_symbol_table(&mut symbol_table, parser.ast.clone(), String::new());

        Self::check_types(&symbol_table, parser.ast.clone(), String::new())?;

        Self::check_return(&symbol_table, parser.ast.clone(), String::new())?;

        Ok(Self {
            graph,
            symbol_table,
            ast: parser.ast.clone(),
        })
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

                Self::check_semantics(stack, decl_node.children[1].clone())?;

                while !stack.is_empty() {
                    let top = stack.pop_back().unwrap();

                    if top == ScopeElem::NodeScope(node_id.clone()) {
                        break;
                    }
                }
            }
            SyntaxTreeNode::DeclareFunc => {
                let func_id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                stack.push_back(ScopeElem::FuncScope(func_id.clone()));

                Self::check_semantics(stack, children[1].clone())?;

                Self::check_semantics(stack, children[3].clone())?;

                while !stack.is_empty() {
                    let top = stack.pop_back().unwrap();

                    if top == ScopeElem::FuncScope(func_id.clone()) {
                        stack.push_back(ScopeElem::Func(func_id));
                        break;
                    }
                }
            }
            SyntaxTreeNode::WhileLoop => {
                Self::check_semantics(stack, children[0].clone())?;

                stack.push_back(ScopeElem::WhileScope);

                Self::check_semantics(stack, children[1].clone())?;

                while !stack.is_empty() {
                    let top = stack.pop_back().unwrap();

                    if top == ScopeElem::WhileScope {
                        break;
                    }
                }
            }
            SyntaxTreeNode::IfStmt => {
                Self::check_semantics(stack, children[0].clone())?;

                stack.push_back(ScopeElem::IfScope);

                Self::check_semantics(stack, children[1].clone())?;

                while !stack.is_empty() {
                    let top = stack.pop_back().unwrap();

                    if top == ScopeElem::IfScope {
                        break;
                    }
                }

                if children[2].clone().node != SyntaxTreeNode::Null {
                    stack.push_back(ScopeElem::ElseScope);

                    Self::check_semantics(stack, children[2].clone())?;

                    while !stack.is_empty() {
                        let top = stack.pop_back().unwrap();

                        if top == ScopeElem::ElseScope {
                            break;
                        }
                    }
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

                for elem in stack.clone() {
                    if elem.clone() == ScopeElem::Variable(id.clone())
                        || elem.clone() == ScopeElem::Const(id.clone())
                    {
                        return Err(());
                    }
                }

                stack.push_back(ScopeElem::Const(id));
            }
            SyntaxTreeNode::DeclareVar => {
                Self::check_semantics(stack, children[2].clone())?;

                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                for elem in stack.clone() {
                    if elem.clone() == ScopeElem::Variable(id.clone())
                        || elem.clone() == ScopeElem::Const(id.clone())
                    {
                        return Err(());
                    }
                }

                stack.push_back(ScopeElem::Variable(id));
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
        table: &mut HashMap<SymbolTableKey, SymbolTableEntry>,
        ast: AbstractSyntaxTree,
        scope: String,
    ) {
        let children = ast.children.clone();

        match ast.node {
            SyntaxTreeNode::DeclareNode => {
                let header = children[0].clone();
                let id = match header.children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                let scope = format!("{id}");
                Self::generate_symbol_table(table, children[1].clone(), scope);
            }
            SyntaxTreeNode::DeclareConst => {
                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                let t = match children[1].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                table.insert(
                    SymbolTableKey::ID(format!("{scope}::{id}")),
                    SymbolTableEntry::Const(t),
                );

                Self::generate_symbol_table(table, children[2].clone(), scope);
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
                table.insert(
                    SymbolTableKey::ID(format!("{scope}::{id}")),
                    SymbolTableEntry::Variable(t),
                );

                Self::generate_symbol_table(table, children[2].clone(), scope);
            }
            SyntaxTreeNode::DeclareFunc => {
                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                let params = Self::get_params(children[1].clone());
                params.iter().for_each(|(p_id, p_t)| {
                    table.insert(
                        SymbolTableKey::ID(format!("{scope}::{id}::{p_id}")),
                        SymbolTableEntry::Variable(p_t.clone()),
                    );
                });
                let t = match children[2].clone().node {
                    SyntaxTreeNode::ReturnType => match children[2].children[0].clone().node {
                        SyntaxTreeNode::Identifier(id) => id,
                        _ => "".to_string(),
                    },
                    _ => "".to_string(),
                };
                let param_types = params.iter().map(|(_, p_t)| p_t.clone()).collect();
                table.insert(
                    SymbolTableKey::ID(format!("{scope}::{id}")),
                    SymbolTableEntry::Function(t, param_types),
                );

                let scope = format!("{scope}::{id}");

                Self::generate_symbol_table(table, children[3].clone(), scope);
            }
            SyntaxTreeNode::Integer(num) => {
                table.insert(
                    SymbolTableKey::Int(format!("{num}")),
                    SymbolTableEntry::Const("int".to_string()),
                );
            }
            SyntaxTreeNode::Float(num) => {
                table.insert(
                    SymbolTableKey::Float(format!("{num}")),
                    SymbolTableEntry::Const("float".to_string()),
                );
            }
            _ => {
                for child in children {
                    Self::generate_symbol_table(table, child, scope.clone());
                }
            }
        }
    }

    fn get_params(ast: AbstractSyntaxTree) -> Vec<(String, String)> {
        if ast.node == SyntaxTreeNode::Null {
            vec![]
        } else {
            let param = ast.children[0].clone();
            let param_id = match param.children[0].clone().node {
                SyntaxTreeNode::Identifier(id) => id,
                _ => "".to_string(),
            };
            let param_type = match param.children[1].clone().node {
                SyntaxTreeNode::Identifier(id) => id,
                _ => "".to_string(),
            };

            let mut ret = vec![(param_id, param_type)];
            let mut rest = Self::get_params(ast.children[1].clone());
            ret.append(&mut rest);

            ret
        }
    }

    fn check_types(
        table: &HashMap<SymbolTableKey, SymbolTableEntry>,
        ast: AbstractSyntaxTree,
        scope: String,
    ) -> Result<(), ()> {
        let children = ast.children.clone();

        match ast.node {
            SyntaxTreeNode::DeclareNode => {
                let header = children[0].clone();
                let id = match header.children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                let scope = format!("{id}");
                Self::check_types(table, children[1].clone(), scope)?;
            }
            SyntaxTreeNode::DeclareFunc => {
                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                let scope = format!("{scope}::{id}");
                Self::check_types(table, children[3].clone(), scope)?;
            }
            SyntaxTreeNode::DeclareConst => {
                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                match table.get(&SymbolTableKey::ID(format!("{scope}::{id}"))) {
                    Some(SymbolTableEntry::Const(l_value)) => {
                        if l_value.clone()
                            != Self::get_type(table, children[2].clone(), scope.clone())?
                        {
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
                match table.get(&SymbolTableKey::ID(format!("{scope}::{id}"))) {
                    Some(SymbolTableEntry::Variable(l_value)) => {
                        if l_value.clone()
                            != Self::get_type(table, children[2].clone(), scope.clone())?
                        {
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
                match table.get(&SymbolTableKey::ID(format!("{scope}::{id}"))) {
                    Some(SymbolTableEntry::Variable(l_value)) => {
                        if l_value.clone()
                            != Self::get_type(table, children[1].clone(), scope.clone())?
                        {
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
                let l_value = Self::get_type(table, children[0].clone(), scope.clone())?;
                let r_value = Self::get_type(table, children[1].clone(), scope.clone())?;

                if l_value != r_value {
                    return Err(());
                }
            }
            _ => {
                for child in children {
                    Self::check_types(table, child, scope.clone())?;
                }
            }
        }

        Ok(())
    }

    fn get_type(
        table: &HashMap<SymbolTableKey, SymbolTableEntry>,
        ast: AbstractSyntaxTree,
        scope: String,
    ) -> Result<String, ()> {
        match ast.node {
            SyntaxTreeNode::AddOp
            | SyntaxTreeNode::SubOp
            | SyntaxTreeNode::DivOp
            | SyntaxTreeNode::MulOp => {
                let l_value = Self::get_type(table, ast.children[0].clone(), scope.clone());
                let r_value = Self::get_type(table, ast.children[1].clone(), scope.clone());

                if l_value == r_value {
                    l_value
                } else {
                    Err(())
                }
            }
            SyntaxTreeNode::Integer(_) => Ok(String::from("int")),
            SyntaxTreeNode::Float(_) => Ok(String::from("float")),
            SyntaxTreeNode::Identifier(id) => {
                match table
                    .get(&SymbolTableKey::ID(format!("{scope}::{id}")))
                    .unwrap()
                {
                    SymbolTableEntry::Variable(t) => Ok(t.clone()),
                    SymbolTableEntry::Const(t) => Ok(t.clone()),
                    _ => Ok("".to_string()),
                }
            }
            SyntaxTreeNode::FnCall => {
                let params = Self::get_inputs(table, ast.children[1].clone(), scope.clone());

                match ast.children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => {
                        let idx = scope.find("::").unwrap();
                        let fn_id = format!("{}::{id}", scope.get(0..idx).unwrap());
                        match table.get(&SymbolTableKey::ID(fn_id)).unwrap() {
                            SymbolTableEntry::Function(t, p) => {
                                if params != p.clone() {
                                    return Err(());
                                }
                                Ok(t.clone())
                            }
                            _ => Ok("".to_string()),
                        }
                    }
                    _ => Ok("".to_string()),
                }
            }
            _ => Ok("".to_string()),
        }
    }

    fn get_inputs(
        table: &HashMap<SymbolTableKey, SymbolTableEntry>,
        ast: AbstractSyntaxTree,
        scope: String,
    ) -> Vec<String> {
        if ast.node == SyntaxTreeNode::Null {
            vec![]
        } else {
            let mut ret = match ast.children[0].clone().node {
                SyntaxTreeNode::Integer(num) => {
                    let id_type = match table.get(&SymbolTableKey::Int(format!("{num}"))).unwrap() {
                        SymbolTableEntry::Const(t) => t.clone(),
                        _ => "".to_string(),
                    };
                    vec![id_type]
                }
                SyntaxTreeNode::Float(num) => {
                    let id_type = match table.get(&SymbolTableKey::Float(format!("{num}"))).unwrap()
                    {
                        SymbolTableEntry::Const(t) => t.clone(),
                        _ => "".to_string(),
                    };
                    vec![id_type]
                }
                SyntaxTreeNode::Identifier(id) => {
                    let id_type = match table
                        .get(&SymbolTableKey::ID(format!("{scope}::{id}")))
                        .unwrap()
                    {
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
                    let t = Self::get_type(table, ast.children[0].clone(), scope.clone())
                        .expect("failed type check");
                    vec![t]
                }
                _ => vec![],
            };

            let mut rest = Self::get_inputs(table, ast.children[1].clone(), scope.clone());
            ret.append(&mut rest);

            ret
        }
    }

    fn check_return(
        table: &HashMap<SymbolTableKey, SymbolTableEntry>,
        ast: AbstractSyntaxTree,
        scope: String,
    ) -> Result<(), ()> {
        let children = ast.children.clone();
        match ast.node {
            SyntaxTreeNode::DeclareNode => {
                let header = children[0].clone();
                let id = match header.children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                let scope = format!("{id}");
                Self::check_return(table, children[1].clone(), scope)?;
            }
            SyntaxTreeNode::DeclareFunc => match children[2].children[0].clone().node {
                SyntaxTreeNode::Void | SyntaxTreeNode::NoReturn => {
                    Self::check_return_helper_1(children[3].clone())?;
                }
                SyntaxTreeNode::Identifier(id) => {
                    let fn_id = match children[0].clone().node {
                        SyntaxTreeNode::Identifier(id) => id,
                        _ => "".to_string(),
                    };
                    let scope = format!("{scope}::{fn_id}");
                    Self::check_return_helper_2(table, children[3].clone(), &id, scope.clone())?;
                }
                _ => {
                    return Err(());
                }
            },
            _ => {
                for child in children {
                    Self::check_return(table, child, scope.clone())?;
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
        table: &HashMap<SymbolTableKey, SymbolTableEntry>,
        ast: AbstractSyntaxTree,
        id: &String,
        scope: String,
    ) -> Result<(), ()> {
        match ast.node {
            SyntaxTreeNode::ReturnValue => {
                let return_type = Self::get_type(table, ast.children[0].clone(), scope.clone())?;
                if id.clone() != return_type {
                    return Err(());
                }
            }
            _ => {
                for child in ast.children.clone() {
                    Self::check_return_helper_2(table, child, id, scope.clone())?;
                }
            }
        }
        Ok(())
    }

    pub fn compile(&self) -> Result<(), std::io::Error> {
        if !std::path::Path::new("comp").exists() {
            std::fs::create_dir("comp")?;
        }

        Self::generate_bytecode(self.ast.clone())?;

        Ok(())
    }

    fn generate_bytecode(ast: AbstractSyntaxTree) -> Result<(), std::io::Error> {
        match ast.node {
            SyntaxTreeNode::DeclareNode => {
                let header = ast.children[0].clone();
                let id = match header.children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                let filename = format!("comp/{id}.k");
                let file = if std::path::Path::new(&filename).exists() {
                    std::fs::File::open(filename)?
                } else {
                    std::fs::File::create(filename)?
                };

                Self::generate_node_bytecode(ast.clone(), file)?;
            }
            _ => {
                for child in ast.children.clone() {
                    Self::generate_bytecode(child)?;
                }
            }
        }

        Ok(())
    }

    fn generate_node_bytecode(
        ast: AbstractSyntaxTree,
        file: std::fs::File,
    ) -> Result<(), std::io::Error> {
        Ok(())
    }
}
