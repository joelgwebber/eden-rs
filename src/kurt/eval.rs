use std::{borrow::Borrow, collections::HashMap};

use crate::kurt::Node;

use super::{node::Block, Kurt, NodeRef};

impl Kurt {
    // Evaluates a node within the given environment.
    pub fn eval(&self, env: &Node, expr: &Node) -> Node {
        if self.debug {
            // println!("eval -- {} :: {}", env.borrow(), expr.borrow());
            println!("eval :: {}", expr);
        }

        match expr {
            // Value types are resolved within their environment.
            Node::Nil => self.get(env, expr),
            Node::Bool(_) => self.get(env, expr),
            Node::Num(_) => self.get(env, expr),
            Node::Str(_) => self.get(env, expr),
            Node::Id(_) => self.get(env, expr),

            // Quotes evaluate to their wrapped nodes.
            Node::Quote(s) => self.quote(env, &*s.borrow()),

            // Unquotes explicitly eval their nodes.
            Node::Unquote(s) => self.eval(env, &*s.borrow()),

            // Except blocks, which capture their environment.
            Node::Block(bref) => {
                let b = &*bref.borrow();
                if b.env == Node::Nil {
                    // Grab the block's environment if one isn't already specified.
                    Node::Block(NodeRef::new(Block {
                        params: b.params.clone(),
                        expr: b.expr.clone(),
                        env: env.clone(),
                        slf: b.slf.clone(),
                    }))
                } else {
                    expr.clone()
                }
            }

            // Associations evaluate to dicts, with their keys and values evaluated.
            // { expr expr ... } -> { [eval expr] : [eval expr] ... }
            Node::Assoc(vec_ref) => {
                let mut map = HashMap::<String, Node>::new();
                for (key_node, node) in &*vec_ref.borrow() {
                    let key = self.eval(env, key_node);
                    if let Node::Id(s) = &key {
                        map.insert(s.clone(), self.eval(env, node));
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
                        .map(|node| self.eval(env, node))
                        .collect(),
                ))
            }

            // Dicts evaluate to themselves (their values were already evaluated in their apply form).
            Node::Dict(_) => expr.clone(),

            // Apply (exprs...)
            Node::Apply(vec_ref) => {
                let exprs = &*vec_ref.borrow();
                self.apply(env, exprs.clone())
            }

            // Invoke native func.
            Node::Native(name) => match self.builtins.get(name) {
                Some(f) => f(self, env.borrow()),
                _ => panic!("unimplemented builtin '{}'", name),
            },
        }
    }

    pub fn quote(&self, env: &Node, expr: &Node) -> Node {
        match &expr {
            Node::List(vec_ref) => {
                let vec = &*vec_ref.borrow();
                Node::List(NodeRef::new(
                    vec.into_iter()
                        .map(|node| self.quote(env, node))
                        .collect(),
                ))
            }

            Node::Assoc(vec_ref) => {
                let vec = &*vec_ref.borrow();
                Node::Assoc(NodeRef::new(
                    vec.into_iter()
                        .map(|pair| {
                            (
                                self.quote(env, &pair.0),
                                self.quote(env, &pair.1),
                            )
                        })
                        .collect(),
                ))
            }

            Node::Apply(vec_ref) => {
                let vec = &*vec_ref.borrow();
                Node::Apply(NodeRef::new(
                    vec.into_iter()
                        .map(|node| self.quote(env, node))
                        .collect(),
                ))
            }

            Node::Unquote(node_ref) => self.eval(env, &*node_ref.borrow()),

            _ => expr.clone(),
        }
    }
}
