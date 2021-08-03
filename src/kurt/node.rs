use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::panic::RefUnwindSafe;
use std::panic::UnwindSafe;

use gc::Finalize;
use gc::Gc;
use gc::GcCell;
use gc::GcCellRef;
use gc::GcCellRefMut;
use gc::Trace;

// Node represents both the AST and runtime state.
// Parsing produces a Node graph, and evaluation updates that graph.
#[derive(Trace, Finalize, PartialEq)]
pub enum Node {
    Nil,
    Num(f64),
    Bool(bool),
    Str(String),
    Id(String),
    Native(&'static str),

    List(NodeRef<Vec<Node>>),
    Assoc(NodeRef<Vec<(Node, Node)>>),
    Dict(NodeRef<HashMap<String, Node>>),
    Block(NodeRef<Block>),
    Apply(NodeRef<Vec<Node>>),

    Quote(NodeRef<Node>),
    Unquote(NodeRef<Node>),
}

// Needed for the use of nodes in the panic handler.
impl UnwindSafe for Node {}
impl RefUnwindSafe for Node {}

// State for a (| block) node, including params and environment.
#[derive(Trace, Finalize, PartialEq)]
pub struct Block {
    pub params: Vec<String>,
    pub expr: Node,
    pub env: Node,
    pub slf: Node,
}

// Utilities to simplify borrowing through NodeRefs.
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

#[derive(Trace, Finalize, PartialEq)]
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

// Make NodeRefs cloneable, so that Node can be cloneable.
impl<T: Trace> Clone for NodeRef<T> {
    #[inline]
    fn clone(&self) -> Self {
        NodeRef(self.0.clone())
    }
}

// Make nodes cloneable. Value-types are trivially cloned by value; ref-types only clone their refs.
// Cloning a node is always a cheap operation.
impl Clone for Node {
    fn clone(&self) -> Self {
        match self {
            Node::Nil => Node::Nil,
            Node::Num(x) => Node::Num(*x),
            Node::Bool(x) => Node::Bool(*x),
            Node::Str(x) => Node::Str(x.clone()),
            Node::Id(x) => Node::Id(x.clone()),
            Node::Native(x) => Node::Native(*x),
            Node::List(r) => Node::List(r.clone()),
            Node::Assoc(r) => Node::Assoc(r.clone()),
            Node::Dict(r) => Node::Dict(r.clone()),
            Node::Block(r) => Node::Block(r.clone()),
            Node::Apply(r) => Node::Apply(r.clone()),
            Node::Quote(r) => Node::Quote(r.clone()),
            Node::Unquote(r) => Node::Unquote(r.clone()),
        }
    }
}
