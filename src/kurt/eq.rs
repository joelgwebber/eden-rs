use super::{Kurt, Expr};

impl Kurt {
    pub fn native_eq(&self, env: &Expr) -> Expr {
        let _a = self.loc(env, "x");
        let _b = self.loc(env, "y");

        Expr::Bool(expr_eq(_a, _b))
    }
}

pub fn expr_eq(_a: Expr, _b: Expr) -> bool {
    match &_a {
        Expr::Nil => {
            if let Expr::Nil = &_b {
                true
            } else {
                false
            }
        }
        Expr::Bool(a) => {
            if let Expr::Bool(b) = &_b {
                a == b
            } else {
                false
            }
        }
        Expr::Num(a) => {
            if let Expr::Num(b) = &_b {
                a == b
            } else {
                false
            }
        }
        Expr::Str(a) => {
            if let Expr::Str(b) = &_b {
                a == b
            } else {
                false
            }
        }
        Expr::Id(a) => {
            if let Expr::Id(b) = &_b {
                a == b
            } else {
                false
            }
        }

        Expr::List(a_ref) => {
            if let Expr::List(b_ref) = &_b {
                let a_vec = &*a_ref.borrow();
                let b_vec = &*b_ref.borrow();
                let mut eq = false;
                if a_vec.len() == b_vec.len() {
                    eq = true;
                    for i in 0..a_vec.len() {
                        if !expr_eq(a_vec[i].clone(), b_vec[i].clone()) {
                            eq = false;
                            break;
                        }
                    }
                }
                eq
            } else {
                false
            }
        }

        Expr::Assoc(a_ref) => {
            if let Expr::Assoc(b_ref) = &_b {
                let a_vec = &*a_ref.borrow();
                let b_vec = &*b_ref.borrow();
                let mut eq = false;
                if a_vec.len() == b_vec.len() {
                    eq = true;
                    for i in 0..a_vec.len() {
                        if !expr_eq(a_vec[i].0.clone(), b_vec[i].0.clone()) {
                            eq = false;
                            break;
                        }
                        if !expr_eq(a_vec[i].1.clone(), b_vec[i].1.clone()) {
                            eq = false;
                            break;
                        }
                    }
                }
                eq
            } else {
                false
            }
        }

        Expr::Dict(a_ref) => {
            if let Expr::Dict(b_ref) = &_b {
                let a_map = &*a_ref.borrow();
                let b_map = &*b_ref.borrow();
                let mut eq = false;
                if a_map.len() == b_map.len() {
                    eq = true;
                    for (k, v) in a_map {
                        if !expr_eq(v.clone(), b_map[k].clone()) {
                            eq = false;
                            break;
                        }
                    }
                }
                eq
            } else {
                false
            }
        }

        // TODO: Merge with List.
        Expr::Apply(a_ref) => {
            if let Expr::Apply(b_ref) = &_b {
                let a_vec = &*a_ref.borrow();
                let b_vec = &*b_ref.borrow();
                let mut eq = false;
                if a_vec.len() == b_vec.len() {
                    eq = true;
                    for i in 0..a_vec.len() {
                        if !expr_eq(a_vec[i].clone(), b_vec[i].clone()) {
                            eq = false;
                            break;
                        }
                    }
                }
                eq
            } else {
                false
            }
        }

        Expr::Quote(a) => {
            if let Expr::Quote(b) = &_b {
                expr_eq((&*a.borrow()).clone(), (&*b.borrow()).clone())
            } else {
                false
            }
        }

        // TODO: Ref equality on blocks, etc?
        _ => false,
    }
}
