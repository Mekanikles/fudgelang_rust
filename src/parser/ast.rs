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
        self.nodes.push(nodes::Invalid {}.into());
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

    pub fn get_node_as<'a>(&'a self, noderef: &NodeRef) -> &'a Node {
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

    pub fn find_first_node(&self, nodeid: NodeId) -> Option<NodeRef> {
        fn search_subtree(ast: &Ast, noderef: &NodeRef, nodeid: NodeId) -> Option<NodeRef> {
            let node = ast.get_node(noderef);
            if node.id() == nodeid {
                return Some(*noderef);
            }

            // Bleh
            let mut found: Option<NodeRef> = None;
            visit_children(node, |childref| {
                if let Some(n) = search_subtree(ast, childref, nodeid) {
                    found = Some(n);
                    return false;
                }
                return true;
            });

            return found;
        }

        if let Some(root) = self.get_root() {
            return search_subtree(self, &root, nodeid);
        }

        return None;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOperationType {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymbolDeclarationType {
    Def,
    Var,
}

// Declares enums and data structs associated with ast nodes
macro_rules! declare_nodes  {
    // Main macro
    ($($node_name:ident // Node name
        $( // Optional
            { // Node body
                $($field:ident: $field_t:ty $(,)?)* // field
            } $(,)?
        )?,
    )*) => {
        pub trait NodeInfo {
            fn id(&self) -> NodeId;
            fn name(&self) -> &str;
        }

        // Enum with ids
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum NodeId {
            $($node_name),*
        }

        // Node implementations
        pub mod nodes {
            pub trait NodeImpl {
                fn id() -> NodeId;
            }

            use super::*;
            $(declare_nodes!(@node_struct
                $node_name {
                    $($($field: $field_t),*)?
                }
            );)*
        }

        // Node union object
        #[derive(Debug)]
        pub enum Node {
            $($node_name(nodes::$node_name)),*
        }

        // Trait implementations for Node
        impl NodeInfo for Node {
            fn id(&self) -> NodeId {
                match (&self) {
                    $(Node::$node_name(n) => n.id()),*
                }
            }
            fn name(&self) -> &str {
                match (&self) {
                    $(Node::$node_name(n) => n.name()),*
                }
            }
        }

        pub fn visit_children<T>(node: &Node, mut func: T) where T: FnMut(&NodeRef) -> bool {
            match &node {
                $(Node::$node_name(n) => {
                    let mut children = Vec::new();
                    n.collect_children(&mut children);
                    for c in children {
                        if !func(&c) {
                            break;
                        }
                    }
                }),*
            }
        }
    };

    // Node struct definitions
    (@node_struct $name:ident {$($field:ident: $t:ty $(,)?)*}) => {
        #[derive(Debug)]
        pub struct $name {
            $(pub $field: $t),*
        }

        // Allow conversion to enum type from struct
        impl Into<Node> for $name {
            fn into(self) -> Node {
                Node::$name(self)
            }
        }

        impl NodeInfo for $name {
            fn id(&self) -> NodeId { NodeId::$name }
            fn name(&self) -> &str { stringify!($name) }
        }

        impl NodeImpl for $name {
            fn id() -> NodeId { NodeId::$name }
        }
    };
}

declare_nodes!(
    Invalid,
    ModuleFragment {
        statementbody: NodeRef,
    },
    StatementBody {
        statements: Vec<NodeRef>,
    },
    BooleanLiteral {
        value: bool,
    },
    IntegerLiteral {
        value: u64,
        signed: bool,
    },
    // TODO: BigIntegerLiteral
    StringLiteral { text: String },
    FunctionLiteral {
        inputparams: Vec<NodeRef>,
        outputparams: Vec<NodeRef>,
        body: NodeRef,
    },
    InputParameter {
        symbol: SymbolRef,
        typeexpr: NodeRef,
    },
    OutputParameter { typeexpr: NodeRef },
    BuiltInObjectReference {
        object: BuiltInObject,
    },
    SymbolReference { symbol: SymbolRef },
    IfStatement { condition: NodeRef, thenstmnt: NodeRef, elsestmnt: Option<NodeRef> },
    ReturnStatement { expr: Option<NodeRef> },
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
        optype: BinaryOperationType,
        lhs: NodeRef,
        rhs: NodeRef,
    },
    SymbolDeclaration {
        symbol: SymbolRef,
        decltype: SymbolDeclarationType,
        typeexpr: Option<NodeRef>,
        initexpr: NodeRef,
    },
);

// Why no trait specializations :(
trait ChildCollector {
    fn collect_children(&self, _collector: &mut Vec<NodeRef>);
}

impl ChildCollector for nodes::Invalid {
    fn collect_children(&self, _collector: &mut Vec<NodeRef>) {
        panic!("Visited invalid node!");
    }
}

impl ChildCollector for nodes::ModuleFragment {
    fn collect_children(&self, collector: &mut Vec<NodeRef>) {
        collector.push(self.statementbody);
    }
}

impl ChildCollector for nodes::StatementBody {
    fn collect_children(&self, collector: &mut Vec<NodeRef>) {
        for n in &self.statements {
            collector.push(*n);
        }
    }
}

impl ChildCollector for nodes::BooleanLiteral {
    fn collect_children(&self, _collector: &mut Vec<NodeRef>) {}
}

impl ChildCollector for nodes::IntegerLiteral {
    fn collect_children(&self, _collector: &mut Vec<NodeRef>) {}
}

impl ChildCollector for nodes::StringLiteral {
    fn collect_children(&self, _collector: &mut Vec<NodeRef>) {}
}

impl ChildCollector for nodes::FunctionLiteral {
    fn collect_children(&self, collector: &mut Vec<NodeRef>) {
        for n in &self.inputparams {
            collector.push(*n);
        }
        for n in &self.outputparams {
            collector.push(*n);
        }

        collector.push(self.body);
    }
}

impl ChildCollector for nodes::InputParameter {
    fn collect_children(&self, collector: &mut Vec<NodeRef>) {
        collector.push(self.typeexpr);
    }
}

impl ChildCollector for nodes::OutputParameter {
    fn collect_children(&self, collector: &mut Vec<NodeRef>) {
        collector.push(self.typeexpr);
    }
}

impl ChildCollector for nodes::BuiltInObjectReference {
    fn collect_children(&self, _collector: &mut Vec<NodeRef>) {}
}

impl ChildCollector for nodes::SymbolReference {
    fn collect_children(&self, _collector: &mut Vec<NodeRef>) {}
}

impl ChildCollector for nodes::IfStatement {
    fn collect_children(&self, collector: &mut Vec<NodeRef>) {
        collector.push(self.condition);
        collector.push(self.thenstmnt);
        if let Some(n) = &self.elsestmnt {
            collector.push(*n);
        }
    }
}

impl ChildCollector for nodes::ReturnStatement {
    fn collect_children(&self, collector: &mut Vec<NodeRef>) {
        if let Some(n) = &self.expr {
            collector.push(*n);
        }
    }
}

impl ChildCollector for nodes::ArgumentList {
    fn collect_children(&self, collector: &mut Vec<NodeRef>) {
        for n in &self.args {
            collector.push(*n);
        }
    }
}

impl ChildCollector for nodes::CallOperation {
    fn collect_children(&self, collector: &mut Vec<NodeRef>) {
        collector.push(self.expr);
        collector.push(self.arglist);
    }
}

impl ChildCollector for nodes::BinaryOperation {
    fn collect_children(&self, collector: &mut Vec<NodeRef>) {
        collector.push(self.lhs);
        collector.push(self.rhs);
    }
}

impl ChildCollector for nodes::SymbolDeclaration {
    fn collect_children(&self, collector: &mut Vec<NodeRef>) {
        if let Some(n) = &self.typeexpr {
            collector.push(*n);
        }
        collector.push(self.initexpr);
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
            }
            .node_print(&r);
        }
    }
}

impl<'a> AstPrinter<'a> {
    fn node_print(&mut self, noderef: &NodeRef) {
        let node = self.ast.get_node(noderef);
        let mut nodetext = format!("{:?}", node);

        // Make symbol references human readable
        {
            let re = Regex::new(r"<string key: (\d+)>").unwrap();
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
        visit_children(node, |noderef| {
            self.level += 1;
            self.node_print(noderef);
            self.level -= 1;
            return true;
        })
    }
}
