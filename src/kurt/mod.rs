use std::cell::RefCell;
use std::collections::HashMap;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::{fs, panic};

use gc::Finalize;
use gc::Trace;
use velcro::hash_map;

use crate::kurt::expr::{_id, _list, _num, _str};

use self::expr::{ERef, Expr, _dict, _NIL};

pub mod apply;
pub mod eval;
pub mod expr;
pub mod parse;
pub mod print;

mod lib;

pub struct Kurt {
    pub root: Expr,
    debug: bool,

    builtins: HashMap<&'static str, fn(&Kurt, &Expr) -> Expr>,
    def_num: Expr,
    def_dict: Expr,
    def_list: Expr,

    exception: RefCell<Option<Expr>>,
}

#[derive(Debug, Default, Trace, Finalize, PartialEq, Clone)]
pub struct Loc {
    file: String,
    name: String,
    pos: (usize, usize),
}

// Needed for the use of kurt refs in the panic handler.
impl UnwindSafe for Kurt {}
impl RefUnwindSafe for Kurt {}

impl Kurt {
    pub fn new() -> Kurt {
        let mut kurt = Kurt {
            builtins: HashMap::new(),
            root: _dict(HashMap::new()),
            def_num: _NIL,
            def_dict: _NIL,
            def_list: _NIL,
            debug: false,
            exception: RefCell::new(None),
        };
        kurt.init_lib();
        kurt
    }

    pub fn test_file(filename: &str) {
        println!("-- {}", filename);
        let kurt = Kurt::new();
        kurt.eval_file(filename);
    }

    pub fn eval_src(&self, env: &Expr, name: &str, src: &str) -> Expr {
        match panic::catch_unwind(|| {
            let expr = self.parse(name.into(), src.into());
            self.eval(env, &expr)
        }) {
            Ok(expr) => expr,
            Err(_) => match self.exception.replace(None) {
                Some(expr) => {
                    self.apply(&self.root, vec![_id("print-exception"), expr]);
                    _NIL
                }
                None => _NIL,
            },
        }
    }

    pub fn eval_file(&self, filename: &str) {
        self.eval_src(
            &self.root,
            filename,
            fs::read_to_string(filename)
                .expect("cannot read test file")
                .as_str(),
        );
    }

    pub fn def(&self, env: &Expr, key: &Expr, val: &Expr) {
        match (env, key) {
            (Expr::EDict(dict_ref), Expr::EId(name)) => {
                let map = &mut dict_ref.borrow_mut().map;
                map.insert(name.clone(), val.clone());
            }
            _ => self.throw(env, "def requires dict :id".to_string()),
        }
    }

    pub fn set(&self, env: &Expr, name: &Expr, val: &Expr) {
        match (env, name) {
            (Expr::EDict(_), Expr::EId(s)) => {
                let target = self.find_scope(env, s);
                match &target {
                    Some(Expr::EDict(dict_ref)) => {
                        let map = &mut dict_ref.borrow_mut().map;
                        map.insert(s.clone(), val.clone());
                    }
                    _ => self.throw(env, format!("{} not found", name)),
                }
            }

            (Expr::EList(list_ref), Expr::ENum(idx)) => {
                let list = &mut *list_ref.borrow_mut();
                let expr = list.exprs.get_mut(idx.floor() as usize).unwrap();
                *expr = val.clone();
            }

            (_, _) => self.throw(
                env,
                format!(
                    "set requires (dict id) or (list num); got ({} {})",
                    env, name
                ),
            ),
        }
    }

    pub fn get(&self, env: &Expr, name: &Expr) -> Expr {
        match (env, &name) {
            (Expr::EDict(_), Expr::EId(name)) => {
                match name.as_str() {
                    // Special case: env refers to the current environment.
                    "env" => env.clone(),
                    _ => {
                        let target = self.find_scope(env, name);
                        match &target {
                            Some(Expr::EDict(dict_ref)) => {
                                dict_ref.borrow().map.get(name).unwrap().clone()
                            }
                            _ => self.throw(env, format!("'{}' not found", name)),
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

            (Expr::EList(_), Expr::EId(name)) => self.get(&self.def_list, &_id(name)),
            (Expr::ENum(_), Expr::EId(name)) => self.get(&self.def_num, &_id(name)),

            (_, _) => name.clone(),
        }
    }

    pub fn find_scope(&self, target: &Expr, name: &String) -> Option<Expr> {
        match target {
            // Check current dict.
            Expr::EDict(dict_ref) => {
                if dict_ref.borrow().map.contains_key(name) {
                    return Some(target.clone());
                }

                // Check parent.
                match dict_ref.borrow().map.get("^") {
                    Some(next) => self.find_scope(next, name),
                    None => match &self.def_dict {
                        Expr::EDict(def_map_ref) => {
                            if def_map_ref.borrow().map.contains_key(name) {
                                Some(self.def_dict.clone())
                            } else {
                                None
                            }
                        }
                        _ => unreachable!(),
                    },
                }
            }

            _ => None,
        }
    }

    pub fn throw(&self, env: &Expr, msg: String) -> ! {
        // TODO: There's gotta be a way to make this less shitty.
        let mut map = HashMap::<String, Expr>::new();
        map.insert("message".to_string(), _str(msg.as_str()));
        let mut stack = Vec::<Expr>::new();

        let mut cur = Some(env.clone());
        while let Some(expr) = cur {
            match expr.loc() {
                Some(loc) => stack.push(_dict(hash_map! {
                    "file".into(): _str(loc.file.as_str()),
                    "name".into(): _str(loc.name.as_str()),
                    "pos".into(): _list(vec![_num(loc.pos.0 as f64), _num(loc.pos.1 as f64)]),
                })),
                None => (),
            }
            cur = expr.caller();
        }

        map.insert("stack".to_string(), _list(stack));
        self.exception.replace(Some(_dict(map)));
        panic!("[exception]")
    }
}
