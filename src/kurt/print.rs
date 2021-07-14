use super::{Node, NodeRef};
use std::{borrow::Borrow, fmt::{self, Debug}};

impl fmt::Display for NodeRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      self.borrow().fmt(f)
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Node::Nil => write!(f, "nil"),
            Node::Num(n) => write!(f, "{}", n),
            Node::Str(n) => write!(f, "{}", n),
            Node::Bool(n) => write!(f, "{}", n),
            Node::Id(n) => write!(f, "{}", n),
            Node::Sym(n) => write!(f, ":{}", n),
            Node::Native(_) => write!(f, "<native>"),

            Node::List(v) => {
                // TODO: There's gotta be a terser way to handle errors in this?
                write!(f, "[")?;
                write_vec(f, v)?;
                write!(f, "]")
            }

            Node::Dict(d) => {
                // TODO: There's gotta be a terser way to handle errors in this?
                write!(f, "{{")?;
                for (name, node) in d {
                    write!(f, "{}: ", name)?;
                    node.fmt(f)?;
                    write!(f, " ")?;
                }
                write!(f, "}}")?;
                Ok(())
            }

            Node::Block(args, expr) => {
                write!(f, "(")?;
                write_vec(f, args)?;
                write!(f, " | ")?;
                expr.fmt(f)?;
                write!(f, ")")
            }
        }
    }
}

fn write_vec(f: &mut fmt::Formatter, v: &Vec<NodeRef>) -> fmt::Result {
    use std::fmt::Display;
    use std::fmt::Write;

    let mut i = 0;
    for n in v {
        n.fmt(f)?;
        i += 1;
        if i < v.len() {
            f.write_char(' ')?;
        }
    }
    Ok(())
}
