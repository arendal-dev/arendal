extern crate num;
extern crate arendal_error;

use num::BigInt;
#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    IntLiteral(BigInt),
}

#[cfg(test)]
mod tests {
    use num::bigint::ToBigInt;

    use super::*;

    #[test]
    fn it_works() {
        let _ = Expr::IntLiteral(0.to_bigint().unwrap());
    }
}
