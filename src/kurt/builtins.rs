use std::{collections::HashMap, panic, vec};

use velcro::vec_from;

use crate::kurt::{eq::native_eq, Node};

use super::{eval::apply, Block, NodeRef};

pub fn init_builtins(map: &mut HashMap<String, Node>) {
    map.insert("do".into(), builtin(vec_from!["exprs"], native_do));
    map.insert("def".into(), builtin(vec_from!["vals"], native_def));
    map.insert("set".into(), builtin(vec_from!["vals"], native_set));
    map.insert("log".into(), builtin(vec_from!["msg"], native_log));
    map.insert(
        "try".into(),
        builtin(vec_from!["block", "catch"], native_try),
    );

    map.insert("=".into(), builtin(vec_from!["x", "y"], native_eq));
    map.insert("+".into(), builtin(vec_from!["vals"], native_add));
    map.insert("*".into(), builtin(vec_from!["vals"], native_mul));
    map.insert("-".into(), builtin(vec_from!["x", "y"], native_sub));
    map.insert("/".into(), builtin(vec_from!["x", "y"], native_div));

    panic::set_hook(Box::new(|info| {
        // TODO: Something special to keep track of panic info to promote to catch blocks.
        println!("{:?}", info);
    }));
}

pub fn builtin(args: Vec<String>, f: fn(Node) -> Node) -> Node {
    Node::Block(NodeRef::new(Block {
        params: args,
        env: Node::Nil,
        expr: Node::Native(f),
    }))
}

pub fn loc(env: &Node, name: &str) -> Node {
    if let Node::Dict(env_map_ref) = &env {
        let env_map = &*env_map_ref.borrow();
        match env_map.get(name) {
            Some(result) => result.clone(),
            None => panic!("missing local '{}' in {}", name, env),
        }
    } else {
        panic!("expected dict env, got '{}'", env)
    }
}

pub fn loc_opt(env: Node, name: &str) -> Option<Node> {
    if let Node::Dict(env_map_ref) = &env {
        let env_map = &*env_map_ref.borrow();
        match env_map.get(name) {
            Some(node) => Some(node.clone()),
            None => None,
        }
    } else {
        panic!("expected dict env, got '{}'", env)
    }
}

pub fn loc_str(env: &Node, name: &str) -> String {
    let node = loc(env, name);
    match &node {
        Node::Str(s) => s.clone(),
        _ => panic!(),
    }
}

pub fn loc_num(env: &Node, name: &str) -> f64 {
    let node = loc(env, name);
    match &node {
        Node::Num(x) => *x,
        _ => panic!(),
    }
}

pub fn loc_opt_num(env: Node, name: &str) -> Option<f64> {
    match loc_opt(env.clone(), name) {
        Some(node) => match &node {
            Node::Num(x) => Some(*x),
            _ => panic!(),
        },
        None => None,
    }
}

fn maybe_apply(env: Node, expr: Node) -> Node {
    match expr {
        Node::Block(_) => return apply(env.clone(), vec![expr.clone()]),
        _ => return expr.clone(),
    }
}

fn native_do(env: Node) -> Node {
    let exprs = loc(&env, "exprs");
    match &exprs {
        Node::List(vec_ref) => {
            let mut last = Node::Nil;
            for expr in &*vec_ref.borrow() {
                last = maybe_apply(env.clone(), expr.clone());
            }
            last
        }
        _ => exprs,
    }
}

fn native_def(env: Node) -> Node {
    let this = loc(&env, "@");
    let vals = loc(&env, "vals");
    match &vals {
        Node::Dict(vals_map_ref) => {
            for (k, v) in &*vals_map_ref.borrow() {
                this.def(&k, v.clone());
            }
        }
        _ => panic!("def requires a dict"),
    }
    Node::Nil
}

fn native_set(env: Node) -> Node {
    let this = loc(&env, "@");
    let vals = loc(&env, "vals");
    match &vals {
        Node::Dict(vals_map_ref) => {
            for (k, v) in &*vals_map_ref.borrow() {
                this.set(&k, v.clone());
            }
        }
        _ => panic!("def requires a dict"),
    }
    Node::Nil
}

fn native_log(env: Node) -> Node {
    println!("{}", loc_str(&env, "msg"));
    Node::Nil
}

fn native_try(env: Node) -> Node {
    let block = loc(&env, "block");
    let catch = loc(&env, "catch");
    match (&block, &catch) {
        (Node::Block(_), Node::Block(_)) => {
            let result = panic::catch_unwind(|| {
                apply(env.clone(), vec![block.clone()]);
            });
            if result.is_err() {
                apply(env.clone(), vec![catch.clone()]);
            }
        }
        (_, _) => panic!(),
    }
    Node::Nil
}

fn native_add(env: Node) -> Node {
    let mut total = 0f64;
    addmul_helper(env, |x| total += x);
    Node::Num(total)
}

fn native_mul(env: Node) -> Node {
    let mut total = 1f64;
    addmul_helper(env, |x| total *= x);
    Node::Num(total)
}

fn addmul_helper<F>(env: Node, mut func: F)
where
    F: FnMut(f64),
{
    match &loc(&env, "vals") {
        Node::List(vec_ref) => {
            for val in &*vec_ref.borrow() {
                match val {
                    Node::Num(x) => func(*x),
                    _ => panic!("+ requires numeric values"),
                }
            }
        }
        _ => panic!("expected vals list"),
    }
}

fn native_sub(env: Node) -> Node {
    let x = loc_num(&env, "x");
    let oy = loc_opt_num(env.clone(), "y");
    match oy {
        Some(y) => Node::Num(x - y),
        None => Node::Num(-x),
    }
}

fn native_div(env: Node) -> Node {
    let x = loc_num(&env, "x");
    let oy = loc_opt_num(env.clone(), "y");
    match oy {
        Some(y) => Node::Num(x / y),
        None => Node::Num(1f64 / x),
    }
}
