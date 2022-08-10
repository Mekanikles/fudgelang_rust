use enum_dispatch::enum_dispatch;
use regex::Regex;

use std::fmt;
use std::str;

use crate::parser::stringstore::*;
use crate::typesystem::*;

use StringRef as SymbolRef;

#[derive(Debug)]
pub enum BuiltInObject {
    Function(BuiltInFunction),
    PrimitiveType(PrimitiveType),
}

#[derive(Copy, Clone)]
pub struct NodeRef {
    index: u32,
}

impl<'a> fmt::Debug for NodeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return f.write_fmt(format_args!("Node {}", self.index));
    }
}

pub struct Ast {
    nodes: Vec<Node>,
    root_index: Option<u32>,
    symbols: StringStore,
}

impl Ast {
    pub fn new() -> Self {
        Ast {
            nodes: Vec::new(),
            root_index: None,
            symbols: StringStore::new(),
        }
    }

    pub fn reserve_node(&mut self) -> NodeRef {
        self.nodes.push(Invalid {}.into());
        return NodeRef {
            index: (self.nodes.len() - 1) as u32,
        };
    }

    pub fn undo_node_reservation(&mut self, noderef: NodeRef) {
        assert_eq!(self.nodes.len() - 1, noderef.index as usize);
        self.nodes.pop();
    }

    pub fn replace_node(&mut self, noderef: NodeRef, node: Node) -> NodeRef {
        *self.get_node_mut(&noderef) = node;
        return noderef;
    }

    pub fn add_node(&mut self, node: Node) -> NodeRef {
        self.nodes.push(node);
        return NodeRef {
            index: (self.nodes.len() - 1) as u32,
        };
    }

    pub fn get_node_mut<'a>(&'a mut self, noderef: &NodeRef) -> &'a mut Node {
        return &mut self.nodes[noderef.index as usize];
    }

    pub fn get_node<'a>(&'a self, noderef: &NodeRef) -> &'a Node {
        return &self.nodes[noderef.index as usize];
    }

    pub fn set_root(&mut self, n: NodeRef) {
        self.root_index = Some(n.index);
    }

    pub fn get_root_node<'a>(&'a self) -> Option<&'a Node> {
        if let Some(n) = self.get_root() {
            return Some(self.get_node(&n));
        }
        return None;
    }

    pub fn get_root(&self) -> Option<NodeRef> {
        if let Some(i) = self.root_index {
            return Some(NodeRef { index: i });
        }
        return None;
    }

    pub fn add_symbol(&mut self, symbol: &str) -> SymbolRef {
        return self.symbols.insert(symbol);
    }

    pub fn get_symbol(&self, symbolref: &SymbolRef) -> Option<&String> {
        return self.symbols.get(symbolref);
    }
}

#[enum_dispatch]
pub trait NamedNode {
    fn name(&self) -> &str;
}

macro_rules! node_type {
    ($name:ident {$($field:ident: $t:ty $(,)?)*}) => {
        #[derive(Debug)]
        pub struct $name {
            $(pub $field: $t),*
        }

        impl NamedNode for $name {
            fn name(&self) -> &str { stringify!($name) }
        }
    }
}

node_type!(Invalid {});

node_type!(ModuleFragment {
    statementbody: NodeRef,
});

node_type!(StatementBody {
    statements: Vec<NodeRef>,
});

node_type!(IntegerLiteral {
    value: u64,
    signed: bool,
});

node_type!(StringLiteral { text: String });

node_type!(FunctionLiteral {
    inputparams: Vec<NodeRef>,
    outputparams: Vec<NodeRef>,
    body: NodeRef,
});

node_type!(InputParameter {
    symbol: SymbolRef,
    typeexpr: NodeRef,
});

node_type!(OutputParameter { typeexpr: NodeRef });

node_type!(BuiltInObjectReference {
    object: BuiltInObject,
});

node_type!(SymbolReference { symbol: SymbolRef });

node_type!(ReturnStatement { expr: NodeRef });

node_type!(ArgumentList {
    args: Vec<NodeRef>,
});

node_type!(CallOperation {
    expr: NodeRef,
    arglist: NodeRef,
});

#[derive(Debug)]
pub enum BinaryOperationType {
    Add,
    Sub,
    Mul,
    Div,
}

node_type!(BinaryOperation {
    optype: BinaryOperationType,
    lhs: NodeRef,
    rhs: NodeRef,
});

node_type!(SymbolDeclaration {
    symbol: SymbolRef,
    typeexpr: Option<NodeRef>,
    initexpr: NodeRef,
});

#[enum_dispatch(NamedNode)]
#[derive(Debug)]
pub enum Node {
    Invalid,
    ModuleFragment,
    StatementBody,
    IntegerLiteral,
    // TODO: BigIntegerLiteral
    StringLiteral,
    FunctionLiteral,
    InputParameter,
    OutputParameter,
    BuiltInObjectReference, // NOTE: This shortcuts having to evaluate built-ins as ordinary expressions
    SymbolReference,
    ReturnStatement,
    ArgumentList,
    // TODO: Can this be generalized to parameterized symbol reference?
    //  The same syntax is used for function calls, type parameteters etc
    CallOperation,
    BinaryOperation,
    SymbolDeclaration,
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
            }
            .node_print(&r);
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

    fn node_print(&mut self, noderef: &NodeRef) {
        let node = self.ast.get_node(noderef);
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
                if let Some(s) = self.ast.symbols.get(&SymbolRef { key: key.unwrap() }) {
                    buf.splice(m.0, s.bytes());
                }

                nodetext = String::from_utf8(buf).unwrap();
            }
        }

        println!(
            "{:indent$}{}: {:?}",
            "",
            noderef.index,
            nodetext,
            indent = (self.left_padding + self.level * 4) as usize
        );

        // Recurse into subtree
        match &node {
            Node::Invalid(_) => (),
            Node::IntegerLiteral(_n) => (),
            Node::StringLiteral(_n) => (),
            Node::BuiltInObjectReference(_n) => (),
            Node::SymbolReference(_n) => (),
            Node::ModuleFragment(n) => {
                self.print_child(&n.statementbody);
            }
            Node::StatementBody(n) => {
                for s in &n.statements {
                    self.print_child(s);
                }
            }
            Node::FunctionLiteral(n) => {
                for p in &n.inputparams {
                    self.print_child(p);
                }
                for p in &n.outputparams {
                    self.print_child(p);
                }
                self.print_child(&n.body);
            }
            Node::InputParameter(n) => {
                self.print_child(&n.typeexpr);
            }
            Node::OutputParameter(n) => {
                self.print_child(&n.typeexpr);
            }
            Node::ReturnStatement(n) => {
                self.print_child(&n.expr);
            }
            Node::ArgumentList(n) => {
                for a in &n.args {
                    self.print_child(a);
                }
            }
            Node::CallOperation(n) => {
                self.print_child(&n.expr);
                self.print_child(&n.arglist);
            }
            Node::BinaryOperation(n) => {
                self.print_child(&n.lhs);
                self.print_child(&n.rhs);
            }
            Node::SymbolDeclaration(n) => {
                self.print_optional_child(&n.typeexpr);
                self.print_child(&n.initexpr);
            }
        }
    }
}
