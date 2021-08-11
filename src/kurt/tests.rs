#[cfg(test)]
mod tests {
    use std::fs;

    use velcro::vec_from;

    use crate::kurt::{
        eq::expr_eq,
        Kurt, Expr,
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
                (| expect [1 2 3] [1 2 3])
                (| expect {:foo 42 :bar 54} {:foo 42 :bar 54})
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
        kurt.add_builtin("expect", &vec_from!["expect", "expr"], native_expect);
        kurt.eval_src(name, src);
    }

    fn native_expect(kurt: &Kurt, env: &Expr) -> Expr {
        let expect = kurt.loc_expr(env, "expect");
        let expr = kurt.loc_expr(env, "expr");
        if !expr_eq(expect.clone(), expr.clone()) {
            kurt.throw(env, format!("expected {} : got {}", expect.clone(), expr.clone()));
        }
        Expr::ENil
    }
}
