use std::borrow::Borrow;
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

use self::builtins::init_builtins;

pub mod builtins;
pub mod eq;
pub mod eval;
pub mod parse;
pub mod print;

mod tests;

pub struct Kurt {
    pub root: Node,
}

impl Kurt {
    pub fn new() -> Kurt {
        let mut root_map = HashMap::new();
        init_builtins(&mut root_map);
        Kurt {
            root: Node::Dict(NodeRef::new(root_map)),
        }
    }

    pub fn eval_src(&mut self, src: &str) -> Node {
        let expr = parse::parse(src.into());
        eval::eval(self.root.clone(), expr)
    }

    pub fn def(&mut self, sym: String, val: Node) {
        if let Node::Dict(map_ref) = self.root.borrow() {
            let map = &mut *map_ref.borrow_mut();
            map.insert(sym, val);
        }
    }
}

// Node represents both the AST and runtime state.
// Parsing produces a Node graph, and evaluation updates that graph.
#[derive(Trace, Finalize, PartialEq)]
pub enum Node {
    Nil,
    Num(f64),
    Bool(bool),
    Str(String),
    Id(String),
    Native(fn(Node) -> Node),

    List(NodeRef<Vec<Node>>),
    Dict(NodeRef<HashMap<String, Node>>),
    Block(NodeRef<Block>),
    Apply(NodeRef<Vec<Node>>),

    DictDef(NodeRef<Vec<(Node, Node)>>),
    Quote(NodeRef<Node>),
}

impl UnwindSafe for Node {}
impl RefUnwindSafe for Node {}

#[derive(Trace, Finalize, PartialEq)]
pub struct Block {
    params: Vec<String>,
    env: Node,
    expr: Node,
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
            Node::Native(x) => Node::Native(*x),
            Node::List(r) => Node::List(r.clone()),
            Node::DictDef(r) => Node::DictDef(r.clone()),
            Node::Dict(r) => Node::Dict(r.clone()),
            Node::Block(r) => Node::Block(r.clone()),
            Node::Apply(r) => Node::Apply(r.clone()),
            Node::Quote(r) => Node::Quote(r.clone()),
        }
    }
}

impl Node {
    pub fn def(&self, name: &String, val: Node) {
        match self {
            Node::Dict(map_ref) => {
                let map = &mut *map_ref.borrow_mut();
                map.insert(name.clone(), val);
            }
            _ => panic!("def requires a dict"),
        }
    }

    pub fn set(&self, name: &String, val: Node) {
        let target = self.find(name);
        match &target {
            Node::Dict(map_ref) => {
                let map = &mut *map_ref.borrow_mut();
                map.insert(name.clone(), val);
            }
            Node::Nil => panic!("{} not found", name),
            _ => panic!("set requires a dict"),
        }
    }

    pub fn get(&self, name: &String) -> Node {
        let target = self.find(name);
        match &target {
            Node::Dict(map_ref) => map_ref.borrow().get(name).unwrap().clone(),
            Node::Nil => panic!("{} not found", name),
            _ => panic!("get requires a dict"),
        }
    }

    pub fn get_node(&self, name: Node) -> Node {
        if let Node::Id(s) = &*name.borrow() {
            return self.get(s);
        }
        panic!("couldn't find symbol")
    }

    fn find(&self, name: &String) -> Node {
        match self {
            // Check current dict.
            Node::Dict(map_ref) => {
                if map_ref.borrow().contains_key(name) {
                    return self.clone();
                }

                // Check parent.
                match map_ref.borrow().get("^") {
                    Some(next) => next.borrow().find(name),
                    None => panic!("couldn't find symbol '{}'\nenv: {}", name, self),
                }
            }

            // TODO: integer lookups for lists.
            Node::List(_) => unimplemented!(),

            _ => Node::Nil,
        }
    }
}
