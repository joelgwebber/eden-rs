use velcro::{hash_map, vec_from};

use crate::kurt::{Expr, expr::{_bool, _dict, _num}};

use super::{Kurt};

impl Kurt {
    pub fn init_math(&mut self) {
        self.add_builtin("+", &vec_from!["vals..."], Kurt::native_add);
        self.add_builtin("*", &vec_from!["vals..."], Kurt::native_mul);
        self.add_builtin("-", &vec_from!["x", "y"], Kurt::native_sub);
        self.add_builtin("/", &vec_from!["x", "y"], Kurt::native_div);

        self.add_builtin("<", &vec_from!["x", "y"], Kurt::native_lt);
        self.add_builtin(">", &vec_from!["x", "y"], Kurt::native_gt);
        self.add_builtin("<=", &vec_from!["x", "y"], Kurt::native_lte);
        self.add_builtin(">=", &vec_from!["x", "y"], Kurt::native_gte);

        self.add_builtin("cos", &vec_from!["x"], Kurt::native_cos);
        self.add_builtin("sin", &vec_from!["x"], Kurt::native_sin);

        self.def_num = _dict(hash_map! {});
    }

    fn native_add(&self, env: &Expr) -> Expr {
        let mut total = 0f64;
        self.addmul_helper(env, |x| total += x);
        _num(total)
    }

    fn native_mul(&self, env: &Expr) -> Expr {
        let mut total = 1f64;
        self.addmul_helper(env, |x| total *= x);
        _num(total)
    }

    fn addmul_helper<F>(&self, env: &Expr, mut func: F)
    where
        F: FnMut(f64),
    {
        match &self.loc(&env, "vals...") {
            Expr::EList(vec_ref) => {
                for val in &vec_ref.borrow().exprs {
                    match val {
                        Expr::ENum(x) => func(*x),
                        _ => self.throw(env, "operator requires numeric values".to_string()),
                    }
                }
            }
            _ => self.throw(env, "operator expected vals list".to_string()),
        }
    }

    fn native_sub(&self, env: &Expr) -> Expr {
        let x = self.loc_num(&env, "x");
        let oy = self.loc_opt_num(env, "y");
        match oy {
            Some(y) => _num(x - y),
            None => _num(-x),
        }
    }

    fn native_div(&self, env: &Expr) -> Expr {
        let x = self.loc_num(&env, "x");
        let oy = self.loc_opt_num(env, "y");
        match oy {
            Some(y) => _num(x / y),
            None => _num(1f64 / x),
        }
    }

    fn native_lt(&self, env: &Expr) -> Expr {
        _bool(self.loc_num(&env, "x") < self.loc_num(&env, "y"))
    }

    fn native_gt(&self, env: &Expr) -> Expr {
        _bool(self.loc_num(&env, "x") > self.loc_num(&env, "y"))
    }

    fn native_lte(&self, env: &Expr) -> Expr {
        _bool(self.loc_num(&env, "x") <= self.loc_num(&env, "y"))
    }

    fn native_gte(&self, env: &Expr) -> Expr {
        _bool(self.loc_num(&env, "x") >= self.loc_num(&env, "y"))
    }

    fn native_sin(&self, env: &Expr) -> Expr {
        _num(f64::sin(self.loc_num(&env, "x")))
    }

    fn native_cos(&self, env: &Expr) -> Expr {
        _num(f64::cos(self.loc_num(&env, "x")))
    }
}

mod tests {
    use crate::kurt::Kurt;

    #[test]
    fn math() {
        Kurt::test_file("src/kurt/lib/math_test.kurt");
    }
}
