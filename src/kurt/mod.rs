use std::collections::HashMap;

use self::expr::{Dict, ERef, Expr};

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
            root: Expr::EDict(ERef::new(Dict {
                map: HashMap::new(),
            })),
            def_dict: Expr::ENil,
            def_list: Expr::ENil,
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
            (Expr::EDict(dict_ref), Expr::EId(name)) => {
                let map = &mut dict_ref.borrow_mut().map;
                map.insert(name.clone(), val.clone());
            }

            _ => panic!(),
        }
    }

    pub fn set(&self, env: &Expr, name: &Expr, val: &Expr) {
        match (env, name) {
            (Expr::EDict(_), Expr::EId(s)) => {
                let target = self.find_named(env, s);
                match &target {
                    Expr::EDict(dict_ref) => {
                        let map = &mut dict_ref.borrow_mut().map;
                        map.insert(s.clone(), val.clone());
                    }
                    Expr::ENil => panic!("{} not found", name),
                    _ => panic!("set requires a dict"),
                }
            }

            (Expr::EList(list_ref), Expr::ENum(idx)) => {
                let list = &mut *list_ref.borrow_mut();
                let expr = list.exprs.get_mut(idx.floor() as usize).unwrap();
                *expr = val.clone();
            }

            (_, _) => panic!(),
        }
    }

    pub fn get(&self, env: &Expr, name: &Expr) -> Expr {
        match (env, &name) {
            (Expr::EDict(_), Expr::EId(name)) => {
                match name.as_str() {
                    // Special case: env refers to the current environment.
                    "env" => env.clone(),
                    _ => {
                        let target = self.find_named(env, name);
                        match &target {
                            Expr::EDict(dict_ref) => {
                                dict_ref.borrow().map.get(name).unwrap().clone()
                            }
                            Expr::ENil => panic!("{} not found", name),
                            _ => panic!("get requires a dict"),
                        }
                    }
                }
            }

            (Expr::EList(list_ref), Expr::ENum(x)) => {
                // TODO: bounds/error checking
                list_ref
                    .borrow()
                    .exprs
                    .get(x.floor() as usize)
                    .unwrap()
                    .clone()
            }

            (Expr::EList(_), Expr::EId(name)) => self.get(&self.def_list, &Expr::EId(name.clone())),

            (_, _) => name.clone(),
        }
    }

    fn find_named(&self, target: &Expr, name: &String) -> Expr {
        match target {
            // Check current dict.
            Expr::EDict(dict_ref) => {
                if dict_ref.borrow().map.contains_key(name) {
                    return target.clone();
                }

                // Check parent.
                match dict_ref.borrow().map.get("^") {
                    Some(next) => self.find_named(next, name),
                    None => match &self.def_dict {
                        Expr::EDict(def_map_ref) => {
                            if def_map_ref.borrow().map.contains_key(name) {
                                self.def_dict.clone()
                            } else {
                                Expr::ENil
                            }
                        }
                        _ => unreachable!(),
                    },
                }
            }

            _ => Expr::ENil,
        }
    }
}
