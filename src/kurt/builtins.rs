use std::{borrow::{Borrow, BorrowMut}, collections::HashMap, vec};

use crate::kurt::Node;

use super::NodeRef;

pub fn init_builtins(map: &mut HashMap<String, Node>) {
    map.insert(String::from("do"), builtin(vec![String::from("exprs")], native_do));
    map.insert(String::from("def"), builtin(vec![String::from("vals")], native_def));
    map.insert(String::from("log"), builtin(vec![String::from("msg")], native_log));
    // map.insert(String::from("+"), builtin(vec![String::from("a"), String::from("b")], native_add));
}

fn native_do(env: Node) -> Node {
    match loc(&env, "exprs") {
        Some(exprs) => {
            if let Node::List(vec_ref) = &exprs {
                vec_ref.borrow().last().unwrap().clone()
            } else {
                Node::Nil
            }
        }
        None => Node::Nil
    }
}

fn native_def(mut env: Node) -> Node {
    match loc(&env, "vals") {
        Some(vals) => {
            if let Node::Dict(vals_map_ref) = &vals {
                for (k, v) in &*vals_map_ref.borrow() {
                  env.borrow_mut().define(&k, v.clone());
                }
            }
        }
        None => panic!("wut"),
    }
    Node::Nil
}

fn native_log(env: Node) -> Node {
    println!("{}", loc_str(env, "msg"));
    Node::Nil
}

fn builtin(args: Vec<String>, f: fn(env: Node) -> Node) -> Node {
    let nargs: Vec<Node> = args.iter().map(|arg| Node::Sym(arg.clone())).collect();
    Node::Block(NodeRef::new((nargs, Node::Native(f))))
}

fn loc(env: &Node, name: &str) -> Option<Node> {
    if let Node::Dict(env_map_ref) = &env {
        let env_map = &*env_map_ref.borrow();
        let result = env_map.get(name).unwrap();
        Some(result.clone())
    } else {
        None
    }
}

fn loc_str(env: Node, name: &str) -> String {
    match &env {
        Node::Dict(map_ref) => match map_ref.borrow().get(name) {
            Some(loc) => match loc {
                Node::Str(s) => s.clone(),
                _ => String::from(""),
            },
            None => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}
