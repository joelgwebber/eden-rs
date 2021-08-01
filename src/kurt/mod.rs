use std::{borrow::Borrow, collections::HashMap};

use self::{
    builtins::init_builtins,
    node::{Node, NodeRef},
};

pub mod node;
pub mod env;
pub mod parse;
pub mod eval;
pub mod print;
pub mod builtins;
pub mod eq;

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
