use std::{borrow::Borrow, collections::HashMap};

use crate::kurt::Expr;

use super::{expr::Block, Kurt, ERef};

impl Kurt {
    // Evaluates an expr within the given environment.
    pub fn eval(&self, env: &Expr, expr: &Expr) -> Expr {
        if self.debug {
            // println!("eval -- {} :: {}", env.borrow(), expr.borrow());
            println!("eval :: {}", expr);
        }

        match expr {
            // Value types are resolved within their environment.
            Expr::Nil => self.get(env, expr),
            Expr::Bool(_) => self.get(env, expr),
            Expr::Num(_) => self.get(env, expr),
            Expr::Str(_) => self.get(env, expr),
            Expr::Id(_) => self.get(env, expr),

            // Quotes evaluate to their wrapped exprs.
            Expr::Quote(s) => self.quote(env, &*s.borrow()),

            // Unquotes explicitly eval their exprs.
            Expr::Unquote(s) => self.eval(env, &*s.borrow()),

            // Except blocks, which capture their environment.
            Expr::Block(bref) => {
                let b = &*bref.borrow();
                if b.env == Expr::Nil {
                    // Grab the block's environment if one isn't already specified.
                    Expr::Block(ERef::new(Block {
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
            Expr::Assoc(vec_ref) => {
                let mut map = HashMap::<String, Expr>::new();
                for (key_expr, expr) in &*vec_ref.borrow() {
                    let key = self.eval(env, key_expr);
                    if let Expr::Id(s) = &key {
                        map.insert(s.clone(), self.eval(env, expr));
                    } else {
                        panic!("expected id key, got {}", key_expr);
                    }
                }
                Expr::Dict(ERef::new(map))
            }

            // Lists also evaluate to themselves, with their values evaluated.
            // [expr ...] -> [ [eval expr] ...]
            Expr::List(vec_ref) => {
                let vec = &*vec_ref.borrow();
                Expr::List(ERef::new(
                    vec.into_iter()
                        .map(|expr| self.eval(env, expr))
                        .collect(),
                ))
            }

            // Dicts evaluate to themselves (their values were already evaluated in their apply form).
            Expr::Dict(_) => expr.clone(),

            // Apply (exprs...)
            Expr::Apply(vec_ref) => {
                let exprs = &*vec_ref.borrow();
                self.apply(env, exprs.clone())
            }

            // Invoke native func.
            Expr::Native(name) => match self.builtins.get(name) {
                Some(f) => f(self, env.borrow()),
                _ => panic!("unimplemented builtin '{}'", name),
            },
        }
    }

    pub fn quote(&self, env: &Expr, expr: &Expr) -> Expr {
        match &expr {
            Expr::List(vec_ref) => {
                let vec = &*vec_ref.borrow();
                Expr::List(ERef::new(
                    vec.into_iter()
                        .map(|expr| self.quote(env, expr))
                        .collect(),
                ))
            }

            Expr::Assoc(vec_ref) => {
                let vec = &*vec_ref.borrow();
                Expr::Assoc(ERef::new(
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

            Expr::Apply(vec_ref) => {
                let vec = &*vec_ref.borrow();
                Expr::Apply(ERef::new(
                    vec.into_iter()
                        .map(|expr| self.quote(env, expr))
                        .collect(),
                ))
            }

            Expr::Unquote(eref) => self.eval(env, &*eref.borrow()),

            _ => expr.clone(),
        }
    }
}
