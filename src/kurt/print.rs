use super::Expr;
use std::{
    collections::HashMap,
    fmt::{self, Display},
};

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Nil => write!(f, "nil"),
            Expr::Num(n) => write!(f, "{}", n),
            Expr::Str(n) => write!(f, "{}", n),
            Expr::Bool(n) => write!(f, "{}", n),
            Expr::Id(n) => write!(f, "{}", n),
            Expr::Native(n) => write!(f, "<native {}>", n),

            Expr::Apply(vec_ref) => {
                write!(f, "(")?;
                write_vec(f, &*vec_ref.borrow())?;
                write!(f, ")")
            }

            Expr::List(vec_ref) => {
                write!(f, "[")?;
                write_vec(f, &*vec_ref.borrow())?;
                write!(f, "]")
            }

            Expr::Assoc(map_ref) => {
                write!(f, "{{")?;
                write_pairs(f, &*map_ref.borrow())?;
                write!(f, "}}")?;
                Ok(())
            }

            Expr::Dict(map_ref) => {
                write!(f, "{{")?;
                write_map(f, &*map_ref.borrow())?;
                write!(f, "}}")?;
                Ok(())
            }

            Expr::Block(block_ref) => {
                let block = &*block_ref.borrow();
                // TODO: env?
                write!(f, "(")?;
                write_vec(f, &block.params)?;
                write!(f, " | ")?;
                block.expr.fmt(f)?;
                write!(f, ")")
            }

            Expr::Quote(eref) => {
                let expr = &*eref.borrow();
                write!(f, ":{}", expr)
            }

            Expr::Unquote(eref) => {
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
            // Expr::Dict(_) => { f.write_str("{...}"); () }
            // Expr::List(_) => { f.write_str("[...]"); () }
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
            // Expr::Dict(_) => { f.write_str("{...}"); () }
            // Expr::List(_) => { f.write_str("[...]"); () }
            _ => expr.fmt(f)?,
        }
        if i < m.len() - 1 {
            f.write_char(' ')?;
        }
        i += 1;
    }
    Ok(())
}
