use std::{borrow::Borrow, collections::HashMap};

use crate::kurt::Node;

use super::NodeRef;

// TODO
pub fn eval(env: Node, expr: Node) -> Node {
    println!("eval -- {} :: {}", env.borrow(), expr.borrow());
    match &expr {
        // <prim> -> <prim>
        Node::Nil => expr.clone(),
        Node::Num(_) => expr.clone(),
        Node::Bool(_) => expr.clone(),
        Node::Str(_) => expr.clone(),

        // :sym -> :sym
        Node::Sym(_) => expr.clone(),

        // (block) -> (block)
        Node::Block(_) => expr.clone(),

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
        Node::Dict(map_ref) => {
            let mut new_map = HashMap::<String, Node>::new();
            for (key, node) in &*map_ref.borrow() {
                new_map.insert(key.clone(), eval(env.clone(), node.clone()));
            }
            Node::Dict(NodeRef::new(new_map))
        }

        // [exprs...] -> [eval [exprs...]]
        Node::List(vec_ref) => {
            let vec = &*vec_ref.borrow();
            let exprs: Vec<Node> = vec
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
            Node::List(NodeRef::new(exprs))
        }
    }
}

pub fn exec(env: Node, vec: Vec<Node>) -> Node {
    let ls = Node::List(NodeRef::new(vec.clone()));
    println!("exec -- {} :: {}", env.clone(), ls);

    match vec.len() {
        // () -> nil
        // TODO: Does this make sense?
        0 => Node::Nil,

        // (expr) -> expr
        // TODO: zero-param call
        1 => vec.first().unwrap().clone(),

        // [expr expr] ->
        2 => {
            let first = vec.get(0).unwrap();
            let second = vec.get(1).unwrap();
            match (&*first.borrow(), &*second.borrow()) {
                // (dict id) -> access
                (Node::Dict(_), Node::Id(_)) => eval(first.clone(), second.clone()),

                // (dict block) -> invoke
                (Node::Dict(map_ref), Node::Block(block_ref)) => {
                    let params = copy_map(&*map_ref.borrow());
                    eval(Node::Dict(NodeRef::new(params)), block_ref.borrow().1.clone())
                }

                _ => unimplemented!(),
            }
        }

        _ => unimplemented!(),
    }
}

fn copy_map(map: &HashMap<String, Node>) -> HashMap<String, Node> {
    let mut new_map = HashMap::<String, Node>::new();
    for (key, node) in map {
        new_map.insert(key.clone(), node.clone());
    }
    new_map
}
