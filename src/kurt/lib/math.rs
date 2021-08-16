use velcro::{hash_map, vec_from};

use crate::kurt::{expr::Dict, Expr, Loc};

use super::{ERef, Kurt};

impl Kurt {
    pub fn init_math(&mut self) {
        self.add_builtin("+", &vec_from!["vals..."], Kurt::native_add);
        self.add_builtin("*", &vec_from!["vals..."], Kurt::native_mul);
        self.add_builtin("-", &vec_from!["x", "y"], Kurt::native_sub);
        self.add_builtin("/", &vec_from!["x", "y"], Kurt::native_div);

        self.add_builtin("cos", &vec_from!["x"], Kurt::native_cos);
        self.add_builtin("sin", &vec_from!["x"], Kurt::native_sin);

        self.add_builtin("not", &vec_from!["x"], Kurt::native_not);

        self.def_num = Expr::EDict(ERef::new(Dict {
            loc: Loc::default(),
            map: hash_map! {},
        }));
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
        match &self.loc_expr(&env, "vals...") {
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

    fn native_sin(&self, env: &Expr) -> Expr {
        Expr::ENum(f64::sin(self.loc_num(&env, "x")))
    }

    fn native_cos(&self, env: &Expr) -> Expr {
        Expr::ENum(f64::cos(self.loc_num(&env, "x")))
    }

    fn native_not(&self, env: &Expr) -> Expr {
        let x = self.loc_bool(&env, "x");
        Expr::EBool(!x)
    }
}

mod tests {
    use crate::kurt::Kurt;

    #[test]
    fn math() {
        Kurt::test_file("src/kurt/lib/math_test.kurt");
    }
}

