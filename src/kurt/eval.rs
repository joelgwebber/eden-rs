use std::{borrow::{Borrow, BorrowMut}, collections::HashMap};

use crate::kurt::{Block, Node};

use super::NodeRef;

const DEBUG: bool = false;

// Evaluates a node within the given environment.
pub fn eval(env: Node, expr: Node) -> Node {
    if DEBUG {
        // println!("eval -- {} :: {}", env.borrow(), expr.borrow());
        println!("eval :: {}", expr.borrow());
    }

    match &expr {
        // Most expressions just evaluate to themselves.
        Node::Nil => expr.clone(),
        Node::Num(_) => expr.clone(),
        Node::Bool(_) => expr.clone(),
        Node::Str(_) => expr.clone(),
        Node::Dict(_) => expr.clone(),

        // Symbols evaluate to identifiers.
        Node::Sym(s) => Node::Id(s.clone()),

        // Except blocks, which capture their environment.
        Node::Block(bref) => {
            let b = &*bref.borrow();
            if b.env == Node::Nil {
                // Grab the block's environment if one isn't already specified.
                Node::Block(NodeRef::new(Block {
                    params: b.params.clone(),
                    env: env.clone(),
                    expr: b.expr.clone(),
                }))
            } else {
                expr.clone()
            }
        }

        // And identifiers, which are always looked up in the current scope.
        // ident -> env:ident
        Node::Id(s) => {
            match s.as_str() {
                // Special case: env refers to the current environment.
                "env" => env.clone(),
                _ => env.borrow().get_node(expr.clone()),
            }
        }

        // Dictionary defs evaluate to dicts, with their keys and values evaluated.
        // { expr expr ... } -> { [eval expr] : [eval expr] ... }
        Node::DictDef(map_ref) => {
            let mut map = HashMap::<String, Node>::new();
            for (key_node, node) in &*map_ref.borrow() {
                let key = eval(env.clone(), key_node.clone());
                if let Node::Id(s) = &key {
                    map.insert(s.clone(), eval(env.clone(), node.clone()));
                } else {
                    panic!("expected id key, got {}", key_node);
                }
            }
            Node::Dict(NodeRef::new(map))
        }

        // Lists also evaluate to themselves, with their values evaluated.
        // [expr ...] -> [ [eval expr] ...]
        Node::List(vec_ref) => {
            let vec = &*vec_ref.borrow();
            Node::List(NodeRef::new(
                vec.into_iter()
                    .map(|node| eval(env.clone(), node.clone()))
                    .collect(),
            ))
        }

        // Apply list:
        // (exprs...)
        Node::Apply(vec_ref) => {
            let exprs = &*vec_ref.borrow();
            apply(env.clone(), exprs.clone())
        }

        // Invoke native func.
        Node::Native(f) => f(env),
    }
}

// Applies a list -- (foo bar baz ...) or [$ foo bar baz ...].
// This covers a number of cases:
// - Apply Block to args     -- (block arg0 arg1 ...)
// - Apply Block to env      -- ({env} block)
// - Apply expr to env       -- ({env} expr)
// - Apply single expression -- (expr) => expr
// - Apply empty list        -- () => nil
// - Dict field access       -- ({dict} sym)
// - List field access       -- ([list] idx)
//
pub fn apply(env: Node, exprs: Vec<Node>) -> Node {
    if DEBUG {
        let ls = Node::List(NodeRef::new(exprs.clone()));
        // println!("apply -- {} :: {}", env.clone(), ls);
        println!("apply :: {}", ls);
    }

    if exprs.len() == 0 {
        return Node::Nil;
    }

    let first = &eval(env.clone(), exprs.first().unwrap().clone());
    match first {
        // (block expr*) -> positional arg invocation
        Node::Block(_) => invoke(env.clone(), first.clone(), exprs[1..].to_vec()),

        // (dict ...) ->
        Node::Dict(map_ref) => {
            if exprs.len() != 2 {
                panic!("dict can only be applied with a single expr")
            }

            let second = &eval(env.clone(), exprs.get(1).unwrap().clone());
            match second {
                // (dict block) -> apply block expr to dict
                Node::Block(block_ref) => {
                    let block = &*block_ref.borrow();
                    let params =
                        new_frame(env.clone(), block.env.clone(), (*map_ref.borrow()).clone());
                    eval(Node::Dict(NodeRef::new(params)), block.expr.clone())
                }

                // (dict sym) -> lookup item
                Node::Sym(sym) => eval(first.clone(), Node::Id(sym.clone())),

                // (dict expr) -> eval expr in env
                _ => eval(first.clone(), second.clone()),
            }
        }

        // (list idx) -> lookup item
        Node::List(vec_ref) => {
            if exprs.len() != 2 {
                panic!("lists can only be applied with a single expr")
            }

            let second = &eval(env.clone(), exprs.get(1).unwrap().clone());
            match second {
                Node::Num(idx) => (&*vec_ref.borrow()).get(*idx as usize).unwrap().clone(),
                _ => panic!(),
            }
        }

        _ => {
            if exprs.len() == 1 {
                first.clone()
            } else {
                panic!("unable to apply ({} ...)", first)
            }
        }
    }
}

fn invoke(env: Node, block_node: Node, args: Vec<Node>) -> Node {
    if DEBUG {
        let ls = Node::List(NodeRef::new(args.clone()));
        // println!("invoke -- {} :: {}", env.clone(), ls);
        println!("invoke :: {}", ls);
    }

    if let Node::Block(block_ref) = block_node.borrow() {
        let block = &*block_ref.borrow();
        let mut frame = HashMap::<String, Node>::new();
        // TODO: validate param/arg match.
        for i in 0..args.len() {
            frame.insert(block.params[i].clone(), eval(env.clone(), args[i].clone()));
        }
        let nf = Node::Dict(NodeRef::new(frame));
        apply(env.clone(), vec![nf.clone(), block_node.clone()])
    } else {
        panic!("tried to invoke with non-block node {}", block_node)
    }
}

fn new_frame(env: Node, sup: Node, map: HashMap<String, Node>) -> HashMap<String, Node> {
    let mut new_map = HashMap::<String, Node>::new();
    for (key, node) in map {
        new_map.insert(key.clone(), node.clone());
    }
    new_map.insert("@".to_string(), env.clone());
    new_map.insert("^".to_string(), sup.clone());
    new_map
}
