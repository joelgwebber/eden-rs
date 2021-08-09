use std::{
    panic::{self},
    vec,
};

use velcro::{hash_map, vec_from};

use crate::kurt::{Expr, Loc, expr::Dict};

use super::{expr::Block, ERef, Kurt};

impl Kurt {
    pub fn init_builtins(&mut self) {
        // Built-in functions.
        self.add_builtin("do", &vec_from!["exprs"], Kurt::native_do);
        self.add_builtin("def", &vec_from!["name", "value"], Kurt::native_def);
        self.add_builtin("let", &vec_from!["vars", "expr"], Kurt::native_let);

        self.add_builtin("set".into(), &vec_from!["name", "value"], Kurt::native_set);
        self.add_builtin("log".into(), &vec_from!["msg"], Kurt::native_log);
        self.add_builtin("try".into(), &vec_from!["block", "catch"], Kurt::native_try);

        self.add_builtin("=".into(), &vec_from!["x", "y"], Kurt::native_eq);
        self.add_builtin("+".into(), &vec_from!["vals"], Kurt::native_add);
        self.add_builtin("*".into(), &vec_from!["vals"], Kurt::native_mul);
        self.add_builtin("-".into(), &vec_from!["x", "y"], Kurt::native_sub);
        self.add_builtin("/".into(), &vec_from!["x", "y"], Kurt::native_div);

        self.add_builtin("not", &vec_from!["x"], Kurt::native_not);

        // Default implementation dicts.
        self.def_dict = Expr::EDict(ERef::new(Dict {
            loc: Loc::default(),
            map: hash_map! {
                "set".into(): self.builtin("set", &vec_from!["name", "value"]),
                "def".into(): self.builtin("def", &vec_from!["name", "value"]),
            },
        }));
        self.def_list = Expr::EDict(ERef::new(Dict {
            loc: Loc::default(),
            map: hash_map! {
                "set".into(): self.builtin("set", &vec_from!["name", "value"]),
            },
        }));

        // Override panic handler to suppress automatic stack traces.
        panic::set_hook(Box::new(|_| {}));
    }

    pub fn builtin(&self, name: &'static str, args: &Vec<String>) -> Expr {
        Expr::EBlock(ERef::new(Block {
            loc: Loc {
                file: "".to_string(),
                name: name.to_string(),
                pos: (0, 0),
            },
            params: args.clone(),
            expr: Expr::ENative(name),
            env: Expr::ENil,
            slf: Expr::ENil,
        }))
    }

    pub fn add_builtin(
        &mut self,
        name: &'static str,
        args: &Vec<String>,
        f: fn(&Kurt, &Expr) -> Expr,
    ) {
        self.builtins.insert(name, f);
        self.def(
            &self.root.clone(),
            &Expr::EId(name.to_string()),
            &self.builtin(name, args).clone(),
        );
    }

    pub fn loc_expr(&self, env: &Expr, name: &str) -> Expr {
        if let Expr::EDict(env_map_ref) = &env {
            let env_map = &env_map_ref.borrow().map;
            match env_map.get(name) {
                Some(result) => result.clone(),
                None => self.throw(&env, format!("missing local '{}' in {}", name, env)),
            }
        } else {
            panic!("expected dict env, got '{}'", env)
        }
    }

    pub fn loc_opt(&self, env: &Expr, name: &str) -> Option<Expr> {
        if let Expr::EDict(env_map_ref) = &env {
            let env_map = &env_map_ref.borrow().map;
            match env_map.get(name) {
                Some(expr) => Some(expr.clone()),
                None => None,
            }
        } else {
            panic!("expected dict env, got '{}'", env)
        }
    }

    pub fn loc_str(&self, env: &Expr, name: &str) -> String {
        let expr = self.loc_expr(env, name);
        match &expr {
            Expr::EStr(s) => s.clone(),
            _ => panic!(),
        }
    }

    pub fn loc_num(&self, env: &Expr, name: &str) -> f64 {
        let expr = self.loc_expr(env, name);
        match &expr {
            Expr::ENum(x) => *x,
            _ => panic!(),
        }
    }

    pub fn loc_bool(&self, env: &Expr, name: &str) -> bool {
        let expr = self.loc_expr(env, name);
        match &expr {
            Expr::EBool(x) => *x,
            _ => panic!(),
        }
    }

    pub fn loc_opt_num(&self, env: &Expr, name: &str) -> Option<f64> {
        match self.loc_opt(env, name) {
            Some(expr) => match &expr {
                Expr::ENum(x) => Some(*x),
                _ => panic!(),
            },
            None => None,
        }
    }

    fn native_do(&self, env: &Expr) -> Expr {
        let exprs = self.loc_expr(&env, "exprs");
        match &exprs {
            Expr::EList(vec_ref) => {
                let mut last = Expr::ENil;
                for expr in &vec_ref.borrow().exprs {
                    last = self.apply(&env, vec![expr.clone()])
                }
                last
            }
            _ => exprs,
        }
    }

    fn native_let(&self, env: &Expr) -> Expr {
        let vars = self.loc_expr(&env, "vars");
        let expr = self.loc_expr(&env, "expr");
        self.apply(env, vec![vars, expr])
    }

    fn native_def(&self, env: &Expr) -> Expr {
        let this = self.loc_expr(&env, "@");
        let name = self.loc_expr(&env, "name");
        let value = self.loc_expr(&env, "value");

        // Kind of a hack -- assign block names in (def ...)
        match (&name, &value) {
            (Expr::EId(id), Expr::EBlock(block_ref)) => {
                let block = &mut *block_ref.borrow_mut();
                block.loc.name = id.clone();
            }
            (_, _) => (),
        }

        self.def(&this, &name, &value);
        Expr::ENil
    }

    pub fn native_set(&self, env: &Expr) -> Expr {
        let this = self.loc_expr(&env, "@");
        let name = self.loc_expr(&env, "name");
        let value = self.loc_expr(&env, "value");
        self.set(&this, &name, &value);
        env.clone()
    }

    fn native_log(&self, env: &Expr) -> Expr {
        println!("{}", self.loc_expr(&env, "msg"));
        Expr::ENil
    }

    fn native_try(&self, env: &Expr) -> Expr {
        let block = self.loc_expr(&env, "block");
        let catch = self.loc_expr(&env, "catch");
        match (&block, &catch) {
            (Expr::EBlock(_), Expr::EBlock(_)) => {
                match panic::catch_unwind(|| self.apply(&env, vec![block.clone()])) {
                    Ok(result) => result,
                    Err(_) => {
                        match self.exception.replace(None) {
                            Some(e) => self.apply(&env, vec![catch.clone(), e]),
                            None => self.apply(&env, vec![catch.clone()]),
                        }
                    }
                }
            }
            (_, _) => panic!(),
        }
    }

    fn native_add(&self, env: &Expr) -> Expr {
        let mut total = 0f64;
        self.addmul_helper(env, |x| total += x);
        Expr::ENum(total)
    }

    fn native_mul(&self, env: &Expr) -> Expr {
        let mut total = 1f64;
        self.addmul_helper(env, |x| total *= x);
        Expr::ENum(total)
    }

    fn addmul_helper<F>(&self, env: &Expr, mut func: F)
    where
        F: FnMut(f64),
    {
        match &self.loc_expr(&env, "vals") {
            Expr::EList(vec_ref) => {
                for val in &vec_ref.borrow().exprs {
                    match val {
                        Expr::ENum(x) => func(*x),
                        _ => panic!("+ requires numeric values"),
                    }
                }
            }
            _ => panic!("expected vals list"),
        }
    }

    fn native_sub(&self, env: &Expr) -> Expr {
        let x = self.loc_num(&env, "x");
        let oy = self.loc_opt_num(env, "y");
        match oy {
            Some(y) => Expr::ENum(x - y),
            None => Expr::ENum(-x),
        }
    }

    fn native_div(&self, env: &Expr) -> Expr {
        let x = self.loc_num(&env, "x");
        let oy = self.loc_opt_num(env, "y");
        match oy {
            Some(y) => Expr::ENum(x / y),
            None => Expr::ENum(1f64 / x),
        }
    }

    fn native_not(&self, env: &Expr) -> Expr {
        let x = self.loc_bool(&env, "x");
        Expr::EBool(!x)
    }
}
