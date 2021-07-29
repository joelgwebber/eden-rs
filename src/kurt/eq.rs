use super::{builtins::loc, Node};

pub fn native_eq(env: Node) -> Node {
    let _a = loc(&env, "x");
    let _b = loc(&env, "y");

    Node::Bool(node_eq(_a, _b))
}

pub fn node_eq(_a: Node, _b: Node) -> bool {
    match &_a {
        Node::Nil => {
            if let Node::Nil = &_b {
                true
            } else {
                false
            }
        }
        Node::Bool(a) => {
            if let Node::Bool(b) = &_b {
                a == b
            } else {
                false
            }
        }
        Node::Num(a) => {
            if let Node::Num(b) = &_b {
                a == b
            } else {
                false
            }
        }
        Node::Str(a) => {
            if let Node::Str(b) = &_b {
                a == b
            } else {
                false
            }
        }
        Node::Id(a) => {
            if let Node::Id(b) = &_b {
                a == b
            } else {
                false
            }
        }

        Node::List(a_ref) => {
            if let Node::List(b_ref) = &_b {
                let a_vec = &*a_ref.borrow();
                let b_vec = &*b_ref.borrow();
                let mut eq = false;
                if a_vec.len() == b_vec.len() {
                    eq = true;
                    for i in 0..a_vec.len() {
                        if !node_eq(a_vec[i].clone(), b_vec[i].clone()) {
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

        Node::Dict(a_ref) => {
            if let Node::Dict(b_ref) = &_b {
                let a_map = &*a_ref.borrow();
                let b_map = &*b_ref.borrow();
                let mut eq = false;
                if a_map.len() == b_map.len() {
                    eq = true;
                    for (k, v) in a_map {
                        if !node_eq(v.clone(), b_map[k].clone()) {
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

        Node::Quote(a) => {
            if let Node::Quote(b) = &_b {
                node_eq((&*a.borrow()).clone(), (&*b.borrow()).clone())
            } else {
                false
            }
        }

        // TODO: Ref equality on blocks, etc?
        _ => false,
    }
}
