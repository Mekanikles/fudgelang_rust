use regex::Regex;

use std::str;
use std::fmt;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(Debug)]
pub enum BuiltInFunction {
    PrintFormat,
}

#[derive(Debug)]
pub enum BuiltInPrimitiveType {
    U32,
}

#[derive(Debug)]
pub enum BuiltInObject {
    Function(BuiltInFunction),
    PrimitiveType(BuiltInPrimitiveType)
}

#[derive(Copy, Clone)]
pub struct NodeRef {
    index: u32,
}

#[derive(Copy, Clone)]
pub struct SymbolRef {
    key: u64,
}

impl<'a> fmt::Debug for NodeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return f.write_fmt(format_args!("Node {}", self.index))
    }
}

impl<'a> fmt::Debug for SymbolRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return f.write_fmt(format_args!("<Symbol {}>", self.key))
    }
}

pub struct Ast {
    nodes: Vec<Node>,
    root_index: Option<u32>,

    symbols: HashMap<u64, String>,
}

impl Ast {
    pub fn new() -> Self {
        Ast {
            nodes: Vec::new(),
            root_index: None,
            symbols: HashMap::new(),
        }
    }

    pub fn reserve_node(&mut self) -> NodeRef {
        self.nodes.push(Node::Invalid);
        return NodeRef {
            index: (self.nodes.len() - 1) as u32,
        };
    }

    pub fn undo_node_reservation(&mut self, noderef: NodeRef) {
        assert_eq!(self.nodes.len() - 1, noderef.index as usize);
        self.nodes.pop();
    }

    pub fn replace_node(&mut self, noderef: NodeRef, node: Node) -> NodeRef {
        *self.get_node_mut(noderef.index) = node;
        return noderef;
    }

    pub fn add_node(&mut self, node: Node) -> NodeRef {
        self.nodes.push(node);
        return NodeRef {
            index: (self.nodes.len() - 1) as u32,
        };
    }

    pub fn get_node_mut<'a>(&'a mut self, index: u32) -> &'a mut Node {
        return &mut self.nodes[index as usize];
    }

    pub fn get_node<'a>(&'a self, index: u32) -> &'a Node {
        return & self.nodes[index as usize];
    }

    pub fn set_root(&mut self, n: NodeRef) {
        self.root_index = Some(n.index);
    }

    pub fn get_root(&self) -> Option<NodeRef> {
        if let Some(i) = self.root_index {
            return Some(NodeRef {
                index: i,
            });
        }
        return None;
    }

    pub fn add_symbol(&mut self, symbol: &str) -> SymbolRef {
        let mut hasher = DefaultHasher::new();
        symbol.hash(&mut hasher);
        let key = hasher.finish();

        if !self.symbols.contains_key(&key) {
            self.symbols.insert(key, symbol.into());
        }

        return SymbolRef { key };
    }
}

#[derive(Debug)]
pub enum Node {
    Invalid,
    StatementBody {
        statements: Vec<NodeRef>
    },
    IntegerLiteral {
        value: u64,
        signed: bool,
    },
    // TODO: BigIntegerLiteral
    StringLiteral {
        text: String,
    },
    FunctionLiteral {
        inputparams: Vec<NodeRef>,
        outputparams: Vec<NodeRef>,
        body: NodeRef,
    },
    InputParameter {
        symbol: SymbolRef,
        typeexpr: NodeRef,
    },
    OutputParameter {
        typeexpr: NodeRef,
    },
    BuiltInObjectReference { // NOTE: This shortcuts having to evaluate built-ins as ordinary expressions
        object: BuiltInObject,
    },
    SymbolReference {
        symbol: SymbolRef,
    },
    ReturnStatement {
        expr: NodeRef,
    },
    ArgumentList {
        args: Vec<NodeRef>,
    },
    // TODO: Can this be generalized to parameterized symbol reference?
    //  The same syntax is used for function calls, type parameteters etc
    CallOperation {
        expr: NodeRef,
        arglist: NodeRef,
    },
    BinaryOperation {
        // TODO: optype
        lhs: NodeRef,
        rhs: NodeRef,
    },
    SymbolDeclaration {
        symbol: SymbolRef,
        typeexpr: Option<NodeRef>,
        initexpr: Option<NodeRef>,
    }
}

struct AstPrinter<'a> {
    ast: &'a Ast,
    left_padding: u32,
    level: u32,
}

impl Ast {
    pub fn print(&self, left_padding: u32) {
        if let Some(r) = self.get_root() {
            AstPrinter {
                ast: self,
                left_padding,
                level: 0,
            }.node_print(&r);
        }
    }
}

impl<'a> AstPrinter<'a> {
    fn print_optional_child(&mut self, node: &Option<NodeRef>) {
        if let Some(n) = node {
            self.print_child(n);
        }
    }

    fn print_child(&mut self, node: &NodeRef) {
        self.level += 1;
        self.node_print(node);
        self.level -= 1;
    }

    fn node_print(&mut self, noderef : &NodeRef) {
        let node = self.ast.get_node(noderef.index);
        let mut nodetext = format!("{:?}", node);

        // Make symbol references human readable
        {
            let re = Regex::new(r"<Symbol (\d+)>").unwrap();
            let mut match_slices = Vec::new(); 
            for cap in re.captures_iter(&nodetext) {
                if let Some(m1) = cap.get(0) {
                    if let Some(m2) = cap.get(1) {
                        match_slices.push((m1.start()..m1.end(), m2.start()..m2.end()));
                    }
                }
            }
            for m in match_slices {
                let mut buf = nodetext.into_bytes();
                let key = str::from_utf8(&buf[m.1]).unwrap().parse::<u64>();
                if let Some(s) = self.ast.symbols.get(&key.unwrap()) {
                    buf.splice(m.0, s.bytes());
                }
                
                nodetext = String::from_utf8(buf).unwrap();
            }
        }

        println!("{:indent$}{}: {:?}", "", noderef.index, nodetext, indent=(self.left_padding + self.level * 4) as usize);
        
        // Recurse into subtree
        match node {
            Node::Invalid | 
            Node::IntegerLiteral {..} |
            Node::StringLiteral {..} |
            Node::BuiltInObjectReference {..} |
            Node::SymbolReference {..} => (),
            Node::StatementBody {statements, ..} => {
                for s in statements {
                    self.print_child(s);
                }
            },
            Node::FunctionLiteral {inputparams, outputparams, body} => {
                for p in inputparams {
                    self.print_child(p);
                }
                for p in outputparams {
                    self.print_child(p);
                }
                self.print_child(body);
            },
            Node::InputParameter {typeexpr, ..} => {
                self.print_child(typeexpr);
            },
            Node::OutputParameter {typeexpr, ..} => {
                self.print_child(typeexpr);
            },
            Node::ReturnStatement {expr, ..} => {
                self.print_child(expr);
            },
            Node::ArgumentList {args, ..} => {
                for a in args {
                    self.print_child(a);
                }
            }
            Node::CallOperation {expr, arglist, ..} => {
                self.print_child(expr);
                self.print_child(arglist);
            },
            Node::BinaryOperation {lhs, rhs, ..} => {
                self.print_child(lhs);
                self.print_child(rhs);
            }
            Node::SymbolDeclaration {typeexpr, initexpr, ..} => {
                self.print_optional_child(typeexpr);
                self.print_optional_child(initexpr);
            }
        }
    }
}


