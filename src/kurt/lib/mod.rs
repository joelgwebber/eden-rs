use super::{Kurt, Loc, expr::{Block, ERef, Expr, _id, _NIL}};

mod core;
mod eq;
mod math;
mod str;
mod list;
mod dict;

impl Kurt {
    pub fn init_lib(&mut self) {
        self.init_core();
        self.init_math();
        self.init_str();
        self.init_list();
        self.init_dict();
    }

    pub fn builtin(&self, name: &'static str, args: &Vec<String>) -> Expr {
        Expr::EBlock(ERef::new(Block {
            loc: Loc {
                file: String::default(),
                name: name.to_string(),
                pos: (0, 0),
            },
            params: args.clone(),
            expr: Expr::ENative(name),
            env: _NIL,
            slf: _NIL,
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
            &_id(name),
            &self.builtin(name, args).clone(),
        );
    }

    pub fn loc(&self, env: &Expr, name: &str) -> Expr {
        if let Expr::EDict(env_map_ref) = &env {
            let env_map = &env_map_ref.borrow().map;
            match env_map.get(name) {
                Some(result) => result.clone(),
                None => self.throw(&env, format!("missing local '{}' in {}", name, env)),
            }
        } else {
            self.throw(env, format!("expected dict env"))
        }
    }

    pub fn loc_list(&self, env: &Expr, name: &str) -> Vec<Expr> {
        match &self.loc(env, name) {
            Expr::EList(list_ref) => {
                let list = &*list_ref.borrow();
                list.exprs.clone()
            }
            _ => self.throw(env, format!("expected list")),
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
            self.throw(env, format!("expected dict env"))
        }
    }

    pub fn loc_str(&self, env: &Expr, name: &str) -> String {
        let expr = self.loc(env, name);
        match &expr {
            Expr::EStr(s) => s.clone(),
            _ => self.throw(env, format!("expected str, got {}", expr)),
        }
    }

    pub fn loc_num(&self, env: &Expr, name: &str) -> f64 {
        let expr = self.loc(env, name);
        match &expr {
            Expr::ENum(x) => *x,
            _ => self.throw(env, format!("expected num, got {}", expr)),
        }
    }

    pub fn loc_bool(&self, env: &Expr, name: &str) -> bool {
        let expr = self.loc(env, name);
        match &expr {
            Expr::EBool(x) => *x,
            _ => self.throw(env, format!("expected bool, got {}", expr)),
        }
    }

    pub fn loc_opt_num(&self, env: &Expr, name: &str) -> Option<f64> {
        match self.loc_opt(env, name) {
            Some(expr) => match &expr {
                Expr::ENum(x) => Some(*x),
                _ => self.throw(env, format!("expected num, got {}", expr)),
            },
            None => None,
        }
    }
}
