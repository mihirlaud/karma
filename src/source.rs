use std::{
    collections::{HashMap, HashSet, LinkedList},
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
                    SyntaxTreeNode::NoReturn => "!".to_string(),
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
                Self::check_semantics_helper(stack, var_set, children[2].clone())?;

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
            | SyntaxTreeNode::CompGreater
            | SyntaxTreeNode::Index => {
                Self::check_semantics_helper(stack, var_set, children[0].clone())?;
                Self::check_semantics_helper(stack, var_set, children[1].clone())?;
            }
            SyntaxTreeNode::Identifier(id) => {
                if children.len() > 0 {
                    Self::check_semantics_helper(stack, var_set, children[0].clone())?;
                }

                for elem in stack.clone() {
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

                if id == "print_int"
                    || id == "print_float"
                    || id == "print_bool"
                    || id == "print_char"
                    || id == "println"
                {
                    return Ok(());
                }

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
                let mut l_value =
                    Self::get_type(functions.clone(), var_set.clone(), children[0].clone())?;

                let arr_type =
                    Self::get_type(functions.clone(), var_set.clone(), children[1].clone())?;

                if arr_type != "int" && arr_type != "" {
                    return Err(22);
                }

                l_value = Self::get_indexed(l_value, children[1].clone())?;

                let r_value =
                    Self::get_type(functions.clone(), var_set.clone(), children[2].clone())?;

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

                if l_value == r_value
                    && (l_value == "int" || l_value == "float" || l_value == "char")
                {
                    Ok(l_value)
                } else {
                    Err(11)
                }
            }
            SyntaxTreeNode::Index => {
                let l_value =
                    Self::get_type(functions.clone(), var_set.clone(), children[0].clone())?;

                let r_value = if children[1].clone().node != SyntaxTreeNode::Null {
                    Self::get_type(functions.clone(), var_set.clone(), children[1].clone())?
                } else {
                    l_value.clone()
                };

                if l_value == r_value && l_value == "int" {
                    Ok(l_value)
                } else {
                    Err(23)
                }
            }
            SyntaxTreeNode::CompEq
            | SyntaxTreeNode::CompNeq
            | SyntaxTreeNode::CompLess
            | SyntaxTreeNode::CompGreater
            | SyntaxTreeNode::CompLeq
            | SyntaxTreeNode::CompGeq => {
                let l_value =
                    Self::get_type(functions.clone(), var_set.clone(), children[0].clone())?;
                let r_value =
                    Self::get_type(functions.clone(), var_set.clone(), children[1].clone())?;

                if l_value == r_value
                    && (l_value == "int" || l_value == "float" || l_value == "char")
                {
                    Ok("bool".to_string())
                } else {
                    Err(11)
                }
            }
            SyntaxTreeNode::AndOp | SyntaxTreeNode::OrOp => {
                let l_value =
                    Self::get_type(functions.clone(), var_set.clone(), children[0].clone())?;
                let r_value =
                    Self::get_type(functions.clone(), var_set.clone(), children[1].clone())?;

                if l_value == r_value && l_value == "bool" {
                    Ok(l_value)
                } else {
                    Err(11)
                }
            }
            SyntaxTreeNode::Identifier(id) => {
                if ast.children.len() > 0 {
                    Self::get_type(functions.clone(), var_set.clone(), children[0].clone())?;
                }

                for (var_id, var_type) in var_set {
                    if var_id == id {
                        let mut fin = var_type.clone();
                        if ast.children.len() > 0 {
                            fin = Self::get_indexed(fin.clone(), children[0].clone())?;
                        }

                        return Ok(fin);
                    }
                }

                Err(12)
            }
            SyntaxTreeNode::InputList => {
                let inputs = Self::get_inputs(functions.clone(), var_set.clone(), ast.clone())?;
                let first = inputs[0].clone();

                for ty in inputs.clone() {
                    if ty != first {
                        return Err(21);
                    }
                }

                Ok(format!("[{first}; {}]", inputs.len()))
            }
            SyntaxTreeNode::Integer(_) => Ok(String::from("int")),
            SyntaxTreeNode::Float(_) => Ok(String::from("float")),
            SyntaxTreeNode::True | SyntaxTreeNode::False => Ok(String::from("bool")),
            SyntaxTreeNode::Character(_) => Ok(String::from("char")),
            SyntaxTreeNode::FnCall => {
                let params =
                    Self::get_inputs(functions.clone(), var_set.clone(), children[1].clone())?;

                match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => {
                        for (fn_id, fn_type, fn_params) in functions.clone() {
                            let fn_params: Vec<String> =
                                fn_params.iter().map(|(_, t)| t.clone()).collect();
                            if fn_id == id && fn_params == params {
                                return Ok(fn_type.clone());
                            }
                        }

                        if id == "print_int"
                            || id == "print_float"
                            || id == "print_bool"
                            || id == "print_char"
                        {
                            return Ok("int".to_string());
                        }

                        Err(13)
                    }
                    _ => Ok("".to_string()),
                }
            }
            _ => Ok(String::new()),
        }
    }

    fn get_indexed(l_value: String, ast: AbstractSyntaxTree) -> Result<String, usize> {
        let children = ast.children.clone();

        match ast.node {
            SyntaxTreeNode::Index => {
                let indexed_l_value = Self::get_indexed(l_value.clone(), children[1].clone())?;

                match children[0].clone().node {
                    SyntaxTreeNode::Integer(i) => {
                        if i < 0 {
                            return Err(22);
                        }
                    }
                    _ => {}
                }

                let last_semicolon = indexed_l_value.rfind(";");
                if last_semicolon == None {
                    return Err(23);
                }

                Ok(indexed_l_value
                    .get(1..last_semicolon.unwrap())
                    .unwrap()
                    .to_string())
            }
            _ => Ok(l_value),
        }
    }

    fn get_inputs(
        functions: Vec<(String, String, Vec<(String, String)>)>,
        var_set: HashSet<(String, String)>,
        ast: AbstractSyntaxTree,
    ) -> Result<Vec<String>, usize> {
        if ast.node == SyntaxTreeNode::Null {
            Ok(vec![])
        } else {
            let mut ret = match ast.children[0].clone().node {
                SyntaxTreeNode::InputList => {
                    let t = Self::get_inputs(
                        functions.clone(),
                        var_set.clone(),
                        ast.children[0].clone(),
                    )?;

                    let first = t[0].clone();

                    for ty in t.clone() {
                        if ty != first {
                            return Err(21);
                        }
                    }

                    vec![format!("[{first}; {}]", t.len())]
                }
                SyntaxTreeNode::Integer(_) => vec![String::from("int")],
                SyntaxTreeNode::Float(_) => vec![String::from("float")],
                SyntaxTreeNode::Character(_) => vec![String::from("char")],
                SyntaxTreeNode::True | SyntaxTreeNode::False => vec![String::from("bool")],
                SyntaxTreeNode::Identifier(id) => {
                    let mut fin = vec![];
                    for (var_id, var_type) in var_set.clone() {
                        if var_id == id {
                            fin = vec![var_type];
                            break;
                        }
                    }

                    fin
                }
                SyntaxTreeNode::AddOp
                | SyntaxTreeNode::SubOp
                | SyntaxTreeNode::MulOp
                | SyntaxTreeNode::DivOp => {
                    let t = Self::get_type(
                        functions.clone(),
                        var_set.clone(),
                        ast.children[0].clone(),
                    )?;
                    vec![t]
                }
                _ => vec![],
            };

            let mut rest =
                Self::get_inputs(functions.clone(), var_set.clone(), ast.children[1].clone())?;
            ret.append(&mut rest);

            Ok(ret)
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
        } else if ret == "!" {
            Self::check_return_func_3(ast)?;
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

    fn check_return_func_3(ast: AbstractSyntaxTree) -> Result<(), usize> {
        let children = ast.children.clone();
        match ast.node {
            SyntaxTreeNode::WhileLoop => Ok(()),
            SyntaxTreeNode::ReturnValue => Err(18),
            _ => {
                for child in children {
                    match Self::check_return_func_1(child) {
                        Ok(_) => {
                            return Ok(());
                        }
                        _ => {}
                    }
                }

                Err(19)
            }
        }
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
                if t == ret_type {
                    Ok(())
                } else {
                    Err(15)
                }
            }
            _ => {
                for child in children {
                    match Self::check_return_func_2(
                        functions.clone(),
                        var_set.clone(),
                        child,
                        ret_type.clone(),
                    ) {
                        Ok(()) => {
                            return Ok(());
                        }
                        _ => {}
                    }
                }
                Err(20)
            }
        }
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
                std::fs::remove_file(filename.clone())?;

                std::fs::File::create(filename.clone())?
            } else {
                std::fs::File::create(filename.clone())?
            };

            let mut bytes: Vec<u8> = vec![];
            let mut function_locations: HashMap<String, usize> = HashMap::new();
            let mut variable_addresses: HashMap<String, (String, u32)> = HashMap::new();
            let mut calls = vec![];
            let mut addr: u32 = 0x0;

            match self.symbol_table[node_id]["main"].clone() {
                TLElement::Function(ret_type, params, var_set, tree) => {
                    function_locations.insert("main".to_string(), bytes.len());

                    for (var_id, var_type) in var_set.clone() {
                        variable_addresses.insert(var_id, (var_type.clone(), addr));
                        bytes.push(match var_type.as_str() {
                            "int" => 0x20,
                            "float" => 0x21,
                            "bool" => 0x28,
                            "char" => 0x2C,
                            _ => {
                                if var_type.get(0..1).unwrap() == "[" {
                                    0x80
                                } else {
                                    0x0
                                }
                            }
                        });

                        let b = addr.to_be_bytes();
                        bytes.extend_from_slice(&b);

                        if var_type.get(0..1).unwrap() == "[" {
                            let mut last_semicolon = var_type.rfind(";");
                            let mut s = var_type.clone();

                            let mut len = 1;
                            while last_semicolon != None {
                                let i = last_semicolon.unwrap();
                                let str_len = s.get(i + 2..s.len() - 1).unwrap();
                                len = len * str_len.parse::<i32>().expect("could not parse to int");

                                s = s.get(1..i).unwrap().to_string();
                                last_semicolon = s.rfind(";");
                            }

                            match s.as_str() {
                                "int" | "float" => bytes.push(0x4),
                                "bool" | "char" => bytes.push(0x1),
                                _ => {}
                            }

                            bytes.extend_from_slice(&len.to_be_bytes());

                            addr += match s.as_str() {
                                "int" | "float" => 4 * len as u32,
                                "bool" | "char" => 1 * len as u32,
                                _ => 0,
                            };
                        }

                        addr += match var_type.as_str() {
                            "int" | "float" => 4,
                            "bool" | "char" => 1,
                            _ => 0,
                        };
                    }

                    for (param_id, param_type) in params.clone() {
                        let addr = variable_addresses[&param_id].1;
                        bytes.push(match param_type.as_str() {
                            "int" => 0x24,
                            "float" => 0x25,
                            "bool" => 0x2A,
                            "char" => 0x2E,
                            _ => {
                                if param_type.get(0..1).unwrap() == "[" {
                                    0x81
                                } else {
                                    0x0
                                }
                            }
                        });

                        let b = addr.to_be_bytes();
                        bytes.extend_from_slice(&b);
                    }
                    Self::generate_function_bytecode(
                        &mut bytes,
                        &functions,
                        &var_set,
                        &variable_addresses,
                        &mut calls,
                        tree,
                    );

                    if ret_type == "" {
                        bytes.push(0x64);
                    }
                }
                _ => {}
            }

            function_locations.insert("print_int".to_string(), bytes.len());
            bytes.extend_from_slice(&[0x90, 0x64]);

            function_locations.insert("print_float".to_string(), bytes.len());
            bytes.extend_from_slice(&[0x91, 0x64]);

            function_locations.insert("print_bool".to_string(), bytes.len());
            bytes.extend_from_slice(&[0x92, 0x64]);

            function_locations.insert("print_char".to_string(), bytes.len());
            bytes.extend_from_slice(&[0x93, 0x64]);

            function_locations.insert("println".to_string(), bytes.len());
            bytes.extend_from_slice(&[0x15, 0xA, 0x93, 0x64]);

            for fn_id in self.symbol_table[node_id].keys() {
                if fn_id == "main" {
                    continue;
                }
                match self.symbol_table[node_id][fn_id].clone() {
                    TLElement::Function(ret_type, params, var_set, tree) => {
                        function_locations.insert(fn_id.clone(), bytes.len());

                        for (var_id, var_type) in var_set.clone() {
                            variable_addresses.insert(var_id, (var_type.clone(), addr));
                            bytes.push(match var_type.as_str() {
                                "int" => 0x20,
                                "float" => 0x21,
                                "bool" => 0x28,
                                "char" => 0x2C,
                                _ => {
                                    if var_type.get(0..1).unwrap() == "[" {
                                        0x80
                                    } else {
                                        0x0
                                    }
                                }
                            });

                            let b = addr.to_be_bytes();
                            bytes.extend_from_slice(&b);

                            if var_type.get(0..1).unwrap() == "[" {
                                let last_semicolon = var_type.rfind(";").unwrap();
                                let len = var_type
                                    .get(last_semicolon + 2..var_type.len() - 1)
                                    .unwrap();
                                let len = len.parse::<i32>().expect("could not parse to int");

                                let arr_type = var_type.get(1..last_semicolon).unwrap();
                                match arr_type {
                                    "int" | "float" => bytes.push(0x4),
                                    "bool" | "char" => bytes.push(0x1),
                                    _ => {}
                                }

                                bytes.extend_from_slice(&len.to_be_bytes());

                                addr += match arr_type {
                                    "int" | "float" => 4 * len as u32,
                                    "bool" | "char" => 1 * len as u32,
                                    _ => 0,
                                };
                            }

                            addr += match var_type.as_str() {
                                "int" | "float" => 4,
                                "bool" | "char" => 1,
                                _ => 0,
                            };
                        }

                        for (param_id, param_type) in params.clone() {
                            let addr = variable_addresses[&param_id].1;
                            bytes.push(match param_type.as_str() {
                                "int" => 0x24,
                                "float" => 0x25,
                                "bool" => 0x2A,
                                "char" => 0x2E,
                                _ => {
                                    if param_type.get(0..1).unwrap() == "[" {
                                        0x81
                                    } else {
                                        0x0
                                    }
                                }
                            });
                            let b = addr.to_be_bytes();
                            bytes.extend_from_slice(&b);
                        }
                        Self::generate_function_bytecode(
                            &mut bytes,
                            &functions,
                            &var_set,
                            &variable_addresses,
                            &mut calls,
                            tree,
                        );

                        if ret_type == "" {
                            bytes.push(0x64);
                        }
                    }
                    _ => {}
                }
            }

            for (call_loc, function_name) in calls {
                let function_location = function_locations[&function_name] as u32;

                let b = function_location.to_be_bytes();

                for (i, byte) in b.iter().enumerate() {
                    bytes[call_loc + i] = *byte;
                }
            }

            file.write(&bytes)?;
        }

        let mut file = if std::path::Path::new("comp/graph.json").exists() {
            std::fs::remove_file("comp/graph.json")?;

            std::fs::File::create("comp/graph.json")?
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
        functions: &Vec<(String, String, Vec<(String, String)>)>,
        var_set: &HashSet<(String, String)>,
        variable_addresses: &HashMap<String, (String, u32)>,
        calls: &mut Vec<(usize, String)>,
        ast: AbstractSyntaxTree,
    ) {
        let children = ast.children.clone();
        match ast.node {
            SyntaxTreeNode::DeclareConst | SyntaxTreeNode::DeclareVar => {
                Self::generate_expr_bytecode(
                    bytes,
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

                let mut slice = vec![];
                bytes.extend_from_slice(match t.as_str() {
                    "int" => &[0x24],
                    "float" => &[0x25],
                    "bool" => &[0x2A],
                    "char" => &[0x2E],
                    _ => {
                        if t.get(0..1).unwrap() == "[" {
                            let mut last_semicolon = t.rfind(";");
                            let mut s = t.clone();

                            let mut len = 1;
                            while last_semicolon != None {
                                let i = last_semicolon.unwrap();
                                let str_len = s.get(i + 2..s.len() - 1).unwrap();
                                len = len * str_len.parse::<i32>().expect("could not parse to int");

                                s = s.get(1..i).unwrap().to_string();
                                last_semicolon = s.rfind(";");
                            }

                            match s.as_str() {
                                "int" => {
                                    for _ in 0..len {
                                        slice.push(0x87);
                                        let b = addr.to_be_bytes();
                                        slice.extend_from_slice(&b);
                                    }

                                    &slice
                                }
                                "float" => {
                                    for _ in 0..len {
                                        slice.push(0x88);
                                        let b = addr.to_be_bytes();
                                        slice.extend_from_slice(&b);
                                    }

                                    &slice
                                }
                                "bool" => {
                                    for _ in 0..len {
                                        slice.push(0x89);
                                        let b = addr.to_be_bytes();
                                        slice.extend_from_slice(&b);
                                    }

                                    &slice
                                }
                                "char" => {
                                    for _ in 0..len {
                                        slice.push(0x8A);
                                        let b = addr.to_be_bytes();
                                        slice.extend_from_slice(&b);
                                    }

                                    &slice
                                }

                                _ => &[0x0],
                            }
                        } else {
                            &[0x0]
                        }
                    }
                });

                if t.get(0..1).unwrap() != "[" {
                    let b = addr.to_be_bytes();
                    bytes.extend_from_slice(&b);
                }
            }
            SyntaxTreeNode::Assign => {
                Self::generate_expr_bytecode(
                    bytes,
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

                if children[1].clone().node == SyntaxTreeNode::Index {
                    Self::generate_index_bytecode(
                        bytes,
                        functions,
                        var_set,
                        variable_addresses,
                        calls,
                        children[1].clone(),
                        t.clone(),
                    );
                }

                bytes.push(match t.as_str() {
                    "int" => 0x24,
                    "float" => 0x25,
                    "bool" => 0x2A,
                    "char" => 0x2E,
                    _ => {
                        if children[1].clone().node == SyntaxTreeNode::Index {
                            let mut last_semicolon = t.rfind(";");
                            let mut s = t.clone();
                            while last_semicolon != None {
                                let i = last_semicolon.unwrap();

                                s = s.get(1..i).unwrap().to_string();
                                last_semicolon = s.rfind(";");
                            }

                            match s.as_str() {
                                "int" => 0x87,
                                "float" => 0x88,
                                "bool" => 0x89,
                                "char" => 0x8A,
                                _ => 0x0,
                            }
                        } else {
                            0x0
                        }
                    }
                });

                let b = addr.to_be_bytes();
                bytes.extend_from_slice(&b);
            }
            SyntaxTreeNode::FnCall => {
                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                bytes.push(0x10);
                bytes.extend_from_slice(&[0x0, 0x0, 0x0, 0x0]);

                let ret_loc = bytes.len() - 4;

                Self::generate_inputs_bytecode(
                    bytes,
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );

                bytes.push(0x5A);
                bytes.extend_from_slice(&[0x0, 0x0, 0x0, 0x0]);

                calls.push((bytes.len() - 4, id.clone()));

                let ret_addr = bytes.len() as u32;
                let b = ret_addr.to_be_bytes();

                for (i, byte) in b.iter().enumerate() {
                    bytes[ret_loc + i] = *byte;
                }
            }
            SyntaxTreeNode::WhileLoop => {
                let return_to = bytes.len() as u32;

                Self::generate_expr_bytecode(
                    bytes,
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[0].clone(),
                );

                bytes.push(0x51);
                bytes.extend_from_slice(&[0x0, 0x0, 0x0, 0x0]);

                let jump_loc = bytes.len() - 4;

                Self::generate_function_bytecode(
                    bytes,
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );

                bytes.push(0x5A);

                let b = return_to.to_be_bytes();
                bytes.extend_from_slice(&b);

                let jump_addr = bytes.len() as u32;
                let b = jump_addr.to_be_bytes();

                for (i, byte) in b.iter().enumerate() {
                    bytes[jump_loc + i] = *byte;
                }
            }
            SyntaxTreeNode::IfStmt => {
                Self::generate_expr_bytecode(
                    bytes,
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[0].clone(),
                );

                bytes.push(0x51);
                bytes.extend_from_slice(&[0x0, 0x0, 0x0, 0x0]);

                let jump_loc = bytes.len() - 4;

                Self::generate_function_bytecode(
                    bytes,
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );

                bytes.push(0x5A);
                bytes.extend_from_slice(&[0x0, 0x0, 0x0, 0x0]);

                let jump_addr = bytes.len() as u32;
                let b = jump_addr.to_be_bytes();

                for (i, byte) in b.iter().enumerate() {
                    bytes[jump_loc + i] = *byte;
                }

                let jump_loc = bytes.len() - 4;

                Self::generate_function_bytecode(
                    bytes,
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[2].clone(),
                );

                let jump_addr = bytes.len() as u32;
                let b = jump_addr.to_be_bytes();

                for (i, byte) in b.iter().enumerate() {
                    bytes[jump_loc + i] = *byte;
                }
            }
            SyntaxTreeNode::ReturnValue => {
                Self::generate_expr_bytecode(
                    bytes,
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
        functions: &Vec<(String, String, Vec<(String, String)>)>,
        var_set: &HashSet<(String, String)>,
        variable_addresses: &HashMap<String, (String, u32)>,
        calls: &mut Vec<(usize, String)>,
        ast: AbstractSyntaxTree,
    ) {
        let children = ast.children.clone();

        match ast.node {
            SyntaxTreeNode::AndOp
            | SyntaxTreeNode::OrOp
            | SyntaxTreeNode::CompEq
            | SyntaxTreeNode::CompNeq
            | SyntaxTreeNode::CompLess
            | SyntaxTreeNode::CompLeq
            | SyntaxTreeNode::CompGreater
            | SyntaxTreeNode::CompGeq
            | SyntaxTreeNode::AddOp
            | SyntaxTreeNode::SubOp
            | SyntaxTreeNode::MulOp
            | SyntaxTreeNode::DivOp => {
                Self::generate_expr_bytecode(
                    bytes,
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[0].clone(),
                );
                Self::generate_expr_bytecode(
                    bytes,
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );
            }
            _ => {}
        }

        match ast.node {
            SyntaxTreeNode::InputList => {
                let tree = Self::build_arr_from_input_list(ast.clone());
                Self::generate_arr_bytecode(
                    bytes,
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    tree.clone(),
                    0,
                );
            }
            SyntaxTreeNode::FnCall => {
                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                bytes.push(0x10);
                bytes.extend_from_slice(&[0x0, 0x0, 0x0, 0x0]);

                let ret_loc = bytes.len() - 4;

                Self::generate_inputs_bytecode(
                    bytes,
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );

                bytes.push(0x5A);
                bytes.extend_from_slice(&[0x0, 0x0, 0x0, 0x0]);

                calls.push((bytes.len() - 4, id.clone()));

                let ret_addr = bytes.len() as u32;
                let b = ret_addr.to_be_bytes();

                for (i, byte) in b.iter().enumerate() {
                    bytes[ret_loc + i] = *byte;
                }
            }
            SyntaxTreeNode::AndOp => {
                bytes.push(0x58);
            }
            SyntaxTreeNode::OrOp => {
                bytes.push(0x59);
            }
            SyntaxTreeNode::CompEq => {
                let t = Self::get_type(functions.clone(), var_set.clone(), children[0].clone())
                    .expect("could not get type");

                bytes.push(match t.as_str() {
                    "int" => 0x52,
                    "float" => 0x5C,
                    "bool" => 0x62,
                    _ => 0x0,
                });
            }
            SyntaxTreeNode::CompNeq => {
                let t = Self::get_type(functions.clone(), var_set.clone(), children[0].clone())
                    .expect("could not get type");

                bytes.push(match t.as_str() {
                    "int" => 0x53,
                    "float" => 0x5D,
                    "bool" => 0x63,
                    _ => 0x0,
                });
            }
            SyntaxTreeNode::CompLess => {
                let t = Self::get_type(functions.clone(), var_set.clone(), children[0].clone())
                    .expect("could not get type");

                bytes.push(match t.as_str() {
                    "int" => 0x54,
                    "float" => 0x5E,
                    _ => 0x0,
                });
            }
            SyntaxTreeNode::CompGreater => {
                let t = Self::get_type(functions.clone(), var_set.clone(), children[0].clone())
                    .expect("could not get type");

                bytes.push(match t.as_str() {
                    "int" => 0x56,
                    "float" => 0x60,
                    _ => 0x0,
                });
            }
            SyntaxTreeNode::CompLeq => {
                let t = Self::get_type(functions.clone(), var_set.clone(), children[0].clone())
                    .expect("could not get type");

                bytes.push(match t.as_str() {
                    "int" => 0x55,
                    "float" => 0x5F,
                    _ => 0x0,
                });
            }
            SyntaxTreeNode::CompGeq => {
                let t = Self::get_type(functions.clone(), var_set.clone(), children[0].clone())
                    .expect("could not get type");
                bytes.push(match t.as_str() {
                    "int" => 0x57,
                    "float" => 0x61,
                    _ => 0x0,
                });
            }
            SyntaxTreeNode::AddOp => {
                let t = Self::get_type(functions.clone(), var_set.clone(), ast.clone())
                    .expect("could not get type");

                bytes.push(match t.as_str() {
                    "int" => 0x30,
                    "float" => 0x31,
                    "char" => 0x38,
                    _ => 0x0,
                });
            }
            SyntaxTreeNode::SubOp => {
                let t = Self::get_type(functions.clone(), var_set.clone(), ast.clone())
                    .expect("could not get type");
                bytes.push(match t.as_str() {
                    "int" => 0x32,
                    "float" => 0x33,
                    "char" => 0x39,
                    _ => 0x0,
                });
            }
            SyntaxTreeNode::MulOp => {
                let t = Self::get_type(functions.clone(), var_set.clone(), ast.clone())
                    .expect("could not get type");
                bytes.push(match t.as_str() {
                    "int" => 0x34,
                    "float" => 0x35,
                    _ => 0x0,
                });
            }
            SyntaxTreeNode::DivOp => {
                let t = Self::get_type(functions.clone(), var_set.clone(), ast.clone())
                    .expect("could not get type");
                bytes.push(match t.as_str() {
                    "int" => 0x36,
                    "float" => 0x37,
                    _ => 0x0,
                });
            }
            SyntaxTreeNode::Integer(num) => {
                bytes.push(0x10);

                let b = num.to_be_bytes();
                bytes.extend_from_slice(&b);
            }
            SyntaxTreeNode::Float(num) => {
                bytes.push(0x11);

                let b = num.to_be_bytes();
                bytes.extend_from_slice(&b);
            }
            SyntaxTreeNode::True => {
                bytes.extend_from_slice(&[0x14, 0x1]);
            }
            SyntaxTreeNode::False => {
                bytes.extend_from_slice(&[0x14, 0x0]);
            }
            SyntaxTreeNode::Character(c) => {
                bytes.extend_from_slice(&[0x15, c as u8]);
            }
            SyntaxTreeNode::Identifier(id) => {
                let (t, addr) = variable_addresses[&id].clone();
                if children.len() > 0 {
                    Self::generate_index_bytecode(
                        bytes,
                        functions,
                        var_set,
                        variable_addresses,
                        calls,
                        children[0].clone(),
                        t.clone(),
                    );
                }

                bytes.push(match t.as_str() {
                    "int" => 0x22,
                    "float" => 0x23,
                    "bool" => 0x29,
                    "char" => 0x2D,
                    _ => {
                        if t.get(0..1).unwrap() == "[" {
                            let mut last_semicolon = t.rfind(";");
                            let mut s = t.clone();

                            while last_semicolon != None {
                                let i = last_semicolon.unwrap();

                                s = s.get(1..i).unwrap().to_string();
                                last_semicolon = s.rfind(";");
                            }

                            match s.as_str() {
                                "int" => 0x82,
                                "float" => 0x83,
                                "bool" => 0x84,
                                "char" => 0x85,
                                _ => 0x0,
                            }
                        } else {
                            0x0
                        }
                    }
                });

                let b = addr.to_be_bytes();
                bytes.extend_from_slice(&b);
            }
            _ => {
                for child in children {
                    Self::generate_expr_bytecode(
                        bytes,
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
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                );
                Self::generate_expr_bytecode(
                    bytes,
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[0].clone(),
                );
            }
            _ => {
                for child in children {
                    Self::generate_inputs_bytecode(
                        bytes,
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

    fn generate_arr_bytecode(
        bytes: &mut Vec<u8>,
        functions: &Vec<(String, String, Vec<(String, String)>)>,
        var_set: &HashSet<(String, String)>,
        variable_addresses: &HashMap<String, (String, u32)>,
        calls: &mut Vec<(usize, String)>,
        ast: AbstractSyntaxTree,
        idx: u32,
    ) {
        let children = ast.children.clone();
        match ast.node {
            SyntaxTreeNode::InputList => {
                Self::generate_arr_bytecode(
                    bytes,
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                    idx + 1,
                );

                Self::generate_expr_bytecode(
                    bytes,
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[0].clone(),
                );

                bytes.push(0x10);
                bytes.extend_from_slice(&idx.to_be_bytes());
            }
            _ => {}
        }
    }

    fn generate_index_bytecode(
        bytes: &mut Vec<u8>,
        functions: &Vec<(String, String, Vec<(String, String)>)>,
        var_set: &HashSet<(String, String)>,
        variable_addresses: &HashMap<String, (String, u32)>,
        calls: &mut Vec<(usize, String)>,
        ast: AbstractSyntaxTree,
        t: String,
    ) {
        let children = ast.children.clone();
        match ast.node {
            SyntaxTreeNode::Index => {
                Self::generate_expr_bytecode(
                    bytes,
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[0].clone(),
                );

                let mut last_semicolon = t.rfind(";");
                let mut s = t.clone();

                let mut len = 1;
                while last_semicolon != None {
                    let i = last_semicolon.unwrap();
                    let str_len = s.get(i + 2..s.len() - 1).unwrap();
                    len = len * str_len.parse::<i32>().expect("could not parse to int");

                    let new_t = s.get(1..i).unwrap().to_string();
                    last_semicolon = new_t.rfind(";");

                    if last_semicolon == None {
                        len /= str_len.parse::<i32>().expect("could not parse to int");
                        break;
                    }

                    s = new_t;
                }

                bytes.push(0x10);
                bytes.extend_from_slice(&len.to_be_bytes());
                bytes.push(0x34);

                Self::generate_index_bytecode(
                    bytes,
                    functions,
                    var_set,
                    variable_addresses,
                    calls,
                    children[1].clone(),
                    s.clone(),
                );

                if children[1].clone().node != SyntaxTreeNode::Null {
                    bytes.push(0x30);
                }
            }
            _ => {}
        }
    }

    fn build_arr_from_input_list(ast: AbstractSyntaxTree) -> AbstractSyntaxTree {
        if ast.node != SyntaxTreeNode::InputList {
            return ast;
        }

        let left = Self::build_arr_from_input_list(ast.children[0].clone());
        let right = Self::build_arr_from_input_list(ast.children[1].clone());

        let tree = Self::build_arr_helper(left, right);

        tree
    }

    fn build_arr_helper(left: AbstractSyntaxTree, right: AbstractSyntaxTree) -> AbstractSyntaxTree {
        match left.node {
            SyntaxTreeNode::Null => right,
            SyntaxTreeNode::InputList => {
                let mut tree = AbstractSyntaxTree::new();

                tree.node = SyntaxTreeNode::InputList;
                tree.children = vec![
                    left.children[0].clone(),
                    Self::build_arr_helper(left.children[1].clone(), right.clone()),
                ];

                tree
            }
            _ => {
                let mut tree = AbstractSyntaxTree::new();

                tree.node = SyntaxTreeNode::InputList;
                tree.children = vec![left.clone(), right.clone()];

                tree
            }
        }
    }
}
