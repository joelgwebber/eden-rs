use std::cell::RefCell;
use std::collections::HashMap;
use std::panic::{RefUnwindSafe, UnwindSafe};

use gc::Finalize;
use gc::Trace;
use velcro::hash_map;

use crate::kurt::expr::List;

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
    debug: bool,

    builtins: HashMap<&'static str, fn(&Kurt, &Expr) -> Expr>,
    def_dict: Expr,
    def_list: Expr,

    exception: RefCell<Option<Expr>>,
}

// Needed for the use of kurt refs in the panic handler.
impl UnwindSafe for Kurt {}
impl RefUnwindSafe for Kurt {}

#[derive(Debug, Default, Trace, Finalize, PartialEq, Clone)]
pub struct Loc {
    file: String,
    name: String,
    pos: (usize, usize),
}

impl Kurt {
    pub fn new() -> Kurt {
        let mut kurt = Kurt {
            builtins: HashMap::new(),
            root: Expr::EDict(ERef::new(Dict {
                map: HashMap::new(),
                loc: Loc::default(),
            })),
            def_dict: Expr::ENil,
            def_list: Expr::ENil,
            debug: false,
            exception: RefCell::new(None),
        };
        kurt.init_builtins();
        kurt
    }

    pub fn eval_src(&self, name: &str, src: &str) -> Expr {
        let expr = self.parse(name.into(), src.into());
        self.eval(&self.root.clone(), &expr)
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
                            Expr::ENil => self.throw(env, format!("'{}' not found", name)),
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

    pub fn throw(&self, env: &Expr, msg: String) -> ! {
        let mut map = HashMap::<String, Expr>::new();
        map.insert("message".to_string(), Expr::EStr(msg));
        let mut stack = Vec::<Expr>::new();

        let mut cur = Some(env.clone());
        while let Some(expr) = cur {
            match expr.loc() {
                Some(loc) => stack.push(Expr::EDict(ERef::new(Dict {
                    loc: Loc::default(),
                    map: hash_map! {
                        "file".into(): Expr::EStr(loc.file.clone()),
                        "name".into(): Expr::EStr(loc.name.clone()),
                        "pos".into(): Expr::EList(ERef::new(List{
                            loc: Loc::default(),
                            exprs: vec![Expr::ENum(loc.pos.0 as f64), Expr::ENum(loc.pos.1 as f64)]
                        })),
                    },
                }))),
                None => (),
            }
            cur = expr.caller();
        }

        map.insert(
            "stack".to_string(),
            Expr::EList(ERef::new(List {
                loc: Loc::default(),
                exprs: stack,
            })),
        );

        self.exception.replace(Some(Expr::EDict(ERef::new(Dict {
            loc: Loc::default(),
            map: map,
        }))));
        panic!("exception")
    }
}
