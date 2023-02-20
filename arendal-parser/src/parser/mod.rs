use super::{lexer, Errors, Expression, LexemeRef, Lexemes, Line, Lines, Result};

// Parses a single expression (a single line for now)
pub fn parse_expression(input: &str) -> Result<Expression> {
    let lines = lexer::lex(input)?;
    Parser::new(lines).parse_expression()
}

struct Parser {
    input: Lines,
    index: usize, // Index of the current input lexer
    errors: Errors,
}

impl Parser {
    fn new(input: Lines) -> Parser {
        Parser {
            input,
            index: 0,
            errors: Default::default(),
        }
    }

    // Returns true if we have reached the end of the input
    fn is_done(&self) -> bool {
        !self.input.contains(self.index)
    }

    // Consumes one lexer, advancing the index accordingly.
    fn consume(&mut self) {
        self.index += 1;
    }

    // Returns a clone of the line at the current index, if any
    fn peek(&self) -> Option<Line> {
        self.input.get(self.index)
    }

    // Consumes one line a returns the next one, if any.
    fn consume_and_peek(&mut self) -> Option<Line> {
        self.consume();
        self.peek()
    }

    // Returns a clone of the line the requested positions after the current one, if any.
    fn peek_ahead(&self, n: usize) -> Option<Line> {
        self.input.get(self.index + n)
    }

    // Parses the current line as a single expression.
    fn parse_expression(mut self) -> Result<Expression> {
        if let Some(line) = self.peek() {
            let lexemes = self.get_expr_lexemes(line);
            self.errors
                .result_to_result(expr::Parser::new(lexemes).parse())
        } else {
            self.expression_expected()
        }
    }

    // Get the lexemes needed to parse an expression.
    fn get_expr_lexemes(&mut self, line: Line) -> Lexemes {
        let mut lines = vec![line.lexemes.clone()];
        self.consume();
        while let Some(additional) = self.peek() {
            if additional.indentation > line.indentation {
                lines.push(additional.lexemes.clone());
                self.consume()
            } else {
                break;
            }
        }
        if lines.len() > 1 {
            Lexemes::merge(lines)
        } else {
            lines.pop().unwrap()
        }
    }

    fn add_error(&mut self, lexeme: &LexemeRef, kind: ErrorKind) -> Option<Expression> {
        self.errors.add(Error::new(lexeme, kind));
        None
    }

    fn err_no_lexeme<T: super::Error + 'static>(mut self, error: T) -> Result<Expression> {
        self.errors.add(error);
        Err(self.errors)
    }

    fn empty_input(mut self) -> Result<Expression> {
        self.err_no_lexeme(EmptyInputError {})
    }

    fn expression_expected(mut self) -> Result<Expression> {
        self.err_no_lexeme(ExpressionExpectedError {})
    }
}

#[derive(Debug)]
struct EmptyInputError {}

impl super::Error for EmptyInputError {}

#[derive(Debug)]
struct ExpressionExpectedError {}

impl super::Error for ExpressionExpectedError {}

#[derive(Debug)]
struct Error {
    lexeme: LexemeRef,
    kind: ErrorKind,
}

impl Error {
    fn new(lexeme: &LexemeRef, kind: ErrorKind) -> Self {
        Error {
            lexeme: lexeme.clone(),
            kind,
        }
    }
}

#[derive(Debug)]
enum ErrorKind {
    ParsingError, // placeholder, temporary error
}

impl super::Error for Error {}

mod expr;

#[cfg(test)]
mod tests;
