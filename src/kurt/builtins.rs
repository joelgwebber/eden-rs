use std::{panic, vec};

use velcro::{hash_map, vec_from};

use crate::kurt::Node;

use super::{node::Block, Kurt, NodeRef};

impl Kurt {
    pub fn init_builtins(&mut self) {
        // Built-in functions.
        self.add_builtin("do", &vec_from!["exprs"], Kurt::native_do);
        self.add_builtin("def", &vec_from!["vals"], Kurt::native_def);
        self.add_builtin("let", &vec_from!["vars", "expr"], Kurt::native_let);

        self.add_builtin("set".into(), &vec_from!["name", "value"], Kurt::native_set);
        self.add_builtin("log".into(), &vec_from!["msg"], Kurt::native_log);
        self.add_builtin("try".into(), &vec_from!["block", "catch"], Kurt::native_try);

        self.add_builtin("=".into(), &vec_from!["x", "y"], Kurt::native_eq);
        self.add_builtin("+".into(), &vec_from!["vals"], Kurt::native_add);
        self.add_builtin("*".into(), &vec_from!["vals"], Kurt::native_mul);
        self.add_builtin("-".into(), &vec_from!["x", "y"], Kurt::native_sub);
        self.add_builtin("/".into(), &vec_from!["x", "y"], Kurt::native_div);

        self.add_builtin("not", &vec_from!["x"], Kurt::native_not);

        // Default implementation dicts.
        self.def_dict = Node::Dict(NodeRef::new(hash_map! {
            "set".into(): Kurt::builtin("set".into(), &vec_from!["name", "value"]),
            "def".into(): Kurt::builtin("def".into(), &vec_from!["vals"]),
        }));
        self.def_list = Node::Dict(NodeRef::new(hash_map! {
            "set".into(): Kurt::builtin("set".into(), &vec_from!["name", "value"]),
        }));

        // Override panic handler.
        panic::set_hook(Box::new(|info| {
            // TODO: Something special to keep track of panic info to promote to catch blocks.
            println!("{:?}", info);
        }));
    }

    pub fn builtin(name: &'static str, args: &Vec<String>) -> Node {
        Node::Block(NodeRef::new(Block {
            params: args.clone(),
            expr: Node::Native(name),
            env: Node::Nil,
            slf: Node::Nil,
        }))
    }

    pub fn add_builtin(&mut self, name: &'static str, args: &Vec<String>, f: fn(&Kurt, &Node) -> Node) {
        self.builtins.insert(name, f);
        self.def(&self.root, &Node::Id(name.to_string()), &Kurt::builtin(name, args));
    }

    pub fn loc(&self, env: &Node, name: &str) -> Node {
        if let Node::Dict(env_map_ref) = &env {
            let env_map = &*env_map_ref.borrow();
            match env_map.get(name) {
                Some(result) => result.clone(),
                None => panic!("missing local '{}' in {}", name, env),
            }
        } else {
            panic!("expected dict env, got '{}'", env)
        }
    }

    pub fn loc_opt(&self, env: &Node, name: &str) -> Option<Node> {
        if let Node::Dict(env_map_ref) = &env {
            let env_map = &*env_map_ref.borrow();
            match env_map.get(name) {
                Some(node) => Some(node.clone()),
                None => None,
            }
        } else {
            panic!("expected dict env, got '{}'", env)
        }
    }

    pub fn loc_str(&self, env: &Node, name: &str) -> String {
        let node = self.loc(env, name);
        match &node {
            Node::Str(s) => s.clone(),
            _ => panic!(),
        }
    }

    pub fn loc_num(&self, env: &Node, name: &str) -> f64 {
        let node = self.loc(env, name);
        match &node {
            Node::Num(x) => *x,
            _ => panic!(),
        }
    }

    pub fn loc_bool(&self, env: &Node, name: &str) -> bool {
        let node = self.loc(env, name);
        match &node {
            Node::Bool(x) => *x,
            _ => panic!(),
        }
    }

    pub fn loc_opt_num(&self, env: &Node, name: &str) -> Option<f64> {
        match self.loc_opt(env, name) {
            Some(node) => match &node {
                Node::Num(x) => Some(*x),
                _ => panic!(),
            },
            None => None,
        }
    }

    fn native_do(&self, env: &Node) -> Node {
        let exprs = self.loc(&env, "exprs");
        match &exprs {
            Node::List(vec_ref) => {
                let mut last = Node::Nil;
                for expr in &*vec_ref.borrow() {
                    last = self.apply(&env, vec![expr.clone()])
                }
                last
            }
            _ => exprs,
        }
    }

    fn native_def(&self, env: &Node) -> Node {
        let this = self.loc(&env, "@");
        let vals = self.loc(&env, "vals");
        match &vals {
            Node::Dict(vals_map_ref) => {
                for (k, v) in &*vals_map_ref.borrow() {
                    self.def(&this, &Node::Id(k.clone()), &v);
                }
            }
            _ => panic!("def requires a dict"),
        }
        env.clone()
    }

    fn native_let(&self, env: &Node) -> Node {
        let vars = self.loc(&env, "vars");
        let expr = self.loc(&env, "expr");
        self.apply(env, vec![vars, expr])
    }

    pub fn native_set(&self, env: &Node) -> Node {
        let this = self.loc(&env, "@");
        let name = self.loc(&env, "name");
        let value = self.loc(&env, "value");
        self.set(&this, &name, &value);
        env.clone()
    }

    fn native_log(&self, env: &Node) -> Node {
        println!("{}", self.loc(&env, "msg"));
        Node::Nil
    }

    fn native_try(&self, env: &Node) -> Node {
        let block = self.loc(&env, "block");
        let catch = self.loc(&env, "catch");
        match (&block, &catch) {
            (Node::Block(_), Node::Block(_)) => {
                let result = panic::catch_unwind(|| {
                    self.apply(&env, vec![block.clone()]);
                });
                if result.is_err() {
                    self.apply(&env, vec![catch.clone()]);
                }
            }
            (_, _) => panic!(),
        }
        Node::Nil
    }

    fn native_add(&self, env: &Node) -> Node {
        let mut total = 0f64;
        self.addmul_helper(env, |x| total += x);
        Node::Num(total)
    }

    fn native_mul(&self, env: &Node) -> Node {
        let mut total = 1f64;
        self.addmul_helper(env, |x| total *= x);
        Node::Num(total)
    }

    fn addmul_helper<F>(&self, env: &Node, mut func: F)
    where
        F: FnMut(f64),
    {
        match &self.loc(&env, "vals") {
            Node::List(vec_ref) => {
                for val in &*vec_ref.borrow() {
                    match val {
                        Node::Num(x) => func(*x),
                        _ => panic!("+ requires numeric values"),
                    }
                }
            }
            _ => panic!("expected vals list"),
        }
    }

    fn native_sub(&self, env: &Node) -> Node {
        let x = self.loc_num(&env, "x");
        let oy = self.loc_opt_num(env, "y");
        match oy {
            Some(y) => Node::Num(x - y),
            None => Node::Num(-x),
        }
    }

    fn native_div(&self, env: &Node) -> Node {
        let x = self.loc_num(&env, "x");
        let oy = self.loc_opt_num(env, "y");
        match oy {
            Some(y) => Node::Num(x / y),
            None => Node::Num(1f64 / x),
        }
    }

    fn native_not(&self, env: &Node) -> Node {
        let x = self.loc_bool(&env, "x");
        Node::Bool(!x)
    }
}
