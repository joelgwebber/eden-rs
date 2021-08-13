use super::{
    expr::{Block, ERef, Expr},
    Kurt, Loc,
};

mod core;
mod math;
mod eq;
mod tests;

impl Kurt {
    pub fn init_lib(&mut self) {
        self.init_core();
        self.init_math();
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
            self.throw(env, format!("expected dict env"))
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
        let expr = self.loc_expr(env, name);
        match &expr {
            Expr::EStr(s) => s.clone(),
            _ => self.throw(env, format!("expected str, got {}", expr)),
        }
    }

    pub fn loc_num(&self, env: &Expr, name: &str) -> f64 {
        let expr = self.loc_expr(env, name);
        match &expr {
            Expr::ENum(x) => *x,
            _ => self.throw(env, format!("expected num, got {}", expr)),
        }
    }

    pub fn loc_bool(&self, env: &Expr, name: &str) -> bool {
        let expr = self.loc_expr(env, name);
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
