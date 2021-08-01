use std::borrow::Borrow;

use velcro::vec_from;

use crate::kurt::builtins::{builtin, native_set};

use super::node::Node;

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
                    None => default_dict(name),
                }
            }

            // TODO: integer lookups for lists.
            Node::List(_) => unimplemented!(),

            _ => Node::Nil,
        }
    }
}

fn default_dict(name: &String) -> Node {
    match name.as_str() {
        "set" => builtin(vec_from!["vals"], native_set),
        _ => panic!("couldn't find symbol '{}'", name),
    }
}
