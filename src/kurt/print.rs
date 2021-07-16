use super::Node;
use std::{collections::HashMap, fmt};

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
            Node::Exec => write!(f, "$!"), // TODO: Can this even happen?

            Node::List(vec_ref) => {
                // TODO: There's gotta be a terser way to handle errors in this?
                write!(f, "[")?;
                write_vec(f, &*vec_ref.borrow())?;
                write!(f, "]")
            }

            Node::Dict(map_ref) => {
                // TODO: There's gotta be a terser way to handle errors in this?
                write!(f, "{{")?;
                write_map(f, &*map_ref.borrow())?;
                write!(f, "}}")?;
                Ok(())
            }

            Node::Block(block_ref) => {
                let block = &*block_ref.borrow();
                write!(f, "(")?;
                write_vec(f, &block.0)?;
                write!(f, " | ")?;
                block.1.fmt(f)?;
                write!(f, ")")
            }
        }
    }
}

fn write_map(f: &mut fmt::Formatter, m: &HashMap<String, Node>) -> fmt::Result {
    use std::fmt::Display;
    use std::fmt::Write;

    let mut i = 0;
    for (name, node) in m {
        write!(f, "{}: ", name)?;
        node.fmt(f)?;
        if i < m.len() - 1 {
            f.write_char(' ')?;
        }
        i += 1;
    }
    Ok(())
}

fn write_vec(f: &mut fmt::Formatter, v: &Vec<Node>) -> fmt::Result {
    use std::fmt::Display;
    use std::fmt::Write;

    let mut i = 0;
    for node in v {
        node.fmt(f)?;
        i += 1;
        if i < v.len() {
            f.write_char(' ')?;
        }
    }
    Ok(())
}
