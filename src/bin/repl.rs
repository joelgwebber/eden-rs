use std::io::{self, Write};

use eden::kurt::{Kurt, expr::{_dict, _str}};
use velcro::hash_map;

fn main() {
    let kurt = Kurt::new();
    let env = _dict(hash_map!(
        "^".into(): kurt.root.clone(),
    ));
    loop {
        print!("repl > ");
        io::stdout().flush();

        let mut buf = String::new();
        io::stdin().read_line(&mut buf);

        if buf.trim().len() > 0 {
            let result = kurt.eval_src(&env, "repl", buf.as_str());
            println!("{}", result);
        }
    }
}
