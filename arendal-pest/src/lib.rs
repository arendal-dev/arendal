extern crate pest;
#[macro_use]
extern crate pest_derive;

extern crate arendal_ast;

use std::str::FromStr;

use arendal_ast::Expr;
use num::BigInt;
use pest::error::Error;
use pest::iterators::Pairs;
use pest::Parser;

#[derive(Parser)]
#[grammar = "arendal.pest"]
pub struct ArendalParser;

pub fn parse(input: &str) -> Result<Expr, Error<Rule>> {
    let pairs = ArendalParser::parse(Rule::integer, input)?;
    Ok(pairs_to_ast(pairs))
}

fn pairs_to_ast(mut pairs: Pairs<Rule>) -> Expr {
    Expr::IntLiteral(BigInt::from_str(pairs.next().unwrap().as_str()).unwrap())
}

#[cfg(test)]
mod tests {
    use num::bigint::ToBigInt;

    use super::*;

    #[test]
    fn it_works() {
        let result = parse("42");
        assert_eq!(result, Ok(Expr::IntLiteral(42.to_bigint().unwrap())));
    }
}
