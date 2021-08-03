use std::{collections::HashMap};

use self::node::{Node, NodeRef};

pub mod apply;
pub mod builtins;
pub mod eq;
pub mod eval;
pub mod node;
pub mod parse;
pub mod print;

mod tests;

pub struct Kurt {
    pub root: Node,
    builtins: HashMap<&'static str, fn(&Kurt, &Node) -> Node>,
    def_dict: Node,
    def_list: Node,
    debug: bool,
}

impl Kurt {
    pub fn new() -> Kurt {
        let mut kurt = Kurt {
            builtins: HashMap::new(),
            root: Node::Dict(NodeRef::new(HashMap::new())),
            def_dict: Node::Nil,
            def_list: Node::Nil,
            debug: false,
        };
        kurt.init_builtins();
        kurt
    }

    pub fn eval_src(&mut self, src: &str) -> Node {
        let expr = self.parse(src.into());
        self.eval(&self.root, &expr)
    }

    pub fn def(&self, env: &Node, key: &Node, val: &Node) {
        match (env, key) {
            (Node::Dict(map_ref), Node::Id(name)) => {
                let map = &mut *map_ref.borrow_mut();
                map.insert(name.clone(), val.clone());
            }

            _ => panic!(),
        }
    }

    pub fn set(&self, env: &Node, name: &Node, val: &Node) {
        match (env, name) {
            (Node::Dict(_), Node::Id(s)) => {
                let target = self.find_named(env, s);
                match &target {
                    Node::Dict(map_ref) => {
                        let map = &mut *map_ref.borrow_mut();
                        map.insert(s.clone(), val.clone());
                    }
                    Node::Nil => panic!("{} not found", name),
                    _ => panic!("set requires a dict"),
                }
            }

            (Node::List(vec_ref), Node::Num(idx)) => {
                let vec = &mut *vec_ref.borrow_mut();
                let node_ref = vec.get_mut(idx.floor() as usize).unwrap();
                *node_ref = val.clone();
            }

            (_, _) => panic!(),
        }
    }

    pub fn get(&self, env: &Node, name: &Node) -> Node {
        match (env, &name) {
            (Node::Dict(_), Node::Id(name)) => {
                match name.as_str() {
                    // Special case: env refers to the current environment.
                    "env" => env.clone(),
                    _ => {
                        let target = self.find_named(env, name);
                        match &target {
                            Node::Dict(map_ref) => map_ref.borrow().get(name).unwrap().clone(),
                            Node::Nil => panic!("{} not found", name),
                            _ => panic!("get requires a dict"),
                        }
                    }
                }
            }

            (Node::List(vec_ref), Node::Num(x)) => {
                // TODO: bounds/error checking
                vec_ref.borrow().get(x.floor() as usize).unwrap().clone()
            }

            (Node::List(_), Node::Id(name)) => {
                self.get(&self.def_list, &Node::Id(name.clone()))
            }

            (_, _) => name.clone(),
        }
    }

    fn find_named(&self, target: &Node, name: &String) -> Node {
        match target {
            // Check current dict.
            Node::Dict(map_ref) => {
                if map_ref.borrow().contains_key(name) {
                    return target.clone();
                }

                // Check parent.
                match map_ref.borrow().get("^") {
                    Some(next) => self.find_named(next, name),
                    None => {
                        match &self.def_dict {
                            Node::Dict(def_map_ref) => {
                                if def_map_ref.borrow().contains_key(name) {
                                    self.def_dict.clone()
                                } else {
                                    Node::Nil
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }

            _ => Node::Nil,
        }
    }
}
