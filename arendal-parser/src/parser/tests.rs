use super::parse_expression;

#[test]
fn int_literal() {
    if parse_expression("1234").is_err() {
        panic!();
    }
}
