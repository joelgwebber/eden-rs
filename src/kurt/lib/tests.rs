#[cfg(test)]
mod tests {
    use crate::kurt::Kurt;

    #[test]
    fn eq() {
        Kurt::test_file("src/kurt/lib/eq_test.kurt");
    }

    #[test]
    fn math() {
        Kurt::test_file("src/kurt/lib/math_test.kurt");
    }
}
