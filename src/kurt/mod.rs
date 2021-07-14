use std::borrow::Borrow;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::Display;

use gc::Finalize;
use gc::Gc;
use gc::GcCell;
use gc::GcCellRef;
use gc::GcCellRefMut;
use gc::Trace;

pub mod builtins;
pub mod eval;
pub mod parse;
pub mod print;

// Node represents both the AST and runtime state.
// Parsing produces a Node graph, and evaluation updates that graph.
#[derive(Trace, Finalize)]
pub enum Node {
    Dict(HashMap<String, NodeRef>),
    List(Vec<NodeRef>),
    Str(String),
    Num(f64),
    Bool(bool),
    Id(String),
    Sym(String),
    Block(Vec<NodeRef>, NodeRef),
    Native(fn(env: NodeRef) -> NodeRef),
    Nil,
}

#[derive(Trace, Finalize)]
pub struct NodeRef(Gc<GcCell<Node>>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BorrowError;
impl Error for BorrowError {}
impl Display for BorrowError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt("already mutably borrowed", f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BorrowMutError;
impl Error for BorrowMutError {}
impl Display for BorrowMutError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt("already borrowed", f)
    }
}

pub type Ref<'a, T> = GcCellRef<'a, T>;
pub type RefMut<'a, T, U> = GcCellRefMut<'a, T, U>;

impl NodeRef {
    pub fn new(node: Node) -> Self {
        Self(Gc::new(GcCell::new(node)))
    }

    pub fn borrow(&self) -> Ref<'_, Node> {
        self.try_borrow().expect("already mutably borrowed")
    }

    pub fn borrow_mut(&self) -> RefMut<'_, Node, Node> {
        self.try_borrow_mut().expect("already borrowed")
    }

    pub fn try_borrow(&self) -> Result<Ref<'_, Node>, BorrowError> {
        self.0.try_borrow().map_err(|_| BorrowError)
    }

    pub fn try_borrow_mut(&self) -> Result<RefMut<'_, Node, Node>, BorrowMutError> {
        self.0.try_borrow_mut().map_err(|_| BorrowMutError)
    }
}

impl Clone for NodeRef {
    #[inline]
    fn clone(&self) -> Self {
        NodeRef(self.0.clone())
    }
}

impl Node {
    pub fn define(&mut self, k: &String, val: NodeRef) -> Result<(), String> {
        match self {
            Node::Dict(ref mut map) => {
                map.insert(k.clone(), val.clone());
                Ok(())
            }
            _ => Err(String::from("def only works on dicts")),
        }
    }

    pub fn lookup(&self, name: &String) -> Result<NodeRef, String> {
        match self {
            // Check current dict.
            Node::Dict(map) => {
                if let Some(n) = map.get(name) {
                    return Ok(n.clone());
                }

                // Check parent.
                match map.get("^") {
                    Some(next) => next.borrow().lookup(name),
                    None => Err(String::from("couldn't find symbol")),
                }
            }

            // TODO: integer lookups for lists.
            Node::List(_) => Err(String::from("nyi")),

            _ => Err(String::from("couldn't find symbol")),
        }
    }

    pub fn lookup_node(&self, name: NodeRef) -> Result<NodeRef, String> {
        if let Node::Id(s) = &*name.borrow() {
            return self.lookup(s);
        }
        Err(String::from("couldn't find symbol"))
    }
}
