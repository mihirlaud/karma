use std::{collections::{HashMap, LinkedList}, fs::{File, create_dir}, io::Write, path::Path};

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
    pub fn new(parser: Parser) -> Self {
        let mut graph = HashMap::new();
        Self::create_node_graph(&mut graph, parser.ast.clone());

        let mut stack = LinkedList::new();

        match Self::check_semantics(&mut stack, parser.ast.clone()) {
            Ok(_) => println!("semantic check passed"),
            Err(_) => println!("SEMANTIC CHECK FAILED"),
        }

        let mut symbol_table = HashMap::new();
        println!("{:?}", symbol_table);
        Self::generate_symbol_table(&mut symbol_table, parser.ast.clone(), String::new());

        println!("{:?}", symbol_table);

        match Self::check_types(&symbol_table, parser.ast.clone(), String::new()) {
            Ok(_) => println!("type check passed"),
            Err(_) => println!("TYPE CHECK FAILED"),
        }

        match Self::check_return(&symbol_table, parser.ast.clone(), String::new()) {
            Ok(_) => println!("return type check passed"),
            Err(_) => println!("RETURN TYPE CHECK FAILED"),
        }

        Self {
            graph,
            symbol_table,
            ast: parser.ast.clone(),
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
                    if elem.clone() == ScopeElem::Variable(id.clone()) || elem.clone() == ScopeElem::Const(id.clone()){
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
                    if elem.clone() == ScopeElem::Variable(id.clone()) || elem.clone() == ScopeElem::Const(id.clone()){
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
                table.insert(SymbolTableKey::ID(format!("{scope}::{id}")), SymbolTableEntry::Const(t));

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
                table.insert(SymbolTableKey::ID(format!("{scope}::{id}")), SymbolTableEntry::Variable(t));

                Self::generate_symbol_table(table, children[2].clone(), scope);
            }
            SyntaxTreeNode::DeclareFunc => {
                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                let params = Self::get_params(children[1].clone());
                params.iter().for_each(|(p_id, p_t)| {
                    table.insert(SymbolTableKey::ID(format!("{scope}::{id}::{p_id}")), SymbolTableEntry::Variable(p_t.clone()));
                });
                let t = match children[2].clone().node {
                    SyntaxTreeNode::ReturnType => match children[2].children[0].clone().node {
                        SyntaxTreeNode::Identifier(id) => id,
                        _ => "".to_string(),
                    },
                    _ => "".to_string(),
                };
                let param_types = params.iter().map(|(_, p_t)| p_t.clone()).collect();
                table.insert(SymbolTableKey::ID(format!("{scope}::{id}")), SymbolTableEntry::Function(t, param_types));

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
        ast: AbstractSyntaxTree, scope: String
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
                        if l_value.clone() != Self::get_type(table, children[2].clone(), scope.clone())? {
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
                        if l_value.clone() != Self::get_type(table, children[2].clone(), scope.clone())? {
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
                        if l_value.clone() != Self::get_type(table, children[1].clone(), scope.clone())? {
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
        ast: AbstractSyntaxTree, scope: String,
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
                println!("{scope}::{id}");
                match table.get(&SymbolTableKey::ID(format!("{scope}::{id}"))).unwrap() {
                SymbolTableEntry::Variable(t) => Ok(t.clone()),
                SymbolTableEntry::Const(t) => Ok(t.clone()),
                _ => Ok("".to_string()),
            }},
            SyntaxTreeNode::FnCall => {
                let params = Self::get_inputs(table, ast.children[1].clone(), scope.clone());

                match ast.children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => {
                        let idx = scope.find("::").unwrap();
                        let fn_id = format!("{}::{id}", scope.get(0 .. idx).unwrap());
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
        ast: AbstractSyntaxTree, scope: String,
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
                    let id_type = match table.get(&SymbolTableKey::Float(format!("{num}"))).unwrap() {
                        SymbolTableEntry::Const(t) => t.clone(),
                        _ => "".to_string(),
                    };
                    vec![id_type]
                }
                SyntaxTreeNode::Identifier(id) => {
                    let id_type = match table.get(&SymbolTableKey::ID(format!("{scope}::{id}"))).unwrap() {
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
                        Self::get_type(table, ast.children[0].clone(), scope.clone()).expect("failed type check");
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
        ast: AbstractSyntaxTree, scope: String,
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
        id: &String, scope: String,
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

    pub fn generate_assembly(&self) -> Result<(), std::io::Error> {
        if !Path::new("comp/").exists() {
            create_dir("comp")?;
        }

        let mut file = if Path::new("/comp/out.asm").exists() {
            File::open("comp/out.asm")?
        } else {
            File::create("comp/out.asm")?
        };  

        file.write(b"global _start\n")?;

        file.write(b"section .data\n")?;

        let mut constants = HashMap::new();

        let mut idx = 0;
        for key in self.symbol_table.keys() {
            match key {
                SymbolTableKey::Int(num) | SymbolTableKey::Float(num) => {
                    file.write(format!("c{idx}: dd {num}\n").as_bytes())?;
                    constants.insert(num, idx);

                    idx += 1;
                }
                _ => {}
            }
        }

        file.write(b"section .bss\n")?;

        let mut variables = HashMap::new();

        idx = 0;
        for key in self.symbol_table.keys() {
            match key {
                SymbolTableKey::ID(num) => {
                    match self.symbol_table[key].clone() {
                        SymbolTableEntry::Variable(t) | SymbolTableEntry::Const(t) => {
                            let len = if t == "float" {
                                8
                            } else if t == "int" {
                                4
                            } else {
                                1
                            };

                            file.write(format!("v{idx}: resb {len}\n").as_bytes())?;
                            variables.insert(num, idx);

                            idx += 1;
                        }
                        _ => {}
                    }   
                }
                _ => {}
            }
        }

        file.write(b"section .text\n")?;

        Self::process(self.ast.clone(), &mut file, String::new(), &variables, &constants, 0)?;

        file.write(b"_start:\n")?;
        file.write(b"mov rax, 60\n")?;
        file.write(b"mov rdi, 0\n")?;
        file.write(b"syscall\n")?;


        Ok(())
    }

    fn process(ast: AbstractSyntaxTree, file: &mut File, scope: String, variables: &HashMap<&String, i32>, constants: &HashMap<&String, i32>, label_count: usize) -> Result<(), std::io::Error> {
        let children = ast.children.clone();

        match ast.node {
            SyntaxTreeNode::DeclareNode => {
                let header = children[0].clone();
                let id = match header.children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };
                let scope = format!("{id}");
                Self::process(children[1].clone(), file, scope, variables, constants, label_count)?;
            }
            SyntaxTreeNode::DeclareFunc => {
                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => id,
                    _ => "".to_string(),
                };

                file.write(format!("_{id}:\n").as_bytes())?;

                let scope = format!("{scope}::{id}");
                Self::process(children[3].clone(), file, scope, variables, constants, label_count)?;
            }
            SyntaxTreeNode::DeclareConst | SyntaxTreeNode::DeclareVar => {
                file.write(b"mov rax, 0\n")?;
                Self::process_expression(children[2].clone(), scope.clone(), file, variables, constants)?;

                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => variables[&format!("{scope}::{id}")],
                    _ => -1,
                };

                file.write(format!("mov [v{id}], rax\n").as_bytes())?;
            }
            SyntaxTreeNode::Assign => {
                file.write(b"mov rax, 0\n")?;
                Self::process_expression(children[1].clone(), scope.clone(), file, variables, constants)?;

                let id = match children[0].clone().node {
                    SyntaxTreeNode::Identifier(id) => variables[&format!("{scope}::{id}")],
                    _ => -1,
                };

                file.write(format!("mov [v{id}], rax\n").as_bytes())?;
            }
            SyntaxTreeNode::WhileLoop => {
                file.write(format!("l{label_count}:\n").as_bytes())?;
                Self::process(children[1].clone(), file, scope.clone(), variables, constants, label_count + 1)?;
                Self::process_bool(children[0].clone(), scope, file, variables, constants, label_count)?;
            }
            SyntaxTreeNode::IfStmt => {
                Self::process_bool(ast, scope.clone(), file, variables, constants, label_count)?;
                Self::process(children[2].clone(), file, scope.clone(), variables, constants, label_count + 1)?;
                file.write(format!("jmp e{label_count}\n").as_bytes())?;
                file.write(format!("l{label_count}:\n").as_bytes())?;
                Self::process(children[1].clone(), file, scope.clone(), variables, constants, label_count + 1)?;
                file.write(format!("e{label_count}:\n").as_bytes())?;
            }
            SyntaxTreeNode::ReturnValue => {
                file.write(b"mov rax, 0\n")?;
                Self::process_expression(children[0].clone(), scope, file, variables, constants)?;
                file.write(format!("ret\n").as_bytes())?;
            }
            _ => {
                for child in children {
                    Self::process(child, file, scope.clone(), variables, constants, label_count)?;
                }
            }
        }
        

        Ok(())
    }

    fn process_expression(ast: AbstractSyntaxTree, scope: String, file: &mut File, variables: &HashMap<&String, i32>, constants: &HashMap<&String, i32>) -> Result<(), std::io::Error> {
        let children = ast.children.clone();

        match ast.node {
            SyntaxTreeNode::Integer(id) => {
                let idx = constants[&format!("{id}")];
                file.write(format!("add rax, [c{idx}]\n").as_bytes())?;
            }
            SyntaxTreeNode::Float(id) => {
                let idx = constants[&format!("{id}")];
                file.write(format!("add rax, [c{idx}]\n").as_bytes())?;
            }
            SyntaxTreeNode::Identifier(id) => {
                let idx = variables[&format!("{scope}::{id}")];
                file.write(format!("add rax, [v{idx}]\n").as_bytes())?;
            }
            SyntaxTreeNode::AddOp => {
                Self::process_expression(children[0].clone(), scope.clone(), file, variables, constants)?;
                Self::process_expression(children[1].clone(), scope, file, variables, constants)?;
            }
            SyntaxTreeNode::SubOp => {
                Self::process_expression(children[0].clone(), scope.clone(), file, variables, constants)?;
                file.write(b"neg rax\n")?;
                Self::process_expression(children[1].clone(), scope, file, variables, constants)?;
                file.write(b"neg rax\n")?;
            }
            _ => {
            }
        }
        
        Ok(())
    }

    fn process_bool(ast: AbstractSyntaxTree, scope: String, file: &mut File, variables: &HashMap<&String, i32>, constants: &HashMap<&String, i32>, label_count: usize) -> Result<(), std::io::Error> {
        match ast.node {
            SyntaxTreeNode::AndOp => {
                todo!("implement and assembly");
            }
            SyntaxTreeNode::OrOp => {
                todo!("implement or assembly");
            }
            SyntaxTreeNode::OrOp => {}
            SyntaxTreeNode::CompEq 
            | SyntaxTreeNode::CompGeq 
            | SyntaxTreeNode::CompGreater 
            | SyntaxTreeNode::CompLeq 
            | SyntaxTreeNode::CompLess 
            | SyntaxTreeNode::CompNeq => {
                Self::process_comparison(ast, scope, file, variables, constants, label_count)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn process_comparison(ast: AbstractSyntaxTree, scope: String, file: &mut File, variables: &HashMap<&String, i32>, constants: &HashMap<&String, i32>, label_count: usize) -> Result<(), std::io::Error> {
        let children = ast.children;
        file.write(b"mov rax, 0\n")?;
        Self::process_expression(children[0].clone(), scope.clone(), file, variables, constants)?;
        file.write(b"mov r8, rax\n")?;
        file.write(b"mov rax, 0\n")?;
        Self::process_expression(children[1].clone(), scope, file, variables, constants)?;
        file.write(b"cmp r8, rax\n")?;
        
        match ast.node {
            SyntaxTreeNode::CompEq => {
                file.write(format!("jeq l{label_count}\n").as_bytes())?;
            }
            SyntaxTreeNode::CompNeq => {
                file.write(format!("jne l{label_count}\n").as_bytes())?;
            }
            SyntaxTreeNode::CompLess => {
                file.write(format!("jl l{label_count}\n").as_bytes())?;
            }
            SyntaxTreeNode::CompLeq => {
                file.write(format!("jle l{label_count}\n").as_bytes())?;
            }
            SyntaxTreeNode::CompGreater => {
                file.write(format!("jg l{label_count}\n").as_bytes())?;
            }
            SyntaxTreeNode::CompGeq => {
                file.write(format!("jge l{label_count}\n").as_bytes())?;
            }
            _ => {}
        }

        Ok(())
    }
}
