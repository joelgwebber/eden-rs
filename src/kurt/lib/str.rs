use velcro::{hash_map, vec_from};

use crate::kurt::{
    expr::{_dict, _num},
    Expr,
};

use super::Kurt;

impl Kurt {
    pub fn init_str(&mut self) {
        self.add_builtin("str:len", &vec_from![], Kurt::native_str_len);
        self.def_str = _dict(hash_map!(
            "len".into(): self.builtin("str:len", &vec_from![]),
        ));
    }

    fn native_str_len(&self, env: &Expr) -> Expr {
        let s = self.loc_str(&env, "@");
        _num(s.len() as f64)
    }
}

mod tests {
    use crate::kurt::Kurt;

    #[test]
    fn math() {
        Kurt::test_file("src/kurt/lib/str_test.kurt");
    }
}
