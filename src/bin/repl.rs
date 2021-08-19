use eden::kurt::{expr::_dict, Kurt};
use rustyline::error::ReadlineError;
use velcro::hash_map;

fn main() {
    let kurt = Kurt::new();
    let env = _dict(hash_map!(
        "^".into(): kurt.root.clone(),
    ));

    let mut rl = rustyline::Editor::<()>::new();
    loop {
        match rl.readline("kurt > ") {
            Ok(line) => {
                if line.trim().len() > 0 {
                    let result = kurt.eval_src(&env, "repl", line.as_str());
                    println!("{}", result);
                }
            }

            Err(e) => match e {
                ReadlineError::Eof => break,
                ReadlineError::Interrupted => break,
                _ => println!("error: {}", e),
            },
        }
    }
}
