use super::Expr;
use std::{
    collections::HashMap,
    fmt::{self, Display},
};

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::ENil => write!(f, "nil"),
            Expr::ENum(n) => write!(f, "{}", n),
            Expr::EStr(n) => write!(f, "{}", n),
            Expr::EBool(n) => write!(f, "{}", n),
            Expr::EId(n) => write!(f, "{}", n),
            Expr::ENative(n) => write!(f, "<native {}>", n),

            Expr::EApply(apply_ref) => {
                write!(f, "(")?;
                write_vec(f, &(apply_ref.borrow()).exprs)?;
                write!(f, ")")
            }

            Expr::EList(list_ref) => {
                write!(f, "[")?;
                write_vec(f, &(list_ref.borrow()).exprs)?;
                write!(f, "]")
            }

            Expr::EAssoc(map_ref) => {
                write!(f, "{{")?;
                write_pairs(f, &map_ref.borrow().pairs)?;
                write!(f, "}}")?;
                Ok(())
            }

            Expr::EDict(map_ref) => {
                write!(f, "{{")?;
                write_map(f, &map_ref.borrow().map)?;
                write!(f, "}}")?;
                Ok(())
            }

            Expr::EBlock(block_ref) => {
                let block = &*block_ref.borrow();
                // TODO: env?
                write!(f, "(")?;
                write_vec(f, &block.params)?;
                write!(f, " | ")?;
                // block.expr.fmt(f)?;
                write!(f, "...")?;
                write!(f, ")")
            }

            Expr::EQuote(eref) => {
                let expr = &*eref.borrow();
                write!(f, ":{}", expr)
            }

            Expr::EUnquote(eref) => {
                let expr = &*eref.borrow();
                write!(f, "\\{}", expr)
            }
        }
    }
}

fn write_vec<T: Display>(f: &mut fmt::Formatter, v: &Vec<T>) -> fmt::Result {
    use std::fmt::Write;

    let mut i = 0;
    for expr in v {
        expr.fmt(f)?;
        i += 1;
        if i < v.len() {
            f.write_char(' ')?;
        }
    }
    Ok(())
}

fn write_pairs(f: &mut fmt::Formatter, m: &Vec<(Expr, Expr)>) -> fmt::Result {
    use std::fmt::Write;

    let mut i = 0;
    for (key, expr) in m {
        write!(f, "{} ", key)?;
        match expr {
            Expr::EDict(_) => { f.write_str("{...}")?; () }
            Expr::EList(_) => { f.write_str("[...]")?; () }
            _ => expr.fmt(f)?,
        }
        if i < m.len() - 1 {
            f.write_char(' ')?;
        }
        i += 1;
    }
    Ok(())
}

fn write_map(f: &mut fmt::Formatter, m: &HashMap<String, Expr>) -> fmt::Result {
    use std::fmt::Write;

    let mut i = 0;
    for (name, expr) in m {
        write!(f, ":{} ", name)?;
        match expr {
            Expr::EDict(_) => { f.write_str("{...}")?; () }
            Expr::EList(_) => { f.write_str("[...]")?; () }
            _ => expr.fmt(f)?,
        }
        if i < m.len() - 1 {
            f.write_char(' ')?;
        }
        i += 1;
    }
    Ok(())
}
