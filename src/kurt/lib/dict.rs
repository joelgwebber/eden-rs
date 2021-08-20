use velcro::{hash_map, vec_from};

use crate::kurt::{Kurt, expr::_dict};

impl Kurt {
    pub fn init_dict(&mut self) {
        self.def_dict = _dict(hash_map! {
            "set".into(): self.builtin("set", &vec_from!["name", "value"]),
            "set-all".into(): self.builtin("set-all", &vec_from!["values"]),
            "def".into(): self.builtin("def", &vec_from!["name", "value"]),
            "def-all".into(): self.builtin("def-all", &vec_from!["values"]),
            "?".into(): self.builtin("?", &vec_from!["id"]),
        });
    }
}
