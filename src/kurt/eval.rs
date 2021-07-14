use std::{borrow::Borrow, collections::HashMap};

use crate::kurt::Node;

use super::NodeRef;

// TODO
pub fn eval(env: NodeRef, expr: NodeRef) -> NodeRef {
    println!("eval -- {} :: {}", env.borrow(), expr.borrow());
    match &*expr.borrow() {
        // <prim> -> <prim>
        Node::Nil => expr.clone(),
        Node::Num(x) => expr.clone(),
        Node::Bool(x) => expr.clone(),
        Node::Str(ref x) => expr.clone(),

        // :sym -> :sym
        Node::Sym(ref s) => expr.clone(),

        // (block) -> (block)
        Node::Block(_, _) => expr.clone(),

        Node::Native(f) => f(env),

        // ident -> env:ident
        Node::Id(s) => {
            match s.as_str() {
                // Special cases:
                // - env refers to the current environment.
                // - $ evaluates to itself (used to trigger list exec below)
                "env" => env.clone(),
                "$" => expr.clone(),
                _ => match env.borrow().lookup_node(expr.clone()) {
                    Ok(n) => n,
                    Err(msg) => panic!("lookup failed: {}", msg),
                },
            }
        }

        // { key:expr... } -> { key:[eval expr]... }
        Node::Dict(map) => {
            let mut new_map = HashMap::<String, NodeRef>::new();
            for (key, node) in map {
                new_map.insert(key.clone(), eval(env.clone(), node.clone()));
            }
            NodeRef::new(Node::Dict(new_map))
        }

        // [exprs...] -> [eval [exprs...]]
        Node::List(vec) => {
            let exprs: Vec<NodeRef> = vec
                .into_iter()
                .map(|node| eval(env.clone(), node.clone()))
                .collect();
            if vec.len() > 0 {
                let first = vec.get(0).unwrap().borrow();
                match &*first {
                    Node::Id(id) => {
                        if id == "$" {
                            return exec(env.clone(), exprs[1..].to_vec());
                        }
                    }
                    _ => {}
                }
            }
            NodeRef::new(Node::List(exprs))
        }
    }
}

pub fn exec(env: NodeRef, vec: Vec<NodeRef>) -> NodeRef {
    let ls = Node::List(vec.clone());
    println!("exec -- {} :: {}", env.clone(), ls);

    match vec.len() {
        // () -> nil
        // TODO: Does this make sense?
        0 => NodeRef::new(Node::Nil),

        // (expr) -> expr
        // TODO: zero-param call
        1 => vec.first().unwrap().clone(),

        // [expr expr] ->
        2 => {
            let first = vec.get(0).unwrap();
            let second = vec.get(1).unwrap();
            match (&*first.borrow(), &*second.borrow()) {
                // (dict id) -> access
                (Node::Dict(_), Node::Id(s)) => eval(first.clone(), second.clone()),

                // (dict block) -> invoke
                (Node::Dict(map), Node::Block(_, expr)) => {
                    let params = copy_map(map);
                    eval(NodeRef::new(Node::Dict(params)), expr.clone())
                }

                _ => unimplemented!(),
            }
        }

        _ => unimplemented!(),
    }
}

fn copy_map(map: &HashMap<String, NodeRef>) -> HashMap<String, NodeRef> {
    let mut new_map = HashMap::<String, NodeRef>::new();
    for (key, node) in map {
        new_map.insert(key.clone(), node.clone());
    }
    new_map
}
