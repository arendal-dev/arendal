use super::{lexer, Errors, Expression, Indentation, LexemeKind, LexemeRef, Lexemes, Result};

// Parses a single expression
fn parse_expression(input: &str) -> Result<Expression> {
    let lexemes = lexer::lex(input)?;
    Parser::new(lexemes).parse_expression()
}

struct Parser {
    input: Lexemes,
    index: usize, // Index of the current input lexer
    errors: Errors,
}

impl Parser {
    fn new(input: Lexemes) -> Parser {
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

    // Returns a clone of the lexer at the current index, if any
    fn peek(&self) -> Option<LexemeRef> {
        self.input.get(self.index)
    }

    // Consumes one lexer a returns the next one, if any.
    fn consume_and_peek(&mut self) -> Option<LexemeRef> {
        self.consume();
        self.peek()
    }

    // Returns a clone of the lexer the requested positions after the current one, if any.
    fn peek_ahead(&self, n: usize) -> Option<LexemeRef> {
        self.input.get(self.index + n)
    }

    // Parses the input as single expression, if any, consuming as many tokens as needed.
    // Assumes that the expression starts on a line.
    fn parse_expression(mut self) -> Result<Expression> {
        if self.is_done() {
            println!("Empty input: {:?}", self.input);
            self.empty_input()
        } else if let Some(e) = self.expression(Indentation::new(0, 0)) {
            println!("Expression returned");
            self.errors.to_result(e)
        } else {
            println!("None returned");
            Err(self.errors)
        }
    }

    // Parses a single expression, if any, consuming as many tokens as needed.
    // Assumes that the expression starts on a line.
    fn expression(&mut self, min_indent: Indentation) -> Option<Expression> {
        let peek = self.peek();
        if let Some(lexeme) = peek {
            match lexeme.kind() {
                LexemeKind::Indent(indentation) => {
                    if *indentation >= min_indent {
                        self.consume();
                        let ctx = ExprCtx::new(*indentation, &lexeme);
                        let rule_result = self.rule_expression(ctx);
                        match rule_result {
                            Ok(expr) => return Some(expr),
                            Err(error) => {
                                self.errors.add(error);
                                // TODO advance until next "resonable" line
                            }
                        }
                    } else {
                        self.add_error(&lexeme, ErrorKind::ParsingError);
                    }
                }
                _ => {
                    self.add_error(&lexeme, ErrorKind::ParsingError);
                }
            }
        }
        None
    }

    fn rule_expression(&mut self, ctx: ExprCtx) -> ExprResult {
        self.rule_primary(ctx)
    }

    // primary -> IntLiteral
    fn rule_primary(&mut self, ctx: ExprCtx) -> ExprResult {
        if let Some(lexeme) = self.peek() {
            match &lexeme.kind() {
                LexemeKind::Integer(n) => {
                    self.consume();
                    Ok(Expression::int_literal(lexeme.clone(), n.clone()))
                }
                _ => Err(Error::new(&lexeme, ErrorKind::ParsingError)),
            }
        } else {
            Err(Error::new(&ctx.last_seen, ErrorKind::ParsingError))
        }
    }

    fn add_error(&mut self, lexeme: &LexemeRef, kind: ErrorKind) {
        self.errors.add(Error::new(lexeme, kind));
    }

    fn empty_input<T>(mut self) -> Result<T> {
        self.errors.add(EmptyInputError {});
        Err(self.errors)
    }
}

#[derive(Debug)]
struct ExprCtx {
    ref_indent: Indentation, // Indentation to use as reference for this token
    last_seen: LexemeRef,    // Last lexeme seen, used for errors
}

impl ExprCtx {
    fn new(ref_indent: Indentation, last_seen: &LexemeRef) -> ExprCtx {
        ExprCtx {
            ref_indent,
            last_seen: last_seen.clone(),
        }
    }
}

#[derive(Debug)]
struct EmptyInputError {}

impl super::Error for EmptyInputError {}

#[derive(Debug)]
struct Error {
    lexeme: LexemeRef,
    kind: ErrorKind,
}

type ExprResult = std::result::Result<Expression, Error>;

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

#[cfg(test)]
mod tests;
