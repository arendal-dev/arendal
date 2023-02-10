use ast::Type;

#[test]
fn integer() {
    let typed = super::expression(&ast::bare::lit_integer(num::Integer::from(1234)));
    assert_eq!(typed.unwrap().payload.loc_type, Type::Integer);
}
