use std::{borrow::Borrow, collections::HashMap};

use crate::kurt::{
    expr::{Dict, List},
    Expr,
};

use super::{
    expr::{Apply, Assoc, Block},
    ERef, Kurt,
};

impl Kurt {
    // Evaluates an expr within the given environment.
    pub fn eval(&self, env: &Expr, expr: &Expr) -> Expr {
        if self.debug {
            // println!("eval -- {} :: {}", env.borrow(), expr.borrow());
            println!("eval :: {}", expr);
        }

        match expr {
            // Value types are resolved within their environment.
            Expr::ENil => self.get(env, expr),
            Expr::EBool(_) => self.get(env, expr),
            Expr::ENum(_) => self.get(env, expr),
            Expr::EStr(_) => self.get(env, expr),
            Expr::EId(_) => self.get(env, expr),

            // Quotes evaluate to their wrapped exprs.
            Expr::EQuote(s) => self.quote(env, &*s.borrow()),

            // Unquotes explicitly eval their exprs.
            Expr::EUnquote(s) => self.eval(env, &*s.borrow()),

            // Except blocks, which capture their environment.
            Expr::EBlock(bref) => {
                let b = &*bref.borrow();
                if b.env == Expr::ENil {
                    // Grab the block's environment if one isn't already specified.
                    Expr::EBlock(ERef::new(Block {
                        loc: b.loc.clone(),
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
            Expr::EAssoc(assoc_ref) => {
                let mut map = HashMap::<String, Expr>::new();
                let assoc = &*assoc_ref.borrow();
                for (key_expr, expr) in &assoc.pairs {
                    let key = self.eval(env, &key_expr);
                    if let Expr::EId(s) = &key {
                        map.insert(s.clone(), self.eval(env, expr));
                    } else {
                        panic!("expected id key, got {}", key_expr);
                    }
                }
                Expr::EDict(ERef::new(Dict {
                    loc: assoc.loc.clone(),
                    map: map,
                }))
            }

            // Lists also evaluate to themselves, with their values evaluated.
            // [expr ...] -> [ [eval expr] ...]
            Expr::EList(list_ref) => {
                let list = &*list_ref.borrow();
                let exprs = &list.exprs;
                Expr::EList(ERef::new(List {
                    loc: list.loc.clone(),
                    exprs: exprs
                        .into_iter()
                        .map(|expr| self.eval(env, expr))
                        .collect(),
                }))
            }

            // Dicts evaluate to themselves (their values were already evaluated in their apply form).
            Expr::EDict(_) => expr.clone(),

            // Apply (exprs...)
            Expr::EApply(vec_ref) => {
                let exprs = &vec_ref.borrow().exprs;
                self.apply(env, exprs.clone())
            }

            // Invoke native func.
            Expr::ENative(name) => match self.builtins.get(name) {
                Some(f) => f(self, env.borrow()),
                _ => panic!("unimplemented builtin '{}'", name),
            },
        }
    }

    pub fn quote(&self, env: &Expr, expr: &Expr) -> Expr {
        match &expr {
            Expr::EList(list_ref) => {
                let list = &*list_ref.borrow();
                let exprs = &list.exprs;
                Expr::EList(ERef::new(List {
                    loc: list.loc.clone(),
                    exprs: exprs
                        .into_iter()
                        .map(|expr| self.quote(env, expr))
                        .collect(),
                }))
            }

            Expr::EAssoc(assoc_ref) => {
                let pairs = &assoc_ref.borrow().pairs;
                Expr::EAssoc(ERef::new(Assoc {
                    pairs: pairs
                        .into_iter()
                        .map(|pair| (self.quote(env, &pair.0), self.quote(env, &pair.1)))
                        .collect(),
                    loc: assoc_ref.borrow().loc.clone(),
                }))
            }

            Expr::EApply(apply_ref) => {
                let apply = &*apply_ref.borrow();
                let exprs = &apply.exprs;
                Expr::EApply(ERef::new(Apply {
                    loc: apply.loc.clone(),
                    exprs: exprs
                        .into_iter()
                        .map(|expr| self.quote(env, expr))
                        .collect(),
                }))
            }

            Expr::EUnquote(eref) => self.eval(env, &*eref.borrow()),

            _ => expr.clone(),
        }
    }
}
