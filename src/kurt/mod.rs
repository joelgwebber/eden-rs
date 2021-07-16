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
    Nil,
    Num(f64),
    Bool(bool),
    Str(String),
    Id(String),
    Sym(String),
    Native(fn(env: Node) -> Node),

    List(NodeRef<Vec<Node>>),
    Dict(NodeRef<HashMap<String, Node>>),
    Block(NodeRef<(Vec<Node>, Node)>),

    Exec,
}

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

#[derive(Trace, Finalize)]
pub struct NodeRef<T: Trace + 'static>(Gc<GcCell<T>>);

impl<T: Trace> NodeRef<T> {
    pub fn new(node: T) -> Self {
        NodeRef(Gc::new(GcCell::new(node)))
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        self.try_borrow().expect("already mutably borrowed")
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T, T> {
        self.try_borrow_mut().expect("already borrowed")
    }

    pub fn try_borrow(&self) -> Result<Ref<'_, T>, BorrowError> {
        self.0.try_borrow().map_err(|_| BorrowError)
    }

    pub fn try_borrow_mut(&self) -> Result<RefMut<'_, T, T>, BorrowMutError> {
        self.0.try_borrow_mut().map_err(|_| BorrowMutError)
    }
}

impl<T: Trace> Clone for NodeRef<T> {
    #[inline]
    fn clone(&self) -> Self {
        NodeRef(self.0.clone())
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        match self {
            Node::Nil => Node::Nil,
            Node::Num(x) => Node::Num(*x),
            Node::Bool(x) => Node::Bool(*x),
            Node::Str(x) => Node::Str(x.clone()),
            Node::Id(x) => Node::Id(x.clone()),
            Node::Sym(x) => Node::Sym(x.clone()),
            Node::Native(x) => Node::Native(*x),
            Node::List(r) => Node::List(r.clone()),
            Node::Dict(r) => Node::Dict(r.clone()),
            Node::Block(r) => Node::Block(r.clone()),
            Node::Exec => Node::Exec,
        }
    }
}

impl Node {
    pub fn define(&mut self, k: &String, val: Node) -> Result<(), String> {
        match self {
            Node::Dict(ref mut map_ref) => {
                map_ref.borrow_mut().insert(k.clone(), val);
                Ok(())
            }
            _ => Err(String::from("def only works on dicts")),
        }
    }

    pub fn lookup(&self, name: &String) -> Result<Node, String> {
        match self {
            // Check current dict.
            Node::Dict(map_ref) => {
                if let Some(n) = map_ref.borrow().get(name) {
                    return Ok(n.clone());
                }

                // Check parent.
                match map_ref.borrow().get("^") {
                    Some(next) => next.borrow().lookup(name),
                    None => Err(String::from("couldn't find symbol")),
                }
            }

            // TODO: integer lookups for lists.
            Node::List(_) => Err(String::from("nyi")),

            _ => Err(String::from("couldn't find symbol")),
        }
    }

    pub fn lookup_node(&self, name: Node) -> Result<Node, String> {
        if let Node::Id(s) = &*name.borrow() {
            return self.lookup(s);
        }
        Err(String::from("couldn't find symbol"))
    }
}
