use std::{
    collections::{HashMap, HashSet, LinkedList},
    fs::OpenOptions,
    io::Write,
};

use crate::parser::{AbstractSyntaxTree, Parser, SyntaxTreeNode};

#[derive(Clone, Debug, PartialEq)]
enum ScopeElem {
    IfScope,
    WhileScope,
    ElseScope,
    Variable(String),
    Const(String),
    Func(String),
}

#[derive(Debug, Clone)]
enum TLElement {
    Function(
        String,
        Vec<(String, String)>,
        HashSet<(String, String)>,
        AbstractSyntaxTree,
    ),
    Struct,
    Export,
}

pub struct Source {
    graph: HashMap<String, Vec<String>>,
    symbol_table: HashMap<String, HashMap<String, TLElement>>,
}

impl Source {
    pub fn new(parser: Parser) -> Result<Self, usize> {
        let mut graph = HashMap::new();
        Self::create_node_graph(&mut graph, parser.ast.clone());

        let mut symbol_table = HashMap::new();
        Self::seed_symbol_table(&mut symbol_table, parser.ast.clone())?;

        Self::check_semantics(&mut symbol_table)?;

        Ok(Self {
            graph,
            symbol_table,
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
    ) -> Result<(), usize> {
        match ast.node {
            SyntaxTreeNode::DeclareNode => {
                let header = ast.children[0].clone();
                let id = match header.children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                if symbol_table.contains_key(&id) {
                    return Err(1);
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
    ) -> Result<(), usize> {
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
                let set = HashSet::from_iter(params.clone());

                let entry = TLElement::Function(ret, params, set, ast.children[3].clone());

                let mut map = match symbol_table.get(&node_id) {
                    Some(m) => m.clone(),
                    None => HashMap::new(),
                };

                if map.contains_key(&id) {
                    return Err(2);
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

    fn sst_func(ast: AbstractSyntaxTree) -> Result<Vec<(String, String)>, usize> {
        match ast.node {
            SyntaxTreeNode::ParamList => {
                let param = Self::sst_func(ast.children[0].clone())?;
                let mut rest = Self::sst_func(ast.children[1].clone())?;

                for p in param {
                    if rest.contains(&p) {
                        return Err(3);
                    }
                    rest.push(p);
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

                Ok(vec![(id, t)])
            }
            _ => Ok(vec![]),
        }
    }

    fn check_semantics(
        symbol_table: &mut HashMap<String, HashMap<String, TLElement>>,
    ) -> Result<(), usize> {
        let mut functions = vec![];
        for (_, node_tl) in symbol_table.clone() {
            for (tl_id, tl_elem) in node_tl {
                match tl_elem {
                    TLElement::Function(ret, params, _, _) => {
                        functions.push((tl_id, ret, params));
                    }
                    _ => {}
                }
            }
        }

        for (_, node_tl) in symbol_table {
            for (_, tl_elem) in node_tl {
                match tl_elem {
                    TLElement::Function(ret, _, set, tree) => {
                        let mut stack = LinkedList::new();
                        for (func_name, _, _) in functions.clone() {
                            stack.push_back(ScopeElem::Func(func_name.clone()));
                        }

                        for (var_id, _) in set.clone() {
                            stack.push_back(ScopeElem::Variable(var_id));
                        }

                        Self::check_semantics_helper(&mut stack, set, tree.clone())?;

                        Self::check_types(functions.clone(), set.clone(), tree.clone())?;
                        Self::check_return(
                            functions.clone(),
                            set.clone(),
                            tree.clone(),
                            ret.clone(),
                        )?;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn check_semantics_helper(
        stack: &mut LinkedList<ScopeElem>,
        var_set: &mut HashSet<(String, String)>,
        ast: AbstractSyntaxTree,
    ) -> Result<(), usize> {
        let children = ast.children.clone();

        match ast.node {
            SyntaxTreeNode::DeclareConst => {
                Self::check_semantics_helper(stack, var_set, children[2].clone())?;

                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                let t = match children[1].clone().node {
                    SyntaxTreeNode::Identifier(i) => i,
                    _ => "".to_string(),
                };

                for elem in stack.clone() {
                    if elem == ScopeElem::Const(id.clone())
                        || elem == ScopeElem::Variable(id.clone())
                    {
                        return Err(4);
                    }
                }

                var_set.insert((id.clone(), t.clone()));

                stack.push_back(ScopeElem::Const(id));
            }
            SyntaxTreeNode::DeclareVar => {
                Self::check_semantics_helper(stack, var_set, children[2].clone())?;

                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                let t = match children[1].clone().node {
                    SyntaxTreeNode::Identifier(i) => i,
                    _ => "".to_string(),
                };

                for elem in stack.clone() {
                    if elem == ScopeElem::Variable(id.clone())
                        || elem == ScopeElem::Const(id.clone())
                    {
                        return Err(5);
                    }
                }

                var_set.insert((id.clone(), t.clone()));

                stack.push_back(ScopeElem::Variable(id));
            }
            SyntaxTreeNode::Assign => {
                Self::check_semantics_helper(stack, var_set, children[1].clone())?;

                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                for elem in stack.clone() {
                    if elem == ScopeElem::Variable(id.clone()) {
                        return Ok(());
                    }
                }

                return Err(16);
            }
            SyntaxTreeNode::WhileLoop => {
                stack.push_back(ScopeElem::WhileScope);

                Self::check_semantics_helper(stack, var_set, children[1].clone())?;

                while !stack.is_empty() {
                    let top = stack.pop_back().unwrap();

                    if top == ScopeElem::WhileScope {
                        break;
                    }
                }
            }
            SyntaxTreeNode::IfStmt => {
                stack.push_back(ScopeElem::IfScope);

                Self::check_semantics_helper(stack, var_set, children[1].clone())?;

                while !stack.is_empty() {
                    let top = stack.pop_back().unwrap();

                    if top == ScopeElem::IfScope {
                        break;
                    }
                }

                if children[2].clone().node != SyntaxTreeNode::Null {
                    stack.push_back(ScopeElem::ElseScope);

                    Self::check_semantics_helper(stack, var_set, children[2].clone())?;

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

                Self::check_semantics_helper(stack, var_set, children[1].clone())?;
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
                Self::check_semantics_helper(stack, var_set, children[0].clone())?;
                Self::check_semantics_helper(stack, var_set, children[1].clone())?;
            }
            SyntaxTreeNode::Identifier(id) => {
                for elem in stack {
                    if elem.clone() == ScopeElem::Variable(id.clone())
                        || elem.clone() == ScopeElem::Const(id.clone())
                    {
                        return Ok(());
                    }
                }
                return Err(6);
            }
            SyntaxTreeNode::FnCall => {
                Self::check_semantics_helper(stack, var_set, children[1].clone())?;

                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                for elem in stack {
                    if elem.clone() == ScopeElem::Func(id.clone()) {
                        return Ok(());
                    }
                }

                return Err(7);
            }
            _ => {
                for child in children {
                    Self::check_semantics_helper(stack, var_set, child)?;
                }
            }
        }

        Ok(())
    }

    fn check_types(
        functions: Vec<(String, String, Vec<(String, String)>)>,
        var_set: HashSet<(String, String)>,
        ast: AbstractSyntaxTree,
    ) -> Result<(), usize> {
        let children = ast.children.clone();

        match ast.node {
            SyntaxTreeNode::DeclareVar | SyntaxTreeNode::DeclareConst => {
                let l_value = match children[1].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                let r_value = Self::get_type(functions, var_set, children[2].clone())?;

                if l_value != r_value {
                    return Err(8);
                }
            }
            SyntaxTreeNode::Assign => {
                let l_value =
                    Self::get_type(functions.clone(), var_set.clone(), children[0].clone())?;

                let r_value =
                    Self::get_type(functions.clone(), var_set.clone(), children[1].clone())?;

                if l_value != r_value {
                    return Err(9);
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
                let l_value =
                    Self::get_type(functions.clone(), var_set.clone(), children[0].clone())?;
                let r_value =
                    Self::get_type(functions.clone(), var_set.clone(), children[1].clone())?;

                if l_value != r_value {
                    return Err(10);
                }
            }
            _ => {
                for child in children {
                    Self::check_types(functions.clone(), var_set.clone(), child)?;
                }
            }
        }

        Ok(())
    }

    fn get_type(
        functions: Vec<(String, String, Vec<(String, String)>)>,
        var_set: HashSet<(String, String)>,
        ast: AbstractSyntaxTree,
    ) -> Result<String, usize> {
        let children = ast.children.clone();
        match ast.node {
            SyntaxTreeNode::AddOp
            | SyntaxTreeNode::SubOp
            | SyntaxTreeNode::DivOp
            | SyntaxTreeNode::MulOp => {
                let l_value =
                    Self::get_type(functions.clone(), var_set.clone(), children[0].clone())?;
                let r_value =
                    Self::get_type(functions.clone(), var_set.clone(), children[1].clone())?;

                if l_value == r_value {
                    Ok(l_value)
                } else {
                    Err(11)
                }
            }
            SyntaxTreeNode::Identifier(id) => {
                for (var_id, var_type) in var_set {
                    if var_id == id {
                        return Ok(var_type.clone());
                    }
                }

                Err(12)
            }
            SyntaxTreeNode::Integer(_) => Ok(String::from("int")),
            SyntaxTreeNode::Float(_) => Ok(String::from("float")),
            SyntaxTreeNode::FnCall => {
                let params =
                    Self::get_inputs(functions.clone(), var_set.clone(), children[1].clone());

                match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => {
                        for (fn_id, fn_type, fn_params) in functions.clone() {
                            let fn_params: Vec<String> =
                                fn_params.iter().map(|(_, t)| t.clone()).collect();
                            if fn_id == id && fn_params == params {
                                return Ok(fn_type.clone());
                            }
                        }

                        Err(13)
                    }
                    _ => Ok("".to_string()),
                }
            }
            _ => Ok(String::new()),
        }
    }

    fn get_inputs(
        functions: Vec<(String, String, Vec<(String, String)>)>,
        var_set: HashSet<(String, String)>,
        ast: AbstractSyntaxTree,
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
                    for (var_id, var_type) in var_set.clone() {
                        if var_id == id {
                            return vec![var_type];
                        }
                    }

                    vec![]
                }
                SyntaxTreeNode::AddOp
                | SyntaxTreeNode::SubOp
                | SyntaxTreeNode::MulOp
                | SyntaxTreeNode::DivOp => {
                    let t =
                        Self::get_type(functions.clone(), var_set.clone(), ast.children[0].clone())
                            .expect("failed type check");
                    vec![t]
                }
                _ => vec![],
            };

            let mut rest =
                Self::get_inputs(functions.clone(), var_set.clone(), ast.children[1].clone());
            ret.append(&mut rest);

            ret
        }
    }

    fn check_return(
        functions: Vec<(String, String, Vec<(String, String)>)>,
        var_set: HashSet<(String, String)>,
        ast: AbstractSyntaxTree,
        ret: String,
    ) -> Result<(), usize> {
        if ret == "" {
            Self::check_return_func_1(ast)?;
        } else {
            Self::check_return_func_2(functions, var_set, ast, ret)?;
        }

        Ok(())
    }

    fn check_return_func_1(ast: AbstractSyntaxTree) -> Result<(), usize> {
        let children = ast.children.clone();
        match ast.node {
            SyntaxTreeNode::ReturnValue => {
                return Err(14);
            }
            _ => {
                for child in children {
                    Self::check_return_func_1(child)?;
                }
            }
        }

        Ok(())
    }

    fn check_return_func_2(
        functions: Vec<(String, String, Vec<(String, String)>)>,
        var_set: HashSet<(String, String)>,
        ast: AbstractSyntaxTree,
        ret_type: String,
    ) -> Result<(), usize> {
        let children = ast.children.clone();
        match ast.node {
            SyntaxTreeNode::ReturnValue => {
                let t = Self::get_type(functions, var_set, children[0].clone())?;
                if t != ret_type {
                    return Err(15);
                }
            }
            _ => {
                for child in children {
                    Self::check_return_func_2(
                        functions.clone(),
                        var_set.clone(),
                        child,
                        ret_type.clone(),
                    )?;
                }
            }
        }

        Ok(())
    }

    pub fn compile(&self) -> Result<(), std::io::Error> {
        if !std::path::Path::new("comp").exists() {
            std::fs::create_dir("comp")?;
        }

        self.generate_bytecode()?;

        Ok(())
    }

    fn generate_bytecode(&self) -> Result<(), std::io::Error> {
        let mut functions = vec![];
        for (_, node_tl) in self.symbol_table.clone() {
            for (tl_id, tl_elem) in node_tl {
                match tl_elem {
                    TLElement::Function(ret, params, _, _) => {
                        functions.push((tl_id, ret, params));
                    }
                    _ => {}
                }
            }
        }

        for node_id in self.symbol_table.keys() {
            let filename = format!("comp/{node_id}.k");
            let mut file = if std::path::Path::new(&filename).exists() {
                OpenOptions::new().write(true).open(filename).unwrap()
            } else {
                std::fs::File::create(filename.clone())?
            };

            let mut bytes: Vec<u8> = vec![];
            let mut function_locations: HashMap<String, usize> = HashMap::new();
            let mut variable_addresses: HashMap<String, (String, u32)> = HashMap::new();
            let mut calls = vec![];
            let mut addr: u32 = 0x0;

            match self.symbol_table[node_id]["main"].clone() {
                TLElement::Function(_, params, var_set, tree) => {
                    function_locations.insert("main".to_string(), bytes.len());

                    for (var_id, var_type) in var_set.clone() {
                        variable_addresses.insert(var_id, (var_type.clone(), addr));
                        if var_type == "int" {
                            bytes.push(0x20);

                            bytes.push(((addr & 0xFF000000) >> 24) as u8);
                            bytes.push(((addr & 0x00FF0000) >> 16) as u8);
                            bytes.push(((addr & 0x0000FF00) >> 8) as u8);
                            bytes.push((addr & 0x000000FF) as u8);

                            addr += 4;
                        } else if var_type == "float" {
                            bytes.push(0x21);

                            bytes.push(((addr & 0xFF000000) >> 24) as u8);
                            bytes.push(((addr & 0x00FF0000) >> 16) as u8);
                            bytes.push(((addr & 0x0000FF00) >> 8) as u8);
                            bytes.push((addr & 0x000000FF) as u8);

                            addr += 4;
                        }
                    }

                    for (param_id, param_type) in params.clone() {
                        let addr = variable_addresses[&param_id].1;
                        if param_type == "int" {
                            bytes.push(0x24);

                            bytes.push(((addr & 0xFF000000) >> 24) as u8);
                            bytes.push(((addr & 0x00FF0000) >> 16) as u8);
                            bytes.push(((addr & 0x0000FF00) >> 8) as u8);
                            bytes.push((addr & 0x000000FF) as u8);
                        } else if param_type == "float" {
                            bytes.push(0x25);

                            bytes.push(((addr & 0xFF000000) >> 24) as u8);
                            bytes.push(((addr & 0x00FF0000) >> 16) as u8);
                            bytes.push(((addr & 0x0000FF00) >> 8) as u8);
                            bytes.push((addr & 0x000000FF) as u8);
                        }
                    }
                    Self::generate_function_bytecode(
                        &mut bytes,
                        "main".to_string(),
                        &functions,
                        &var_set,
                        &variable_addresses,
                        &mut calls,
                        tree,
                    );
                }
                _ => {}
            }

            for fn_id in self.symbol_table[node_id].keys() {
                println!("{fn_id}");
                if fn_id == "main" {
                    continue;
                }
                match self.symbol_table[node_id][fn_id].clone() {
                    TLElement::Function(_, params, var_set, tree) => {
                        function_locations.insert(fn_id.clone(), bytes.len());

                        for (var_id, var_type) in var_set.clone() {
                            variable_addresses.insert(var_id, (var_type.clone(), addr));
                            if var_type == "int" {
                                bytes.push(0x20);

                                bytes.push(((addr & 0xFF000000) >> 24) as u8);
                                bytes.push(((addr & 0x00FF0000) >> 16) as u8);
                                bytes.push(((addr & 0x0000FF00) >> 8) as u8);
                                bytes.push((addr & 0x000000FF) as u8);

                                addr += 4;
                            } else if var_type == "float" {
                                bytes.push(0x21);

                                bytes.push(((addr & 0xFF000000) >> 24) as u8);
                                bytes.push(((addr & 0x00FF0000) >> 16) as u8);
                                bytes.push(((addr & 0x0000FF00) >> 8) as u8);
                                bytes.push((addr & 0x000000FF) as u8);

                                addr += 4;
                            }
                        }

                        for (param_id, param_type) in params.clone() {
                            let addr = variable_addresses[&param_id].1;
                            if param_type == "int" {
                                bytes.push(0x24);

                                bytes.push(((addr & 0xFF000000) >> 24) as u8);
                                bytes.push(((addr & 0x00FF0000) >> 16) as u8);
                                bytes.push(((addr & 0x0000FF00) >> 8) as u8);
                                bytes.push((addr & 0x000000FF) as u8);
                            } else if param_type == "float" {
                                bytes.push(0x25);

                                bytes.push(((addr & 0xFF000000) >> 24) as u8);
                                bytes.push(((addr & 0x00FF0000) >> 16) as u8);
                                bytes.push(((addr & 0x0000FF00) >> 8) as u8);
                                bytes.push((addr & 0x000000FF) as u8);
                            }
                        }
                        Self::generate_function_bytecode(
                            &mut bytes,
                            fn_id.clone(),
                            &functions,
                            &var_set,
                            &variable_addresses,
                            &mut calls,
                            tree,
                        );
                    }
                    _ => {}
                }
            }

            println!("{:?}", function_locations);

            for (call_loc, function_name) in calls {
                let function_location = function_locations[&function_name] as u32;

                bytes[call_loc] = ((function_location & 0xFF000000) >> 24) as u8;
                bytes[call_loc + 1] = ((function_location & 0x00FF0000) >> 16) as u8;
                bytes[call_loc + 2] = ((function_location & 0x0000FF00) >> 8) as u8;
                bytes[call_loc + 3] = (function_location & 0x000000FF) as u8;
            }

            file.write(&bytes)?;
        }

        let mut file = if std::path::Path::new("comp/graph.json").exists() {
            OpenOptions::new()
                .write(true)
                .open("comp/graph.json")
                .unwrap()
        } else {
            std::fs::File::create("comp/graph.json")?
        };

        file.write(
            serde_json::to_string(&self.graph)
                .expect("could not convert to json")
                .as_bytes(),
        )?;

        Ok(())
    }

    fn generate_function_bytecode(
        bytes: &mut Vec<u8>,
        fn_id: String,
        functions: &Vec<(String, String, Vec<(String, String)>)>,
        var_set: &HashSet<(String, String)>,
        variable_addresses: &HashMap<String, (String, u32)>,
        calls: &mut Vec<(usize, String)>,
        ast: AbstractSyntaxTree,
    ) {
        let children = ast.children.clone();
        match ast.node {
            SyntaxTreeNode::DeclareConst => {
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[2].clone(),
                );

                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                let (t, addr) = variable_addresses[&id].clone();

                if t == "int" {
                    bytes.push(0x24);

                    bytes.push(((addr & 0xFF000000) >> 24) as u8);
                    bytes.push(((addr & 0x00FF0000) >> 16) as u8);
                    bytes.push(((addr & 0x0000FF00) >> 8) as u8);
                    bytes.push((addr & 0x000000FF) as u8);
                } else if t == "float" {
                    bytes.push(0x25);

                    bytes.push(((addr & 0xFF000000) >> 24) as u8);
                    bytes.push(((addr & 0x00FF0000) >> 16) as u8);
                    bytes.push(((addr & 0x0000FF00) >> 8) as u8);
                    bytes.push((addr & 0x000000FF) as u8);
                }
            }
            SyntaxTreeNode::DeclareVar => {
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[2].clone(),
                );

                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                let (t, addr) = variable_addresses[&id].clone();

                if t == "int" {
                    bytes.push(0x24);

                    bytes.push(((addr & 0xFF000000) >> 24) as u8);
                    bytes.push(((addr & 0x00FF0000) >> 16) as u8);
                    bytes.push(((addr & 0x0000FF00) >> 8) as u8);
                    bytes.push((addr & 0x000000FF) as u8);
                } else if t == "float" {
                    bytes.push(0x25);

                    bytes.push(((addr & 0xFF000000) >> 24) as u8);
                    bytes.push(((addr & 0x00FF0000) >> 16) as u8);
                    bytes.push(((addr & 0x0000FF00) >> 8) as u8);
                    bytes.push((addr & 0x000000FF) as u8);
                }
            }
            SyntaxTreeNode::Assign => {
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );

                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                let (t, addr) = variable_addresses[&id].clone();

                if t == "int" {
                    bytes.push(0x24);

                    bytes.push(((addr & 0xFF000000) >> 24) as u8);
                    bytes.push(((addr & 0x00FF0000) >> 16) as u8);
                    bytes.push(((addr & 0x0000FF00) >> 8) as u8);
                    bytes.push((addr & 0x000000FF) as u8);
                } else if t == "float" {
                    bytes.push(0x25);

                    bytes.push(((addr & 0xFF000000) >> 24) as u8);
                    bytes.push(((addr & 0x00FF0000) >> 16) as u8);
                    bytes.push(((addr & 0x0000FF00) >> 8) as u8);
                    bytes.push((addr & 0x000000FF) as u8);
                }
            }
            SyntaxTreeNode::FnCall => {
                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                bytes.push(0x10);
                bytes.push(0x0);
                bytes.push(0x0);
                bytes.push(0x0);
                bytes.push(0x0);

                let ret_loc = bytes.len() - 4;

                Self::generate_inputs_bytecode(
                    bytes,
                    fn_id,
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );

                bytes.push(0x5A);
                bytes.push(0x00);
                bytes.push(0x00);
                bytes.push(0x00);
                bytes.push(0x00);

                calls.push((bytes.len() - 4, id.clone()));

                let ret_addr = bytes.len() as u32;
                bytes[ret_loc] = ((ret_addr & 0xFF000000) >> 24) as u8;
                bytes[ret_loc + 1] = ((ret_addr & 0x00FF0000) >> 16) as u8;
                bytes[ret_loc + 2] = ((ret_addr & 0x0000FF00) >> 8) as u8;
                bytes[ret_loc + 3] = (ret_addr & 0x000000FF) as u8;
            }
            SyntaxTreeNode::WhileLoop => {
                let return_to = bytes.len() as u32;

                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[0].clone(),
                );

                bytes.push(0x51);
                bytes.push(0x00);
                bytes.push(0x00);
                bytes.push(0x00);
                bytes.push(0x00);

                let jump_loc = bytes.len() - 4;

                Self::generate_function_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );

                bytes.push(0x5A);

                println!("{return_to}");

                bytes.push(((return_to & 0xFF000000) >> 24) as u8);
                bytes.push(((return_to & 0x00FF0000) >> 16) as u8);
                bytes.push(((return_to & 0x0000FF00) >> 8) as u8);
                bytes.push((return_to & 0x000000FF) as u8);

                println!(
                    "{} {} {} {}",
                    bytes[bytes.len() - 4],
                    bytes[bytes.len() - 3],
                    bytes[bytes.len() - 2],
                    bytes[bytes.len() - 1]
                );

                let jump_addr = bytes.len() as u32;
                bytes[jump_loc] = ((jump_addr & 0xFF000000) >> 24) as u8;
                bytes[jump_loc + 1] = ((jump_addr & 0x00FF0000) >> 16) as u8;
                bytes[jump_loc + 2] = ((jump_addr & 0x0000FF00) >> 8) as u8;
                bytes[jump_loc + 3] = (jump_addr & 0x000000FF) as u8;
            }
            SyntaxTreeNode::IfStmt => {
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[0].clone(),
                );

                bytes.push(0x51);
                bytes.push(0x00);
                bytes.push(0x00);
                bytes.push(0x00);
                bytes.push(0x00);

                let jump_loc = bytes.len() - 4;

                Self::generate_function_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );

                bytes.push(0x5A);
                bytes.push(0x00);
                bytes.push(0x00);
                bytes.push(0x00);
                bytes.push(0x00);

                let jump_addr = bytes.len() as u32;
                bytes[jump_loc] = ((jump_addr & 0xFF000000) >> 24) as u8;
                bytes[jump_loc + 1] = ((jump_addr & 0x00FF0000) >> 16) as u8;
                bytes[jump_loc + 2] = ((jump_addr & 0x0000FF00) >> 8) as u8;
                bytes[jump_loc + 3] = (jump_addr & 0x000000FF) as u8;

                let jump_loc = bytes.len() - 4;

                Self::generate_function_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[2].clone(),
                );

                let jump_addr = bytes.len() as u32;
                bytes[jump_loc] = ((jump_addr & 0xFF000000) >> 24) as u8;
                bytes[jump_loc + 1] = ((jump_addr & 0x00FF0000) >> 16) as u8;
                bytes[jump_loc + 2] = ((jump_addr & 0x0000FF00) >> 8) as u8;
                bytes[jump_loc + 3] = (jump_addr & 0x000000FF) as u8;
            }
            SyntaxTreeNode::ReturnValue => {
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id,
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[0].clone(),
                );
                bytes.push(0x5B);
            }
            _ => {
                for child in children {
                    Self::generate_function_bytecode(
                        bytes,
                        fn_id.clone(),
                        functions,
                        var_set,
                        variable_addresses,
                        calls,
                        child,
                    );
                }
            }
        }
    }

    fn generate_expr_bytecode(
        bytes: &mut Vec<u8>,
        fn_id: String,
        functions: &Vec<(String, String, Vec<(String, String)>)>,
        var_set: &HashSet<(String, String)>,
        variable_addresses: &HashMap<String, (String, u32)>,
        calls: &mut Vec<(usize, String)>,
        ast: AbstractSyntaxTree,
    ) {
        let children = ast.children.clone();
        match ast.node {
            SyntaxTreeNode::FnCall => {
                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                bytes.push(0x10);
                bytes.push(0x0);
                bytes.push(0x0);
                bytes.push(0x0);
                bytes.push(0x0);

                let ret_loc = bytes.len() - 4;

                Self::generate_inputs_bytecode(
                    bytes,
                    fn_id,
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );

                bytes.push(0x5A);
                bytes.push(0x00);
                bytes.push(0x00);
                bytes.push(0x00);
                bytes.push(0x00);

                calls.push((bytes.len() - 4, id.clone()));

                let ret_addr = bytes.len() as u32;
                bytes[ret_loc] = ((ret_addr & 0xFF000000) >> 24) as u8;
                bytes[ret_loc + 1] = ((ret_addr & 0x00FF0000) >> 16) as u8;
                bytes[ret_loc + 2] = ((ret_addr & 0x0000FF00) >> 8) as u8;
                bytes[ret_loc + 3] = (ret_addr & 0x000000FF) as u8;
            }
            SyntaxTreeNode::AndOp => {
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[0].clone(),
                );
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );
                bytes.push(0x58);
            }
            SyntaxTreeNode::OrOp => {
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[0].clone(),
                );
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );
                bytes.push(0x59);
            }
            SyntaxTreeNode::CompEq => {
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[0].clone(),
                );
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );
                bytes.push(0x52);
            }
            SyntaxTreeNode::CompNeq => {
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[0].clone(),
                );
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );
                bytes.push(0x53);
            }
            SyntaxTreeNode::CompLess => {
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[0].clone(),
                );
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );
                bytes.push(0x54);
            }
            SyntaxTreeNode::CompGreater => {
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[0].clone(),
                );
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );
                bytes.push(0x56);
            }
            SyntaxTreeNode::CompLeq => {
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[0].clone(),
                );
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );
                bytes.push(0x55);
            }
            SyntaxTreeNode::CompGeq => {
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[0].clone(),
                );
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );
                bytes.push(0x57);
            }
            SyntaxTreeNode::AddOp => {
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[0].clone(),
                );
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );

                let t = Self::get_type(functions.clone(), var_set.clone(), ast.clone())
                    .expect("could not get type");

                if t == "int" {
                    bytes.push(0x30);
                } else if t == "float" {
                    bytes.push(0x31);
                }
            }
            SyntaxTreeNode::SubOp => {
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[0].clone(),
                );
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );

                let t = Self::get_type(functions.clone(), var_set.clone(), ast.clone())
                    .expect("could not get type");

                if t == "int" {
                    bytes.push(0x32);
                } else if t == "float" {
                    bytes.push(0x33);
                }
            }
            SyntaxTreeNode::MulOp => {
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[0].clone(),
                );
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );

                let t = Self::get_type(functions.clone(), var_set.clone(), ast.clone())
                    .expect("could not get type");

                if t == "int" {
                    bytes.push(0x34);
                } else if t == "float" {
                    bytes.push(0x35);
                }
            }
            SyntaxTreeNode::DivOp => {
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[0].clone(),
                );
                Self::generate_expr_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );

                let t = Self::get_type(functions.clone(), var_set.clone(), ast.clone())
                    .expect("could not get type");

                if t == "int" {
                    bytes.push(0x36);
                } else if t == "float" {
                    bytes.push(0x37);
                }
            }
            SyntaxTreeNode::Integer(num) => {
                bytes.push(0x10);

                let b1 = (0xFF & (num as u32)) as u8;
                let b2 = (0xFF & ((num >> 8) as u32)) as u8;
                let b3 = (0xFF & ((num >> 16) as u32)) as u8;
                let b4 = (0xFF & ((num >> 24) as u32)) as u8;

                bytes.push(b4);
                bytes.push(b3);
                bytes.push(b2);
                bytes.push(b1);
            }
            SyntaxTreeNode::Float(num) => {
                bytes.push(0x11);

                let b1 = (0xFF & (num as u32)) as u8;
                let b2 = (0xFF & (num as u32 >> 8)) as u8;
                let b3 = (0xFF & (num as u32 >> 16)) as u8;
                let b4 = (0xFF & (num as u32 >> 24)) as u8;

                bytes.push(b4);
                bytes.push(b3);
                bytes.push(b2);
                bytes.push(b1);
            }
            SyntaxTreeNode::Identifier(id) => {
                let (t, addr) = variable_addresses[&id].clone();

                if t == "int" {
                    bytes.push(0x22);

                    bytes.push(((addr & 0xFF000000) >> 24) as u8);
                    bytes.push(((addr & 0x00FF0000) >> 16) as u8);
                    bytes.push(((addr & 0x0000FF00) >> 8) as u8);
                    bytes.push((addr & 0x000000FF) as u8);
                } else if t == "float" {
                    bytes.push(0x23);

                    bytes.push(((addr & 0xFF000000) >> 24) as u8);
                    bytes.push(((addr & 0x00FF0000) >> 16) as u8);
                    bytes.push(((addr & 0x0000FF00) >> 8) as u8);
                    bytes.push((addr & 0x000000FF) as u8);
                }
            }
            _ => {
                for child in children {
                    Self::generate_expr_bytecode(
                        bytes,
                        fn_id.clone(),
                        functions,
                        var_set,
                        variable_addresses,
                        calls,
                        child,
                    );
                }
            }
        }
    }

    fn generate_inputs_bytecode(
        bytes: &mut Vec<u8>,
        fn_id: String,
        functions: &Vec<(String, String, Vec<(String, String)>)>,
        var_set: &HashSet<(String, String)>,
        variable_addresses: &HashMap<String, (String, u32)>,
        calls: &mut Vec<(usize, String)>,
        ast: AbstractSyntaxTree,
    ) {
        let children = ast.children.clone();
        match ast.node {
            SyntaxTreeNode::InputList => {
                Self::generate_inputs_bytecode(
                    bytes,
                    fn_id.clone(),
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );
                Self::generate_inputs_bytecode(
                    bytes,
                    fn_id,
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[0].clone(),
                );
            }
            SyntaxTreeNode::Identifier(id) => {
                let (t, addr) = variable_addresses[&id].clone();

                if t == "int" {
                    bytes.push(0x22);

                    bytes.push(((addr & 0xFF000000) >> 24) as u8);
                    bytes.push(((addr & 0x00FF0000) >> 16) as u8);
                    bytes.push(((addr & 0x0000FF00) >> 8) as u8);
                    bytes.push((addr & 0x000000FF) as u8);
                } else if t == "float" {
                    bytes.push(0x23);

                    bytes.push(((addr & 0xFF000000) >> 24) as u8);
                    bytes.push(((addr & 0x00FF0000) >> 16) as u8);
                    bytes.push(((addr & 0x0000FF00) >> 8) as u8);
                    bytes.push((addr & 0x000000FF) as u8);
                }
            }
            SyntaxTreeNode::Integer(num) => {
                bytes.push(0x10);

                let b1 = (0xFF & (num as u32)) as u8;
                let b2 = (0xFF & ((num >> 8) as u32)) as u8;
                let b3 = (0xFF & ((num >> 16) as u32)) as u8;
                let b4 = (0xFF & ((num >> 24) as u32)) as u8;

                bytes.push(b4);
                bytes.push(b3);
                bytes.push(b2);
                bytes.push(b1);
            }
            SyntaxTreeNode::Float(num) => {
                bytes.push(0x11);

                let b1 = (0xFF & (num as u32)) as u8;
                let b2 = (0xFF & (num as u32 >> 8)) as u8;
                let b3 = (0xFF & (num as u32 >> 16)) as u8;
                let b4 = (0xFF & (num as u32 >> 24)) as u8;

                bytes.push(b4);
                bytes.push(b3);
                bytes.push(b2);
                bytes.push(b1);
            }
            _ => {
                for child in children {
                    Self::generate_inputs_bytecode(
                        bytes,
                        fn_id.clone(),
                        functions,
                        var_set,
                        variable_addresses,
                        calls,
                        child,
                    );
                }
            }
        }
    }
}
