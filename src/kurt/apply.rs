use std::{borrow::Borrow, collections::HashMap};

use crate::kurt::expr::{Dict, ERef, List};

use super::{
    expr::{Block, Expr},
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
    pub fn apply(&self, env: &Expr, exprs: Vec<Expr>) -> Expr {
        if self.debug {
            let ls = Expr::EList(ERef::new(List {
                pos: (0, 0),
                exprs: exprs.clone(),
            }));
            // println!("apply -- {} :: {}", env.clone(), ls);
            println!("apply :: {}", ls);
        }

        // () => nil
        if exprs.len() == 0 {
            return Expr::ENil;
        }

        let first = &self.eval(env, exprs.first().unwrap());
        match first {
            // (block expr*) -> positional arg invocation
            Expr::EBlock(_) => self.invoke(env, first.clone(), exprs[1..].to_vec()),

            _ => {
                let result = match exprs.len() {
                    // (expr) -> expr
                    1 => first.clone(),

                    2 => {
                        let second = &self.eval(env, exprs.get(1).unwrap());
                        match second {
                            // (env block) -> eval block in env.
                            Expr::EBlock(block_ref) => {
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
        }
    }

    fn maybe_wrap(&self, slf: Expr, result: Expr) -> Expr {
        match &result {
            Expr::EBlock(blk_ref) => {
                // Capture self reference in block.
                let block = &*blk_ref.borrow();
                Expr::EBlock(ERef::new(Block {
                    pos: block.pos,
                    params: block.params.clone(),
                    env: block.env.clone(),
                    expr: block.expr.clone(),
                    slf: slf.clone(),
                }))
            }
            _ => result,
        }
    }

    fn invoke(&self, env: &Expr, block_expr: Expr, args: Vec<Expr>) -> Expr {
        if self.debug {
            let ls = Expr::EList(ERef::new(List {
                pos: (0, 0),
                exprs: args.clone(),
            }));
            // println!("invoke -- {} :: {}", env.clone(), ls);
            println!("invoke :: {}", ls);
        }

        if let Expr::EBlock(block_ref) = block_expr.borrow() {
            let block = &*block_ref.borrow();
            let mut frame = HashMap::<String, Expr>::new();
            // TODO: validate param/arg match.
            for i in 0..args.len() {
                frame.insert(block.params[i].clone(), self.eval(env, &args[i]));
            }
            let nf = Expr::EDict(ERef::new(Dict {
                pos: (0, 0),
                map: frame,
            }));
            self.apply(env, vec![nf.clone(), block_expr.clone()])
        } else {
            panic!("tried to invoke with non-block expr {}", block_expr)
        }
    }

    fn new_frame(&self, env: &Expr, args: &Expr, blk: &Block) -> Expr {
        let mut new_map = HashMap::<String, Expr>::new();
        if let Expr::EDict(map_ref) = args {
            for (key, expr) in &map_ref.borrow().map {
                new_map.insert(key.clone(), expr.clone());
            }
        }
        new_map.insert(
            "@".to_string(),
            if blk.slf != Expr::ENil {
                blk.slf.clone()
            } else {
                env.clone()
            }
        );
        new_map.insert("^".to_string(), blk.env.clone());
        Expr::EDict(ERef::new(Dict { pos: (0, 0), map: new_map }))
    }
}
