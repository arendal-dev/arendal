use super::{Token, Tokens};
use crate::{Errors, Expression, Pos, Result};

// Tries to parses an expression
fn parse_expression(input: &str) -> Result<Option<Expression>> {
    let pass2 = super::tokenize(input)?;
    let expr = Parser::new(pass2).expression();
    Ok(expr)
}

struct Parser<'a> {
    input: Tokens<'a>,
    index: usize, // Index of the current input token
}

impl<'a> Parser<'a> {
    fn new(input: Tokens<'a>) -> Parser<'a> {
        Parser { input, index: 0 }
    }

    // Returns true if we have reached the end of the input
    fn is_done(&self) -> bool {
        self.index >= self.input.len()
    }

    // Consumes one token, advancing the index accordingly.
    fn consume(&mut self) {
        self.index += 1;
    }

    // Returns a clone of the token at the current index, if any
    fn peek(&self) -> Option<Box<Token<'a>>> {
        if self.is_done() {
            None
        } else {
            Some(self.input[self.index].clone())
        }
    }

    // Consumes one token a returns the next one, if any.
    fn consume_and_peek(&mut self) -> Option<Box<Token<'a>>> {
        self.consume();
        self.peek()
    }

    // Returns a clone of the token the requested positions after the current one, if any.
    fn peek_ahead(&self, n: usize) -> Option<Box<Token<'a>>> {
        let i = self.index + n;
        if i >= self.input.len() {
            None
        } else {
            Some(self.input[i].clone())
        }
    }

    // Tries to parses an expression, if any, consuming as many tokens as needed
    fn expression(&mut self) -> Option<Expression<'a>> {
        None
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
