use velcro::{hash_map, vec_from};

use crate::kurt::{
    expr::{Expr, _dict, _NIL, _num},
    Kurt,
};

impl Kurt {
    pub fn init_list(&mut self) {
        self.add_builtin("List:len", &vec_from![""], Kurt::native_len);
        self.add_builtin("List:for-each", &vec_from!["block"], Kurt::native_for_each);
        self.add_builtin("List:push", &vec_from!["value"], Kurt::native_push);
        self.add_builtin("List:pop", &vec_from![""], Kurt::native_pop);

        self.def_list = _dict(hash_map! {
            "set".into(): self.builtin("set", &vec_from!["name", "value"]),
            "len".into(): self.builtin("List:len", &vec_from![]),
            "for-each".into(): self.builtin("List:for-each", &vec_from!["block"]),
            "push".into(): self.builtin("List:push", &vec_from!["value"]),
            "pop".into(): self.builtin("List:pop", &vec_from![]),
        });
    }

    fn native_len(&self, env: &Expr) -> Expr {
        let this = self.loc(env, "@");
        match &this {
            Expr::EList(list_ref) => {
                let list = &mut *list_ref.borrow_mut();
                _num(list.exprs.len() as f64)
            }
            _ => self.throw(env, "len requires a list".into()),
        }
    }

    fn native_push(&self, env: &Expr) -> Expr {
        let this = self.loc(env, "@");
        let value = self.loc(env, "value");
        match &this {
            Expr::EList(list_ref) => {
                let list = &mut *list_ref.borrow_mut();
                list.exprs.push(value);
                _NIL
            }
            _ => self.throw(env, "push requires a lit".into()),
        }
    }

    fn native_pop(&self, env: &Expr) -> Expr {
        let this = self.loc(env, "@");
        match &this {
            Expr::EList(list_ref) => {
                let list = &mut *list_ref.borrow_mut();
                match list.exprs.pop() {
                    Some(expr) => expr.clone(),
                    None => self.throw(env, "attempted to pop an empty list".into()),
                }
            }
            _ => self.throw(env, "pop requires a list".into()),
        }
    }

    fn native_for_each(&self, env: &Expr) -> Expr {
        let this = self.loc(env, "@");
        let block = self.loc(env, "block");
        match &this {
            Expr::EList(list_ref) => {
                let list = &*list_ref.borrow();
                for i in 0..list.exprs.len() {
                    let item = list.exprs.get(i).unwrap();
                    self.apply(env, vec![block.clone(), _num(i as f64), item.clone()]);
                }
            }
            _ => self.throw(env, "for-each requires a list".into()),
        }
        _NIL
    }
}

#[cfg(test)]
mod tests {
    use crate::kurt::Kurt;

    #[test]
    fn list() {
        Kurt::test_file("src/kurt/lib/list_test.kurt");
    }
}
