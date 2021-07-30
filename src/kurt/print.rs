use super::Node;
use std::{collections::HashMap, fmt::{self, Display}};

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Node::Nil => write!(f, "nil"),
            Node::Num(n) => write!(f, "{}", n),
            Node::Str(n) => write!(f, "{}", n),
            Node::Bool(n) => write!(f, "{}", n),
            Node::Id(n) => write!(f, "{}", n),
            Node::Native(_) => write!(f, "<native>"),

            Node::Apply(vec_ref) => {
                write!(f, "(")?;
                write_vec(f, &*vec_ref.borrow())?;
                write!(f, ")")
            }

            Node::List(vec_ref) => {
                write!(f, "[")?;
                write_vec(f, &*vec_ref.borrow())?;
                write!(f, "]")
            }

            Node::Assoc(map_ref) => {
                write!(f, "{{")?;
                write_pairs(f, &*map_ref.borrow())?;
                write!(f, "}}")?;
                Ok(())
            }

            Node::Dict(map_ref) => {
                write!(f, "{{")?;
                write_map(f, &*map_ref.borrow())?;
                write!(f, "}}")?;
                Ok(())
            }

            Node::Block(block_ref) => {
                let block = &*block_ref.borrow();
                // TODO: env?
                write!(f, "(")?;
                write_vec(f, &block.params)?;
                write!(f, " | ")?;
                block.expr.fmt(f)?;
                write!(f, ")")
            }

            Node::Quote(node_ref) => {
                let node = &*node_ref.borrow();
                write!(f, ":{}", node)
            }

            Node::Unquote(node_ref) => {
                let node = &*node_ref.borrow();
                write!(f, "\\{}", node)
            }
        }
    }
}

fn write_vec<T:Display>(f: &mut fmt::Formatter, v: &Vec<T>) -> fmt::Result {
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

fn write_pairs(f: &mut fmt::Formatter, m: &Vec<(Node, Node)>) -> fmt::Result {
    use std::fmt::Write;

    let mut i = 0;
    for (key, node) in m {
        write!(f, "{} ", key)?;
        match node {
            Node::Dict(_) => { f.write_str("{...}"); () }
            Node::List(_) => { f.write_str("[...]"); () }
            _ => node.fmt(f)?,
        }
        if i < m.len() - 1 {
            f.write_char(' ')?;
        }
        i += 1;
    }
    Ok(())
}

fn write_map(f: &mut fmt::Formatter, m: &HashMap<String, Node>) -> fmt::Result {
    use std::fmt::Write;

    let mut i = 0;
    for (name, node) in m {
        write!(f, "{} ", name)?;
        match node {
            Node::Dict(_) => { f.write_str("{...}"); () }
            Node::List(_) => { f.write_str("[...]"); () }
            _ => node.fmt(f)?,
        }
        if i < m.len() - 1 {
            f.write_char(' ')?;
        }
        i += 1;
    }
    Ok(())
}
