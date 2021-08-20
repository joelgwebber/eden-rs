use std::panic;

use velcro::vec_from;

use crate::kurt::{
    expr::{Expr, _bool, _id, _FALSE, _NIL, _TRUE},
    Kurt,
};

use super::eq::expr_eq;

impl Kurt {
    pub fn init_core(&mut self) {
        self.add_builtin("do", &vec_from!["exprs..."], Kurt::native_do);
        self.add_builtin("def", &vec_from!["name", "value"], Kurt::native_def);
        self.add_builtin("def-all", &vec_from!["values"], Kurt::native_def_all);
        self.add_builtin("let", &vec_from!["vars", "expr"], Kurt::native_let);
        self.add_builtin("set", &vec_from!["name", "value"], Kurt::native_set);
        self.add_builtin("set-all", &vec_from!["values"], Kurt::native_set_all);
        self.add_builtin("if", &vec_from!["cond", "if", "else"], Kurt::native_if);
        self.add_builtin("?", &vec_from!["id"], Kurt::native_exists);
        self.add_builtin("try", &vec_from!["block", "catch"], Kurt::native_try);
        self.add_builtin("print", &vec_from!["msgs..."], Kurt::native_print);

        self.add_builtin("=", &vec_from!["x", "y"], Kurt::native_eq);
        self.add_builtin("!=", &vec_from!["x", "y"], Kurt::native_neq);

        self.add_builtin("test", &vec_from!["name", "expr"], Kurt::native_test);
        self.add_builtin("expect", &vec_from!["expect", "expr"], Kurt::native_expect);

        self.add_builtin("not", &vec_from!["x"], Kurt::native_not);

        self.eval_file("./src/kurt/lib/core.kurt");

        // Override panic handler to suppress automatic stack traces.
        panic::set_hook(Box::new(|info| {
            // Kind of a hack -- panics created by (throw) will contain the string [exception].
            let s = format!("{}", info);
            if !s.contains("[exception]") {
                println!("{}", s);
            }
        }));
    }

    fn native_eq(&self, env: &Expr) -> Expr {
        let _a = self.loc(env, "x");
        let _b = self.loc(env, "y");
        _bool(expr_eq(_a, _b))
    }

    fn native_neq(&self, env: &Expr) -> Expr {
        let _a = self.loc(env, "x");
        let _b = self.loc(env, "y");
        _bool(!expr_eq(_a, _b))
    }

    fn native_do(&self, env: &Expr) -> Expr {
        let exprs = self.loc(&env, "exprs...");
        match &exprs {
            Expr::EList(vec_ref) => {
                let mut last = _NIL;
                for expr in &vec_ref.borrow().exprs {
                    last = self.apply(&env, vec![expr.clone()])
                }
                last
            }
            _ => exprs,
        }
    }

    fn native_let(&self, env: &Expr) -> Expr {
        let vars = self.loc(&env, "vars");
        let expr = self.loc(&env, "expr");
        self.apply(env, vec![vars, expr])
    }

    fn native_def(&self, env: &Expr) -> Expr {
        let this = self.loc(&env, "@");
        let name = self.loc(&env, "name");
        let value = self.loc(&env, "value");

        name_block(&name, &value);
        self.def(&this, &name, &value);
        this
    }

    fn native_def_all(&self, env: &Expr) -> Expr {
        let this = self.loc(&env, "@");
        let values = self.loc(&env, "values");
        match &values {
            Expr::EDict(dict_ref) => {
                let dict = &*dict_ref.borrow();
                for (name, value) in &dict.map {
                    let name_expr = _id(name.as_str());
                    name_block(&name_expr, &value);
                    self.def(&this, &name_expr, &value);
                }
                this
            }
            _ => self.throw(env, "def_all takes dict".into()),
        }
    }

    fn native_set(&self, env: &Expr) -> Expr {
        let this = self.loc(&env, "@");
        let name = self.loc(&env, "name");
        let value = self.loc(&env, "value");

        name_block(&name, &value);
        self.set(&this, &name, &value);
        this
    }

    fn native_set_all(&self, env: &Expr) -> Expr {
        let this = self.loc(&env, "@");
        let values = self.loc(&env, "values");
        match &values {
            Expr::EDict(dict_ref) => {
                let dict = &*dict_ref.borrow();
                for (name, value) in &dict.map {
                    let name_expr = _id(name.as_str());
                    name_block(&name_expr, &value);
                    self.def(&this, &name_expr, &value);
                }
                this
            }
            _ => self.throw(env, "def_all takes dict".into()),
        }
    }

    fn native_if(kurt: &Kurt, env: &Expr) -> Expr {
        let cond = kurt.loc(env, "cond");
        let _if = kurt.loc(env, "if");
        let _else = kurt.loc(env, "else");
        match &cond {
            Expr::EBool(b) => {
                if *b {
                    kurt.apply(env, vec![_if.clone()])
                } else {
                    kurt.apply(env, vec![_else.clone()])
                }
            }
            _ => kurt.throw(env, format!("")),
        }
    }

    fn native_exists(kurt: &Kurt, env: &Expr) -> Expr {
        let this = kurt.loc(env, "@");
        let id = kurt.loc(env, "id");
        match &id {
            Expr::EId(name) => match kurt.find_scope(&this, &name) {
                Some(_) => _TRUE,
                _ => _FALSE,
            },
            _ => _FALSE,
        }
    }

    fn native_print(&self, env: &Expr) -> Expr {
        let list = self.loc_list(&env, "msgs...");
        for expr in list {
            if expr != _NIL {
                print!("{} ", expr);
            }
        }
        println!();
        _NIL
    }

    fn native_try(&self, env: &Expr) -> Expr {
        let block = self.loc(&env, "block");
        let catch = self.loc(&env, "catch");
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

    fn native_test(&self, env: &Expr) -> Expr {
        let name = self.loc_str(env, "name");
        let expr = self.loc(env, "expr");
        println!("-[ {} ]-", name);
        self.apply(&env, vec![expr.clone()]);
        println!();
        _NIL
    }

    fn native_expect(&self, env: &Expr) -> Expr {
        let expect = self.loc(env, "expect");
        let expr = self.loc(env, "expr");
        if !expr_eq(expect.clone(), expr.clone()) {
            self.throw(
                env,
                format!("expected {} : got {}", expect.clone(), expr.clone()),
            );
        }
        _NIL
    }

    fn native_not(&self, env: &Expr) -> Expr {
        _bool(!self.loc_bool(&env, "x"))
    }
}

// Kind of a hack -- assign block names in (def ...)
fn name_block(name: &Expr, value: &Expr) {
    match (name, value) {
        (Expr::EId(id), Expr::EBlock(block_ref)) => {
            let block = &mut *block_ref.borrow_mut();
            block.loc.name = id.clone();
        }
        (_, _) => (),
    }
}

#[cfg(test)]
mod tests {
    use crate::kurt::Kurt;

    #[test]
    fn core() {
        Kurt::test_file("src/kurt/lib/core_test.kurt");
    }

    #[test]
    fn core_obj() {
        Kurt::test_file("src/kurt/lib/core_obj_test.kurt");
    }
}
