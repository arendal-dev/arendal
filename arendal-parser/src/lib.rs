mod lexer;

use ast::{
    Binary, Expr, Expression, Statement, TypeAnnotation,
    common::BinaryOp,
    input::StringInput,
    position::Position,
    problem::{self, ErrorType, Problems, ProblemsResult, Result},
};
use lexer::{Lexeme, LexemeData, Lexemes, Separator};

pub fn parse(input: &str) -> Result<Vec<Statement>> {
    let input = StringInput::from_str(input);
    lexer::lex(input)?.and_then(|lexemes| parse_statements(&lexemes))
}

type EResult = Result<Expression>;

fn parse_statements(lexemes: &Lexemes) -> Result<Vec<Statement>> {
    let mut statements = Vec::<Statement>::new();
    let mut result: std::result::Result<problem::Warnings<()>, problem::Errors> = Problems::ok(());
    let mut index: usize = 0;
    while index < lexemes.len() {
        result = result.merge(rule_statement(&mut index, lexemes), |_, s| {
            statements.push(s)
        });
    }
    result.and_then_wp(|_| Problems::ok(statements))
}

// Returns whether the current lexeme is EOS (end of statement)
// I.e., either end of the input or a newline separator
fn is_eos(index: usize, lexemes: &Lexemes) -> bool {
    match lexemes.get(index) {
        Some(lexeme) => match lexeme.separator {
            Separator::Start | Separator::Nothing => false,
            _ => true,
        },
        None => true,
    }
}

/*

fn advance(&mut self) {
    self.index += 1;
}

fn get_and_advance<'a>(&mut self, lexemes: &'a Lexemes) -> Option<&'a Lexeme> {
    let current = self.index;
    self.advance();
    lexemes.get(current)
}

*/

fn rule_statement(index: &mut usize, lexemes: &Lexemes) -> Result<Statement> {
    rule_expression(index, lexemes).and_then_wp(|e| {
        if !is_eos(*index, lexemes) {
            Error::EndOfStatementExpected.to_err(lexemes.get(*index).unwrap())
        } else {
            Problems::ok(Statement::Expression(e))
        }
    })
}

fn rule_expression(index: &mut usize, lexemes: &Lexemes) -> EResult {
    rule_logterm(index, lexemes)
}

fn binary_rule<O, F>(index: &mut usize, lexemes: &Lexemes, op: O, rule: F) -> EResult
where
    O: Fn(&LexemeData) -> Option<BinaryOp>,
    F: Fn(&mut usize, &Lexemes) -> EResult,
{
    let mut left = rule(index, lexemes);
    while let Some(lexeme) = lexemes.get(*index) {
        if let Some(bop) = op(&lexeme.data) {
            let position = lexeme.position.clone();
            *index += 1;
            left = left
                .merge(rule(index, lexemes), |e1, e2| (e1, e2))
                .and_then_wp(|(expr1, expr2)| {
                    Problems::ok(
                        Expr::Binary(Binary {
                            op: bop,
                            expr1,
                            expr2,
                        })
                        .to_expression(position, None),
                    )
                });
        } else {
            break;
        }
    }
    left
}

fn rule_logterm(index: &mut usize, lexemes: &Lexemes) -> EResult {
    binary_rule(
        index,
        lexemes,
        |k| match k {
            LexemeData::LogicalOr => Some(BinaryOp::Or),
            _ => None,
        },
        rule_logfactor,
    )
}

fn rule_logfactor(index: &mut usize, lexemes: &Lexemes) -> EResult {
    binary_rule(
        index,
        lexemes,
        |k| match k {
            LexemeData::LogicalAnd => Some(BinaryOp::And),
            _ => None,
        },
        rule_equality,
    )
}

fn rule_equality(index: &mut usize, lexemes: &Lexemes) -> EResult {
    binary_rule(
        index,
        lexemes,
        |k| match k {
            LexemeData::Equals => Some(BinaryOp::Eq),
            LexemeData::NotEquals => Some(BinaryOp::NEq),
            _ => None,
        },
        rule_comparison,
    )
}

fn rule_comparison(index: &mut usize, lexemes: &Lexemes) -> EResult {
    binary_rule(
        index,
        lexemes,
        |k| match k {
            LexemeData::Greater => Some(BinaryOp::GT),
            LexemeData::GreaterOrEq => Some(BinaryOp::GE),
            LexemeData::Less => Some(BinaryOp::LT),
            LexemeData::LessOrEq => Some(BinaryOp::LE),
            _ => None,
        },
        rule_term,
    )
}

fn rule_term(index: &mut usize, lexemes: &Lexemes) -> EResult {
    binary_rule(
        index,
        lexemes,
        |k| match k {
            LexemeData::Plus => Some(BinaryOp::Add),
            LexemeData::Minus => Some(BinaryOp::Sub),
            _ => None,
        },
        rule_factor,
    )
}

fn rule_factor(index: &mut usize, lexemes: &Lexemes) -> EResult {
    binary_rule(
        index,
        lexemes,
        |k| match k {
            LexemeData::Star => Some(BinaryOp::Mul),
            LexemeData::Slash => Some(BinaryOp::Div),
            _ => None,
        },
        rule_primary,
    )
}

fn rule_primary(index: &mut usize, lexemes: &Lexemes) -> EResult {
    let current = lexemes.get(*index);
    *index += 1;
    if let Some(lexeme) = current {
        match &lexeme.data {
            LexemeData::Integer(n) => Ok((lexeme.position.clone(), Expr::LitInteger(n.clone()))),
            _ => panic!("TODO: error"),
        }
    } else {
        panic!("TODO: error")
    }
    .and_then(|(position, expr)| {
        Problems::ok(expr.to_expression(position, rule_type_ann(index, lexemes)?.value))
    })
}

fn rule_type_ann(index: &mut usize, lexemes: &Lexemes) -> Result<Option<TypeAnnotation>> {
    if let Some(seplex) = lexemes.get(*index) {
        if LexemeData::TypeAnnSeparator == seplex.data {
            *index += 1;
            if let Some(lexeme) = lexemes.get(*index) {
                if let LexemeData::TSymbol(s) = &lexeme.data {
                    *index += 1;
                    Problems::ok(Some(TypeAnnotation::LocalType(s.clone())))
                } else {
                    Error::TypeAnnotationExpected.to_err(&lexeme)
                }
            } else {
                Error::TypeAnnotationExpected.to_err(&seplex)
            }
        } else {
            Problems::ok(None)
        }
    } else {
        Problems::ok(None)
    }
}

#[derive(Debug)]
enum Error {
    EndOfStatementExpected,
    TypeAnnotationExpected,
}

impl Error {
    fn to_err<T>(self, lexeme: &Lexeme) -> Result<T> {
        self.at(lexeme.position.clone()).to_err()
    }
}

impl ErrorType for Error {
    fn at(self, position: Position) -> problem::Error {
        problem::Error::new(position, self)
    }
}

#[cfg(test)]
mod tests;
