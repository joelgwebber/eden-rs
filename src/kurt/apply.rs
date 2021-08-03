use std::{borrow::Borrow, collections::HashMap};

use crate::kurt::node::NodeRef;

use super::{
    node::{Block, Node},
    Kurt,
};

impl Kurt {
    // Applies a list -- (foo bar baz ...).
    // This covers a number of cases:
    // - Apply block to args     -- (block arg0 arg1 ...) => ({env} block)
    // - Apply block to env      -- ({env} block) => eval(env, block-expr)
    // - Apply expr to env       -- ({env} expr) => eval(env, expr)
    // - Apply single expression -- (expr) => expr
    // - Apply empty list        -- () => nil
    //
    pub fn apply(&self, env: &Node, exprs: Vec<Node>) -> Node {
        if self.debug {
            let ls = Node::List(NodeRef::new(exprs.clone()));
            // println!("apply -- {} :: {}", env.clone(), ls);
            println!("apply :: {}", ls);
        }

        // () => nil
        if exprs.len() == 0 {
            return Node::Nil;
        }

        let first = &self.eval(env, exprs.first().unwrap());
        let result = match first {
            // (block expr*) -> positional arg invocation
            Node::Block(_) => self.invoke(env, first.clone(), exprs[1..].to_vec()),

            _ => {
                let result = match exprs.len() {
                    // (expr) -> expr
                    1 => first.clone(),

                    2 => {
                        let second = &self.eval(env, exprs.get(1).unwrap());
                        match second {
                            // (env block) -> eval block in env.
                            Node::Block(block_ref) => {
                                let block = &*block_ref.borrow();
                                let frame = self.new_frame(&env, &first, &block);
                                self.eval(&frame, &block.expr)
                            }

                            // (env expr) -> eval expr in env
                            _ => self.eval(first, second),
                        }
                    }

                    _ => panic!("apply allows no more than 2 arguments"),
                };
                self.maybe_wrap(first.clone(), result)
            }
        };

        if self.debug {
            println!("  => {}", result.borrow());
        }
        result
    }

    fn maybe_wrap(&self, slf: Node, result: Node) -> Node {
        match &result {
            Node::Block(blk_ref) => {
                // Capture self reference in block.
                let block = &*blk_ref.borrow();
                Node::Block(NodeRef::new(Block {
                    params: block.params.clone(),
                    env: block.env.clone(),
                    expr: block.expr.clone(),
                    slf: slf.clone(),
                }))
            }
            _ => result,
        }
    }

    fn invoke(&self, env: &Node, block_node: Node, args: Vec<Node>) -> Node {
        if self.debug {
            let ls = Node::List(NodeRef::new(args.clone()));
            // println!("invoke -- {} :: {}", env.clone(), ls);
            println!("invoke :: {}", ls);
        }

        if let Node::Block(block_ref) = block_node.borrow() {
            let block = &*block_ref.borrow();
            let mut frame = HashMap::<String, Node>::new();
            // TODO: validate param/arg match.
            for i in 0..args.len() {
                frame.insert(block.params[i].clone(), self.eval(env, &args[i]));
            }
            let nf = Node::Dict(NodeRef::new(frame));
            self.apply(env, vec![nf.clone(), block_node.clone()])
        } else {
            panic!("tried to invoke with non-block node {}", block_node)
        }
    }

    fn new_frame(&self, env: &Node, args: &Node, blk: &Block) -> Node {
        let mut new_map = HashMap::<String, Node>::new();
        if let Node::Dict(map_ref) = args {
            for (key, node) in &*map_ref.borrow() {
                new_map.insert(key.clone(), node.clone());
            }
        }
        new_map.insert(
            "@".to_string(),
            if blk.slf != Node::Nil {
                blk.slf.clone()
            } else {
                env.clone()
            },
        );
        new_map.insert("^".to_string(), blk.env.clone());
        Node::Dict(NodeRef::new(new_map))
    }
}
