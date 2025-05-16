mod lexer;

use ast::{
    BinaryOp, EMPTY,
    input::StringInput,
    problem::{self, ErrorType, Problems, ProblemsResult, Result},
    stmt::{Binary, Expr, Expression, Statement, TypeAnnotation},
};
use lexer::{Lexeme, LexemeData, Lexemes, Separator};

pub fn parse(input: &str) -> Result<Vec<Statement>> {
    let input = StringInput::from_str(input);
    let (problems, lexemes) = lexer::lex(input)?.to_problems();
    Parser { index: 0, problems }.parse_statements(&lexemes)
}

struct Parser {
    index: usize,
    problems: Problems,
}

type EResult = Result<Expression>;

impl Parser {
    fn parse_statements(mut self, lexemes: &Lexemes) -> Result<Vec<Statement>> {
        let mut statements = Vec::<Statement>::default();
        let mut result = problem::ok(());
        while self.index < lexemes.len() {
            result = result.merge(self.rule_statement(lexemes), |_, s| statements.push(s));
        }
        self.problems.to_result(statements)
    }

    // Returns whether the current lexeme is EOS (end of statement)
    // I.e., either end of the input or a newline separator
    fn is_eos(&self, lexemes: &Lexemes) -> bool {
        match lexemes.get(self.index) {
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

    fn rule_statement(&mut self, lexemes: &Lexemes) -> Result<Statement> {
        self.rule_expression(lexemes).and_then_wp(|e| {
            if !self.is_eos(lexemes) {
                Error::EndOfStatementExpected.at(lexemes.get(self.index).unwrap())
            } else {
                problem::ok(Statement::Expression(e))
            }
        })
    }

    fn rule_expression(&mut self, lexemes: &Lexemes) -> EResult {
        self.rule_logterm(lexemes)
    }

    fn binary_rule<O, F>(&mut self, lexemes: &Lexemes, op: O, rule: F) -> EResult
    where
        O: Fn(&LexemeData) -> Option<BinaryOp>,
        F: Fn(&mut Parser, &Lexemes) -> EResult,
    {
        let mut left = rule(self, lexemes);
        while let Some(lexeme) = lexemes.get(self.index) {
            if let Some(bop) = op(&lexeme.data) {
                let position = lexeme.position.clone();
                self.index += 1;
                left = left
                    .merge(rule(self, lexemes), |e1, e2| (e1, e2))
                    .and_then_wp(|(expr1, expr2)| {
                        problem::ok(
                            Expr::Binary(Binary {
                                op: bop,
                                expr1,
                                expr2,
                            })
                            .to_expression(position, None, EMPTY),
                        )
                    });
            } else {
                break;
            }
        }
        left
    }

    fn rule_logterm(&mut self, lexemes: &Lexemes) -> EResult {
        self.binary_rule(
            lexemes,
            |k| match k {
                LexemeData::LogicalOr => Some(BinaryOp::Or),
                _ => None,
            },
            Self::rule_logfactor,
        )
    }

    fn rule_logfactor(&mut self, lexemes: &Lexemes) -> EResult {
        self.binary_rule(
            lexemes,
            |k| match k {
                LexemeData::LogicalAnd => Some(BinaryOp::And),
                _ => None,
            },
            Self::rule_equality,
        )
    }

    fn rule_equality(&mut self, lexemes: &Lexemes) -> EResult {
        self.binary_rule(
            lexemes,
            |k| match k {
                LexemeData::Equals => Some(BinaryOp::Eq),
                LexemeData::NotEquals => Some(BinaryOp::NEq),
                _ => None,
            },
            Self::rule_comparison,
        )
    }

    fn rule_comparison(&mut self, lexemes: &Lexemes) -> EResult {
        self.binary_rule(
            lexemes,
            |k| match k {
                LexemeData::Greater => Some(BinaryOp::GT),
                LexemeData::GreaterOrEq => Some(BinaryOp::GE),
                LexemeData::Less => Some(BinaryOp::LT),
                LexemeData::LessOrEq => Some(BinaryOp::LE),
                _ => None,
            },
            Self::rule_term,
        )
    }

    fn rule_term(&mut self, lexemes: &Lexemes) -> EResult {
        self.binary_rule(
            lexemes,
            |k| match k {
                LexemeData::Plus => Some(BinaryOp::Add),
                LexemeData::Minus => Some(BinaryOp::Sub),
                _ => None,
            },
            Self::rule_factor,
        )
    }

    fn rule_factor(&mut self, lexemes: &Lexemes) -> EResult {
        self.binary_rule(
            lexemes,
            |k| match k {
                LexemeData::Star => Some(BinaryOp::Mul),
                LexemeData::Slash => Some(BinaryOp::Div),
                _ => None,
            },
            Self::rule_primary,
        )
    }

    fn rule_primary(&mut self, lexemes: &Lexemes) -> EResult {
        let current = lexemes.get(self.index);
        self.index += 1;
        if let Some(lexeme) = current {
            match &lexeme.data {
                LexemeData::Integer(n) => {
                    Ok((lexeme.position.clone(), Expr::LitInteger(n.clone())))
                }
                _ => panic!("TODO: error"),
            }
        } else {
            panic!("TODO: error")
        }
        .and_then(|(position, expr)| {
            problem::ok(expr.to_expression(position, self.rule_type_ann(lexemes)?.value, EMPTY))
        })
    }

    fn rule_type_ann(&mut self, lexemes: &Lexemes) -> Result<Option<TypeAnnotation>> {
        if let Some(seplex) = lexemes.get(self.index) {
            if LexemeData::TypeAnnSeparator == seplex.data {
                self.index += 1;
                if let Some(lexeme) = lexemes.get(self.index) {
                    if let LexemeData::TSymbol(s) = &lexeme.data {
                        self.index += 1;
                        problem::ok(Some(TypeAnnotation::LocalType(s.clone())))
                    } else {
                        Error::TypeAnnotationExpected.at(&lexeme)
                    }
                } else {
                    Error::TypeAnnotationExpected.at(&seplex)
                }
            } else {
                problem::ok(None)
            }
        } else {
            problem::ok(None)
        }
    }

    fn add_problem_at<T: ErrorType + 'static>(&mut self, lexeme: &Lexeme, problem: T) {
        self.problems.add_error(lexeme.position.clone(), problem);
    }
}

#[derive(Debug)]
enum Error {
    EndOfStatementExpected,
    TypeAnnotationExpected,
}

impl Error {
    fn at<T>(self, lexeme: &Lexeme) -> Result<T> {
        problem::error(lexeme.position.clone(), self)
    }
}

impl ErrorType for Error {}

#[cfg(test)]
mod tests;
