use std::{
    collections::{HashMap, HashSet, LinkedList},
    hash::Hash,
};

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
}

#[derive(Debug, Clone)]
enum TLElement {
    Function(String, Vec<String>, HashSet<(String, String)>),
    Struct,
    Export,
}

pub struct Source {
    graph: HashMap<String, Vec<String>>,
    symbol_table: HashMap<String, HashMap<String, TLElement>>,
    ast: AbstractSyntaxTree,
}

impl Source {
    pub fn new(parser: Parser) -> Result<Self, ()> {
        let mut graph = HashMap::new();
        Self::create_node_graph(&mut graph, parser.ast.clone());

        let mut symbol_table = HashMap::new();
        Self::seed_symbol_table(&mut symbol_table, parser.ast.clone())?;
        println!("{:?}", symbol_table);

        let mut stack = LinkedList::new();
        Self::check_semantics(&mut stack, &mut symbol_table, parser.ast.clone())?;
        println!("{:?}", symbol_table);

        Self::check_types(&symbol_table, parser.ast.clone())?;

        // Self::check_return(&symbol_table, parser.ast.clone(), String::new())?;

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

    fn seed_symbol_table(
        symbol_table: &mut HashMap<String, HashMap<String, TLElement>>,
        ast: AbstractSyntaxTree,
    ) -> Result<(), ()> {
        match ast.node {
            SyntaxTreeNode::DeclareNode => {
                let header = ast.children[0].clone();
                let id = match header.children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                if symbol_table.contains_key(&id) {
                    return Err(());
                }

                Self::sst_node(symbol_table, ast.children[1].clone(), id)?;
            }
            _ => {
                for child in ast.children.clone() {
                    Self::seed_symbol_table(symbol_table, child)?;
                }
            }
        }

        Ok(())
    }

    fn sst_node(
        symbol_table: &mut HashMap<String, HashMap<String, TLElement>>,
        ast: AbstractSyntaxTree,
        node_id: String,
    ) -> Result<(), ()> {
        match ast.node {
            SyntaxTreeNode::DeclareFunc => {
                let id = match ast.children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                let ret = match ast.children[2].clone().children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                let params = Self::sst_func(ast.children[1].clone())?;

                let param_types = params.iter().map(|(_, t)| t.clone()).collect();

                let entry = TLElement::Function(ret, param_types, params);

                let mut map = match symbol_table.get(&node_id) {
                    Some(m) => m.clone(),
                    None => HashMap::new(),
                };

                if map.contains_key(&id) {
                    return Err(());
                }

                map.insert(id, entry);
                symbol_table.insert(node_id, map);
            }
            _ => {
                for child in ast.children.clone() {
                    Self::sst_node(symbol_table, child, node_id.clone())?;
                }
            }
        }

        Ok(())
    }

    fn sst_func(ast: AbstractSyntaxTree) -> Result<HashSet<(String, String)>, ()> {
        match ast.node {
            SyntaxTreeNode::ParamList => {
                let param = Self::sst_func(ast.children[0].clone())?;
                let mut rest = Self::sst_func(ast.children[1].clone())?;

                for p in param {
                    if rest.contains(&p) {
                        return Err(());
                    }
                    rest.insert(p);
                }

                Ok(rest)
            }
            SyntaxTreeNode::Param => {
                let id = match ast.children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                let t = match ast.children[1].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                Ok(HashSet::from([(id, t)]))
            }
            _ => Ok(HashSet::new()),
        }
    }

    fn check_semantics(
        stack: &mut LinkedList<ScopeElem>,
        symbol_table: &mut HashMap<String, HashMap<String, TLElement>>,
        ast: AbstractSyntaxTree,
    ) -> Result<(), ()> {
        let children = ast.children.clone();

        match ast.node {
            SyntaxTreeNode::DeclareNode => {
                let header = children[0].clone();
                let id = match header.children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                stack.push_back(ScopeElem::NodeScope(id.clone()));

                Self::check_semantics(stack, symbol_table, children[1].clone())?;

                while !stack.is_empty() {
                    let top = stack.pop_back().unwrap();

                    if top == ScopeElem::NodeScope(id.clone()) {
                        break;
                    }
                }
            }
            SyntaxTreeNode::DeclareFunc => {
                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                stack.push_back(ScopeElem::FuncScope(id.clone()));

                Self::check_semantics(stack, symbol_table, children[1].clone())?;

                Self::check_semantics(stack, symbol_table, children[3].clone())?;

                while !stack.is_empty() {
                    let top = stack.pop_back().unwrap();

                    if top == ScopeElem::FuncScope(id.clone()) {
                        break;
                    }
                }
            }
            SyntaxTreeNode::DeclareConst => {
                Self::check_semantics(stack, symbol_table, children[2].clone())?;

                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                let t = match children[1].clone().node {
                    SyntaxTreeNode::Identifier(i) => i,
                    _ => "".to_string(),
                };

                let mut node_id = String::new();
                let mut fn_id = String::new();
                for elem in stack.clone() {
                    if elem == ScopeElem::Const(id.clone()) {
                        return Err(());
                    }

                    match elem {
                        ScopeElem::NodeScope(n) => {
                            node_id = n.clone();
                        }
                        ScopeElem::FuncScope(f) => {
                            fn_id = f.clone();
                        }
                        _ => {}
                    }
                }

                let mut map = symbol_table[&node_id].clone();
                let e = map[&fn_id].clone();
                let (ret_t, in_t, mut var_set) = match e {
                    TLElement::Function(ret, t, s) => (ret, t, s),
                    _ => (String::new(), vec![], HashSet::new()),
                };

                var_set.insert((id.clone(), t.clone()));
                map.insert(fn_id, TLElement::Function(ret_t, in_t, var_set));
                symbol_table.insert(node_id, map);

                stack.push_back(ScopeElem::Const(id));
            }
            SyntaxTreeNode::DeclareVar => {
                Self::check_semantics(stack, symbol_table, children[2].clone())?;

                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                let t = match children[1].clone().node {
                    SyntaxTreeNode::Identifier(i) => i,
                    _ => "".to_string(),
                };

                let mut node_id = String::new();
                let mut fn_id = String::new();
                for elem in stack.clone() {
                    if elem == ScopeElem::Variable(id.clone()) {
                        return Err(());
                    }

                    match elem {
                        ScopeElem::NodeScope(n) => {
                            node_id = n.clone();
                        }
                        ScopeElem::FuncScope(f) => {
                            fn_id = f.clone();
                        }
                        _ => {}
                    }
                }

                let mut map = symbol_table[&node_id].clone();
                let e = map[&fn_id].clone();
                let (ret_t, in_t, mut var_set) = match e {
                    TLElement::Function(ret, t, s) => (ret, t, s),
                    _ => (String::new(), vec![], HashSet::new()),
                };

                var_set.insert((id.clone(), t.clone()));
                map.insert(fn_id, TLElement::Function(ret_t, in_t, var_set));
                symbol_table.insert(node_id, map);

                stack.push_back(ScopeElem::Variable(id));
            }
            SyntaxTreeNode::Assign => {
                Self::check_semantics(stack, symbol_table, children[1].clone())?;

                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                for elem in stack.clone() {
                    if elem == ScopeElem::Variable(id.clone()) {
                        return Ok(());
                    }
                }

                return Err(());
            }
            SyntaxTreeNode::WhileLoop => {
                stack.push_back(ScopeElem::WhileScope);

                Self::check_semantics(stack, symbol_table, children[1].clone())?;

                while !stack.is_empty() {
                    let top = stack.pop_back().unwrap();

                    if top == ScopeElem::WhileScope {
                        break;
                    }
                }
            }
            SyntaxTreeNode::IfStmt => {
                stack.push_back(ScopeElem::IfScope);

                Self::check_semantics(stack, symbol_table, children[1].clone())?;

                while !stack.is_empty() {
                    let top = stack.pop_back().unwrap();

                    if top == ScopeElem::IfScope {
                        break;
                    }
                }

                if children[2].clone().node != SyntaxTreeNode::Null {
                    stack.push_back(ScopeElem::ElseScope);

                    Self::check_semantics(stack, symbol_table, children[2].clone())?;

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

                Self::check_semantics(stack, symbol_table, children[1].clone())?;
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
                Self::check_semantics(stack, symbol_table, children[0].clone())?;
                Self::check_semantics(stack, symbol_table, children[1].clone())?;
            }
            SyntaxTreeNode::Identifier(id) => {
                for elem in stack {
                    if elem.clone() == ScopeElem::Variable(id.clone())
                        || elem.clone() == ScopeElem::Const(id.clone())
                    {
                        return Ok(());
                    }
                }
                println!("{id}");
                return Err(());
            }
            SyntaxTreeNode::FnCall => {
                Self::check_semantics(stack, symbol_table, children[1].clone())?;

                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                let mut node_id = String::new();
                for elem in stack {
                    match elem {
                        ScopeElem::NodeScope(n) => {
                            node_id = n.clone();
                            break;
                        }
                        _ => {}
                    }
                }

                let map = symbol_table[&node_id].clone();
                if map.contains_key(&id) {
                    return Ok(());
                } else {
                    return Err(());
                }
            }
            _ => {
                for child in children {
                    Self::check_semantics(stack, symbol_table, child)?;
                }
            }
        }

        Ok(())
    }

    fn check_types(
        symbol_table: &HashMap<String, HashMap<String, TLElement>>,
        ast: AbstractSyntaxTree,
    ) -> Result<(), ()> {
        let children = ast.children.clone();
        match ast.node {
            SyntaxTreeNode::DeclareNode => {
                let header = children[0].clone();
                let id = match header.children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                Self::check_types_node(symbol_table, children[1].clone(), id)?;
            }
            _ => {
                for child in children {
                    Self::check_types(symbol_table, child)?;
                }
            }
        }

        Ok(())
    }

    fn check_types_node(
        symbol_table: &HashMap<String, HashMap<String, TLElement>>,
        ast: AbstractSyntaxTree,
        node_id: String,
    ) -> Result<(), ()> {
        let children = ast.children.clone();
        match ast.node {
            SyntaxTreeNode::DeclareFunc => {
                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                Self::check_types_func(symbol_table, children[3].clone(), node_id, id)?;
            }
            _ => {
                for child in children {
                    Self::check_types_node(symbol_table, child, node_id.clone())?;
                }
            }
        }
        Ok(())
    }

    fn check_types_func(
        symbol_table: &HashMap<String, HashMap<String, TLElement>>,
        ast: AbstractSyntaxTree,
        node_id: String,
        fn_id: String,
    ) -> Result<(), ()> {
        let children = ast.children.clone();

        match ast.node {
            SyntaxTreeNode::DeclareVar | SyntaxTreeNode::DeclareConst => {
                let l_value = match children[1].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                let r_value = Self::get_type(symbol_table, children[2].clone(), node_id, fn_id)?;

                if l_value != r_value {
                    println!("{l_value} {r_value}");
                    return Err(());
                }
            }
            SyntaxTreeNode::Assign => {
                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                println!("{id}");

                let l_value = Self::get_type(
                    symbol_table,
                    children[0].clone(),
                    node_id.clone(),
                    fn_id.clone(),
                )?;

                let r_value = Self::get_type(symbol_table, children[1].clone(), node_id, fn_id)?;

                if l_value != r_value {
                    println!("{l_value} {r_value}");
                    return Err(());
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
                let l_value = Self::get_type(
                    symbol_table,
                    children[0].clone(),
                    node_id.clone(),
                    fn_id.clone(),
                )?;
                let r_value = Self::get_type(
                    symbol_table,
                    children[1].clone(),
                    node_id.clone(),
                    fn_id.clone(),
                )?;

                if l_value != r_value {
                    return Err(());
                }
            }
            _ => {
                for child in children {
                    Self::check_types_func(symbol_table, child, node_id.clone(), fn_id.clone())?;
                }
            }
        }

        Ok(())
    }

    fn get_type(
        symbol_table: &HashMap<String, HashMap<String, TLElement>>,
        ast: AbstractSyntaxTree,
        node_id: String,
        fn_id: String,
    ) -> Result<String, ()> {
        let children = ast.children.clone();
        match ast.node {
            SyntaxTreeNode::AddOp
            | SyntaxTreeNode::SubOp
            | SyntaxTreeNode::DivOp
            | SyntaxTreeNode::MulOp => {
                let l_value = Self::get_type(
                    symbol_table,
                    children[0].clone(),
                    node_id.clone(),
                    fn_id.clone(),
                );
                let r_value = Self::get_type(
                    symbol_table,
                    children[1].clone(),
                    node_id.clone(),
                    fn_id.clone(),
                );

                if l_value == r_value {
                    l_value
                } else {
                    Err(())
                }
            }
            SyntaxTreeNode::Identifier(id) => {
                let map = symbol_table[&node_id].clone();
                let e = map[&fn_id].clone();

                match e {
                    TLElement::Function(_, _, set) => {
                        for (var_name, var_type) in set {
                            if var_name == id {
                                return Ok(var_type);
                            }
                        }
                        Err(())
                    }
                    _ => Err(()),
                }
            }
            SyntaxTreeNode::Integer(_) => Ok(String::from("int")),
            SyntaxTreeNode::Float(_) => Ok(String::from("float")),
            SyntaxTreeNode::FnCall => {
                let params = Self::get_inputs(
                    symbol_table,
                    children[1].clone(),
                    node_id.clone(),
                    fn_id.clone(),
                );

                match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => {
                        let map = symbol_table[&node_id].clone();
                        let e = match map.get(&id) {
                            Some(elem) => elem,
                            None => {
                                return Err(());
                            }
                        };

                        match e {
                            TLElement::Function(ret, p, _) => {
                                if p.clone() == params {
                                    Ok(ret.clone())
                                } else {
                                    Err(())
                                }
                            }
                            _ => Err(()),
                        }
                    }
                    _ => Ok("".to_string()),
                }
            }
            _ => Ok(String::new()),
        }
    }

    fn get_inputs(
        symbol_table: &HashMap<String, HashMap<String, TLElement>>,
        ast: AbstractSyntaxTree,
        node_id: String,
        fn_id: String,
    ) -> Vec<String> {
        if ast.node == SyntaxTreeNode::Null {
            vec![]
        } else {
            let mut ret = match ast.children[0].clone().node {
                SyntaxTreeNode::Integer(_) => {
                    vec![String::from("int")]
                }
                SyntaxTreeNode::Float(_) => {
                    vec![String::from("float")]
                }
                SyntaxTreeNode::Identifier(id) => {
                    let map = symbol_table[&node_id].clone();
                    let e = map[&fn_id].clone();

                    match e {
                        TLElement::Function(_, _, set) => {
                            for (var_name, var_type) in set {
                                if var_name == id {
                                    return vec![var_type];
                                }
                            }
                            vec![]
                        }
                        _ => vec![],
                    }
                }
                SyntaxTreeNode::AddOp
                | SyntaxTreeNode::SubOp
                | SyntaxTreeNode::MulOp
                | SyntaxTreeNode::DivOp => {
                    let t = Self::get_type(
                        symbol_table,
                        ast.children[0].clone(),
                        node_id.clone(),
                        fn_id.clone(),
                    )
                    .expect("failed type check");
                    vec![t]
                }
                _ => vec![],
            };

            let mut rest = Self::get_inputs(
                symbol_table,
                ast.children[1].clone(),
                node_id.clone(),
                fn_id.clone(),
            );
            ret.append(&mut rest);

            ret
        }
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
