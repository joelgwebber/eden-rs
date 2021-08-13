use std::panic;

use velcro::{hash_map, vec_from};

use crate::kurt::{
    expr::{Dict, ERef, Expr},
    Kurt, Loc,
};

use super::eq::expr_eq;

impl Kurt {
    pub fn init_core(&mut self) {
        self.add_builtin("=", &vec_from!["x", "y"], Kurt::native_eq);
        self.add_builtin("do", &vec_from!["exprs..."], Kurt::native_do);
        self.add_builtin("def", &vec_from!["name", "value"], Kurt::native_def);
        self.add_builtin("let", &vec_from!["vars", "expr"], Kurt::native_let);
        self.add_builtin("set", &vec_from!["name", "value"], Kurt::native_set);
        self.add_builtin("try", &vec_from!["block", "catch"], Kurt::native_try);
        self.add_builtin("log", &vec_from!["msg"], Kurt::native_log);
        self.add_builtin("expect", &vec_from!["expect", "expr"], Kurt::native_expect);

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
    }

    fn native_eq(&self, env: &Expr) -> Expr {
        let _a = self.loc_expr(env, "x");
        let _b = self.loc_expr(env, "y");

        Expr::EBool(expr_eq(_a, _b))
    }

    fn native_do(&self, env: &Expr) -> Expr {
        let exprs = self.loc_expr(&env, "exprs...");
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

    fn native_set(&self, env: &Expr) -> Expr {
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
                    Err(_) => match self.exception.replace(None) {
                        Some(e) => self.apply(&env, vec![catch.clone(), e]),
                        None => self.apply(&env, vec![catch.clone()]),
                    },
                }
            }
            (_, _) => self.throw(env, "try requires body and catch blocks".to_string()),
        }
    }

    fn native_expect(kurt: &Kurt, env: &Expr) -> Expr {
        let expect = kurt.loc_expr(env, "expect");
        let expr = kurt.loc_expr(env, "expr");
        if !expr_eq(expect.clone(), expr.clone()) {
            kurt.throw(env, format!("expected {} : got {}", expect.clone(), expr.clone()));
        }
        Expr::ENil
    }
}
