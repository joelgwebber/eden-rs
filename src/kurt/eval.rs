use std::{borrow::Borrow, collections::HashMap};

use crate::kurt::Node;

use super::NodeRef;

// Evaluates a node within the given environment.
pub fn eval(env: Node, expr: Node) -> Node {
    println!("eval -- {} :: {}", env.borrow(), expr.borrow());
    match &expr {
        // Most expressions just evaluate to themselves.
        Node::Nil => expr.clone(),
        Node::Num(_) => expr.clone(),
        Node::Bool(_) => expr.clone(),
        Node::Str(_) => expr.clone(),
        Node::Sym(_) => expr.clone(),
        Node::Block(_) => expr.clone(),
        Node::Native(f) => f(env),

        // Except identifiers, which are always looked up in the current scope.
        // ident -> env:ident
        Node::Id(s) => {
            match s.as_str() {
                // Special cases:
                // - env refers to the current environment.
                // - $ evaluates to a list exec marker.
                "env" => env.clone(),
                "$" => Node::Exec,
                _ => match env.borrow().lookup_node(expr.clone()) {
                    Ok(n) => n,
                    Err(msg) => panic!("lookup failed: {}", msg),
                },
            }
        }

        Node::Exec => {
            panic!("cannot execute here") // TODO: Can this even happen?
        }

        // Dictionaries evaluate to themselves, but with their values evaluated.
        // { key:expr... } -> { key:[eval expr]... }
        Node::Dict(map_ref) => {
            let mut map = HashMap::<String, Node>::new();
            for (key, node) in &*map_ref.borrow() {
                map.insert(key.clone(), eval(env.clone(), node.clone()));
            }
            Node::Dict(NodeRef::new(map))
        }

        // Lists also evaluate to themselves.
        // [exprs...] -> [eval [exprs...]]
        Node::List(vec_ref) => {
            let vec = &*vec_ref.borrow();
            match vec.len() {
                // [] -> Empty list
                0 => Node::List(NodeRef::new(vec![])),

                _ => {
                    let first = eval(env.clone(), vec.first().unwrap().clone());
                    match first {
                        // Except for the special case of [$ ...], which is a special form that means "exec".
                        // Note that we don't evaluate the other expressions here. That's exec()'s job.
                        Node::Exec => exec(env.clone(), vec[1..].to_vec()),

                        // Regular list; evaluate the rest of the items.
                        _ => {
                            let mut exprs = vec![first];
                            vec[1..]
                                .into_iter()
                                .for_each(|node| exprs.push(eval(env.clone(), node.clone())));
                            Node::List(NodeRef::new(exprs))
                        }
                    }
                }
            }
        }
    }
}

// Executes a list -- (foo bar baz ...) or [$ foo bar baz].
// This covers a number of cases:
// - Block execution with args   -- (block arg0 arg1 ...)
// - Block execution in env      -- ({env} block)
// - Expr execution in env       -- ({env} expr)
// - Dict field access           -- ({dict} key)
// - List field access           -- ([list] idx)
// - Single expression execution -- (expr) => expr
// - Empty list execution        -- () => nil
//
pub fn exec(env: Node, exprs: Vec<Node>) -> Node {
    let ls = Node::List(NodeRef::new(exprs.clone()));
    println!("exec -- {} :: {}", env.clone(), ls);

    match exprs.len() {
        // () -> nil
        0 => Node::Nil,

        _ => {
            let first = exprs.first().unwrap().borrow();
            match first {
                // (block expr*) -> positional arg invocation
                Node::Block(_) => invoke(exprs[1..].to_vec()),

                // TODO: (list idx) -> lookup item
                Node::List(_) => unimplemented!(),

                // (dict expr) ->
                Node::Dict(map_ref) => {
                    if exprs.len() == 1 {
                        panic!("unable to exec dict")
                    } else if exprs.len() > 2 {
                        panic!("dict can only be exec'd with a single expr")
                    }

                    let second = exprs.get(1).unwrap();
                    match second {
                        // (dict block) -> apply dict to block expr
                        Node::Block(block_ref) => {
                            let params = copy_map(&*map_ref.borrow());
                            eval(
                                Node::Dict(NodeRef::new(params)),
                                block_ref.borrow().1.clone(),
                            )
                        }

                        // (dict expr) -> eval expr in env
                        _ => eval(first.clone(), second.clone()),
                    }
                }

                _ => panic!("unable to exec {:?}", first.to_string()),
            }
        }
    }
}

fn invoke(vec: Vec<Node>) -> Node {
    Node::Nil
}

fn copy_map(map: &HashMap<String, Node>) -> HashMap<String, Node> {
    let mut new_map = HashMap::<String, Node>::new();
    for (key, node) in map {
        new_map.insert(key.clone(), node.clone());
    }
    new_map
}
