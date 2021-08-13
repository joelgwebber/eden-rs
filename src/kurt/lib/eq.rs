use super::Expr;

pub fn expr_eq(_a: Expr, _b: Expr) -> bool {
    match &_a {
        Expr::ENil => {
            if let Expr::ENil = &_b {
                true
            } else {
                false
            }
        }
        Expr::EBool(a) => {
            if let Expr::EBool(b) = &_b {
                a == b
            } else {
                false
            }
        }
        Expr::ENum(a) => {
            if let Expr::ENum(b) = &_b {
                a == b
            } else {
                false
            }
        }
        Expr::EStr(a) => {
            if let Expr::EStr(b) = &_b {
                a == b
            } else {
                false
            }
        }
        Expr::EId(a) => {
            if let Expr::EId(b) = &_b {
                a == b
            } else {
                false
            }
        }

        Expr::EList(a_ref) => {
            if let Expr::EList(b_ref) = &_b {
                let a_exprs = &a_ref.borrow().exprs;
                let b_exprs = &b_ref.borrow().exprs;
                let mut eq = false;
                if a_exprs.len() == b_exprs.len() {
                    eq = true;
                    for i in 0..a_exprs.len() {
                        if !expr_eq(a_exprs[i].clone(), b_exprs[i].clone()) {
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

        Expr::EAssoc(a_ref) => {
            if let Expr::EAssoc(b_ref) = &_b {
                let a_pairs = &a_ref.borrow().pairs;
                let b_pairs = &b_ref.borrow().pairs;
                let mut eq = false;
                if a_pairs.len() == b_pairs.len() {
                    eq = true;
                    for i in 0..a_pairs.len() {
                        if !expr_eq(a_pairs[i].0.clone(), b_pairs[i].0.clone()) {
                            eq = false;
                            break;
                        }
                        if !expr_eq(a_pairs[i].1.clone(), b_pairs[i].1.clone()) {
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

        Expr::EDict(a_ref) => {
            if let Expr::EDict(b_ref) = &_b {
                let a_map = &a_ref.borrow().map;
                let b_map = &b_ref.borrow().map;
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
        Expr::EApply(a_ref) => {
            if let Expr::EApply(b_ref) = &_b {
                let a_exprs = &a_ref.borrow().exprs;
                let b_exprs = &b_ref.borrow().exprs;
                let mut eq = false;
                if a_exprs.len() == b_exprs.len() {
                    eq = true;
                    for i in 0..a_exprs.len() {
                        if !expr_eq(a_exprs[i].clone(), b_exprs[i].clone()) {
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

        Expr::EQuote(a) => {
            if let Expr::EQuote(b) = &_b {
                expr_eq((&*a.borrow()).clone(), (&*b.borrow()).clone())
            } else {
                false
            }
        }

        // TODO: Ref equality on blocks, etc?
        _ => false,
    }
}
