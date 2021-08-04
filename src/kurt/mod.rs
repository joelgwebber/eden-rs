use std::{collections::HashMap};

use self::expr::{Expr, ERef};

pub mod apply;
pub mod builtins;
pub mod eq;
pub mod eval;
pub mod expr;
pub mod parse;
pub mod print;

mod tests;

pub struct Kurt {
    pub root: Expr,
    builtins: HashMap<&'static str, fn(&Kurt, &Expr) -> Expr>,
    def_dict: Expr,
    def_list: Expr,
    debug: bool,
}

impl Kurt {
    pub fn new() -> Kurt {
        let mut kurt = Kurt {
            builtins: HashMap::new(),
            root: Expr::Dict(ERef::new(HashMap::new())),
            def_dict: Expr::Nil,
            def_list: Expr::Nil,
            debug: false,
        };
        kurt.init_builtins();
        kurt
    }

    pub fn eval_src(&mut self, src: &str) -> Expr {
        let expr = self.parse(src.into());
        self.eval(&self.root, &expr)
    }

    pub fn def(&self, env: &Expr, key: &Expr, val: &Expr) {
        match (env, key) {
            (Expr::Dict(map_ref), Expr::Id(name)) => {
                let map = &mut *map_ref.borrow_mut();
                map.insert(name.clone(), val.clone());
            }

            _ => panic!(),
        }
    }

    pub fn set(&self, env: &Expr, name: &Expr, val: &Expr) {
        match (env, name) {
            (Expr::Dict(_), Expr::Id(s)) => {
                let target = self.find_named(env, s);
                match &target {
                    Expr::Dict(map_ref) => {
                        let map = &mut *map_ref.borrow_mut();
                        map.insert(s.clone(), val.clone());
                    }
                    Expr::Nil => panic!("{} not found", name),
                    _ => panic!("set requires a dict"),
                }
            }

            (Expr::List(vec_ref), Expr::Num(idx)) => {
                let vec = &mut *vec_ref.borrow_mut();
                let expr = vec.get_mut(idx.floor() as usize).unwrap();
                *expr = val.clone();
            }

            (_, _) => panic!(),
        }
    }

    pub fn get(&self, env: &Expr, name: &Expr) -> Expr {
        match (env, &name) {
            (Expr::Dict(_), Expr::Id(name)) => {
                match name.as_str() {
                    // Special case: env refers to the current environment.
                    "env" => env.clone(),
                    _ => {
                        let target = self.find_named(env, name);
                        match &target {
                            Expr::Dict(map_ref) => map_ref.borrow().get(name).unwrap().clone(),
                            Expr::Nil => panic!("{} not found", name),
                            _ => panic!("get requires a dict"),
                        }
                    }
                }
            }

            (Expr::List(vec_ref), Expr::Num(x)) => {
                // TODO: bounds/error checking
                vec_ref.borrow().get(x.floor() as usize).unwrap().clone()
            }

            (Expr::List(_), Expr::Id(name)) => {
                self.get(&self.def_list, &Expr::Id(name.clone()))
            }

            (_, _) => name.clone(),
        }
    }

    fn find_named(&self, target: &Expr, name: &String) -> Expr {
        match target {
            // Check current dict.
            Expr::Dict(map_ref) => {
                if map_ref.borrow().contains_key(name) {
                    return target.clone();
                }

                // Check parent.
                match map_ref.borrow().get("^") {
                    Some(next) => self.find_named(next, name),
                    None => {
                        match &self.def_dict {
                            Expr::Dict(def_map_ref) => {
                                if def_map_ref.borrow().contains_key(name) {
                                    self.def_dict.clone()
                                } else {
                                    Expr::Nil
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }

            _ => Expr::Nil,
        }
    }
}
