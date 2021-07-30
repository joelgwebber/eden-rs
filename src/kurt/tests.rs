#[cfg(test)]
mod tests {
    use std::fs;

    use velcro::vec_from;

    use crate::kurt::{
        builtins::{builtin, loc},
        eq::node_eq,
        Kurt, Node,
    };

    #[test]
    fn inline() {
        kurt_test(
            "inline test",
            r#"
            (do [
                (| expect true true)
                (| expect 42 42)
                (| expect "foo" "foo")
            ])
        "#,
        );
    }

    #[test]
    fn basic() {
        kurt_test_file("src/kurt/tests/basic.kurt");
    }

    #[test]
    fn eq() {
        kurt_test_file("src/kurt/tests/eq.kurt");
    }

    #[test]
    fn objects() {
        kurt_test_file("src/kurt/tests/objects.kurt");
    }

    #[test]
    fn panics() {
        kurt_test_file("src/kurt/tests/panics.kurt");
    }

    #[test]
    fn math() {
        kurt_test_file("src/kurt/tests/math.kurt");
    }

    #[test]
    fn blocks() {
        kurt_test_file("src/kurt/tests/blocks.kurt");
    }

    #[test]
    fn macros() {
        kurt_test_file("src/kurt/tests/macros.kurt");
    }

    fn kurt_test_file(filename: &str) {
        kurt_test(
            filename,
            fs::read_to_string(filename)
                .expect("cannot read test file")
                .as_str(),
        )
    }

    fn kurt_test(name: &str, src: &str) {
        println!("-- {}", name);
        let mut kurt = Kurt::new();
        kurt.def(
            "expect".into(),
            builtin(vec_from!["expect", "expr"], native_expect),
        );
        kurt.eval_src(src.into());
    }

    fn native_expect(env: Node) -> Node {
        let expect = loc(&env, "expect");
        let expr = loc(&env, "expr");
        if !node_eq(expect.clone(), expr.clone()) {
            assert!(false, "expected {} : got {}", expect.clone(), expr.clone());
        }
        Node::Nil
    }
}
