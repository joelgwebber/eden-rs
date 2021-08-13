#[cfg(test)]
mod tests {
    use crate::kurt::Kurt;

    #[test]
    fn inline() {
        Kurt::test(
            "inline test",
            r#"
            (do
                (| expect true true)
                (| expect 42 42)
                (| expect "foo" "foo")
                (| expect [1 2 3] [1 2 3])
                (| expect {:foo 42 :bar 54} {:foo 42 :bar 54})
            )
        "#,
        );
    }

    #[test]
    fn basic() {
        Kurt::test_file("src/kurt/basic_test.kurt");
    }

    #[test]
    fn objects() {
        Kurt::test_file("src/kurt/objects_test.kurt");
    }

    #[test]
    fn panics() {
        Kurt::test_file("src/kurt/panics_test.kurt");
    }

    #[test]
    fn blocks() {
        Kurt::test_file("src/kurt/blocks_test.kurt");
    }

    #[test]
    fn macros() {
        Kurt::test_file("src/kurt/macros_test.kurt");
    }
}
