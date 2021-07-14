use std::{borrow::BorrowMut, collections::HashMap, vec};

use crate::kurt::Node;

use super::NodeRef;

pub fn init_builtins(map: &mut HashMap<String, NodeRef>) {
    map.insert(String::from("def"), builtin(vec![String::from("vals")], native_def));
    map.insert(String::from("log"), builtin(vec![String::from("msg")], native_log));
    // map.insert(String::from("+"), builtin(vec![String::from("a"), String::from("b")], native_add));
}

fn native_def(mut env: NodeRef) -> NodeRef {
    match loc(env.clone(), "vals") {
        Some(vals) => {
            if let Node::Dict(vals_map) = &*vals.borrow() {
                for (k, v) in vals_map {
                  env.borrow_mut().define(k, v.clone());
                }
            }
        }
        None => panic!("wut"),
    }
    NodeRef::new(Node::Nil)
}

fn native_log(env: NodeRef) -> NodeRef {
    println!("{}", loc_str(env, "msg"));
    NodeRef::new(Node::Nil)
}

fn builtin(args: Vec<String>, f: fn(env: NodeRef) -> NodeRef) -> NodeRef {
    let nargs: Vec<NodeRef> = args.iter().map(|arg| NodeRef::new(Node::Sym(arg.clone()))).collect();
    NodeRef::new(Node::Block(nargs, NodeRef::new(Node::Native(f))))
}

fn loc(env: NodeRef, name: &str) -> Option<NodeRef> {
    if let Node::Dict(env_map) = &*env.borrow_mut() {
        let result = env_map.get(name).unwrap();
        Some(result.clone())
    } else {
        None
    }
}

fn loc_str(env: NodeRef, name: &str) -> String {
    match &*env.borrow() {
        Node::Dict(map) => match map.get(name) {
            Some(loc) => match &*loc.borrow() {
                Node::Str(s) => s.clone(),
                _ => String::from(""),
            },
            None => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}
