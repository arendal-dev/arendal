mod lexer;

use ast::{
    BinaryOp, EMPTY,
    input::StringInput,
    position::Position,
    problem::{Problem, Problems, Result, Severity},
    stmt::{Binary, Expr, Expression, Statement, TypeAnnotation},
};
use lexer::{Lexeme, LexemeData, Lexemes, Separator};

pub fn parse(input: &str) -> Result<Vec<Statement>> {
    let input = StringInput::from_str(input);
    let (lexemes, problems) = lexer::lex(input)?;
    Parser {
        lexemes,
        index: 0,
        problems,
    }
    .parse_statements()
}

struct Parser {
    lexemes: Lexemes,
    index: usize,
    problems: Problems,
}

type PResult<T> = std::result::Result<T, ()>;
type EResult = PResult<Expression>;

impl Parser {
    fn parse_statements(mut self) -> Result<Vec<Statement>> {
        let mut statements: Vec<Statement> = Vec::default();
        while !self.is_done() {
            let _ = self.rule_statement().map(|s| statements.push(s));
        }
        self.problems.to_result(statements)
    }

    fn is_done(&self) -> bool {
        self.index >= self.lexemes.len()
    }

    // Returns whether the current lexeme is EOS (end of statement)
    // I.e., either end of the input or a newline separator
    fn is_eos(&self) -> bool {
        match self.peek() {
            Some(lexeme) => match lexeme.separator {
                Separator::Start | Separator::Nothing => false,
                _ => true,
            },
            None => true,
        }
    }

    fn peek(&self) -> Option<&Lexeme> {
        self.lexemes.get(self.index)
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn get_and_advance(&mut self) -> Option<&Lexeme> {
        let current = self.index;
        self.advance();
        self.lexemes.get(current)
    }

    fn rule_statement(&mut self) -> PResult<Statement> {
        let result = self.rule_expression().map(|e| Statement::Expression(e));
        if !self.is_eos() {
            self.problems.add(
                self.peek().unwrap().position.clone(),
                Error::EndOfStatementExpected,
            );
        }
        result
    }

    fn rule_expression(&mut self) -> EResult {
        self.rule_logfactor()
    }

    fn binary_rule<O, F>(&mut self, op: O, rule: F) -> EResult
    where
        O: Fn(&LexemeData) -> Option<BinaryOp>,
        F: Fn(&mut Parser) -> EResult,
    {
        let mut left = rule(self)?;
        while let Some(bop) = self.peek().and_then(|l| op(&l.data)) {
            let position = self.peek().unwrap().position.clone();
            self.advance();
            let right = rule(self)?;
            left = Expr::Binary(Binary {
                op: bop,
                expr1: left,
                expr2: right,
            })
            .to_expression(position, TypeAnnotation::None, EMPTY)
        }
        Ok(left)
    }

    fn rule_logterm(&mut self) -> EResult {
        self.binary_rule(
            |k| match k {
                LexemeData::LogicalOr => Some(BinaryOp::Or),
                _ => None,
            },
            Self::rule_logfactor,
        )
    }

    fn rule_logfactor(&mut self) -> EResult {
        self.binary_rule(
            |k| match k {
                LexemeData::LogicalAnd => Some(BinaryOp::And),
                _ => None,
            },
            Self::rule_equality,
        )
    }

    fn rule_equality(&mut self) -> EResult {
        self.binary_rule(
            |k| match k {
                LexemeData::Equals => Some(BinaryOp::Eq),
                LexemeData::NotEquals => Some(BinaryOp::NEq),
                _ => None,
            },
            Self::rule_comparison,
        )
    }

    fn rule_comparison(&mut self) -> EResult {
        self.binary_rule(
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

    fn rule_term(&mut self) -> EResult {
        self.binary_rule(
            |k| match k {
                LexemeData::Plus => Some(BinaryOp::Add),
                LexemeData::Minus => Some(BinaryOp::Sub),
                _ => None,
            },
            Self::rule_factor,
        )
    }

    fn rule_factor(&mut self) -> EResult {
        self.binary_rule(
            |k| match k {
                LexemeData::Star => Some(BinaryOp::Mul),
                LexemeData::Slash => Some(BinaryOp::Div),
                _ => None,
            },
            Self::rule_ann_primary,
        )
    }

    fn rule_ann_primary(&mut self) -> EResult {
        let (position, expr) = self.rule_primary()?;
        let mut type_ann = TypeAnnotation::None;
        if let Some(seplex) = self.peek().cloned() {
            if LexemeData::TypeAnnSeparator == seplex.data {
                self.advance();
                if let Some(lexeme) = self.peek().cloned() {
                    if let LexemeData::TSymbol(s) = lexeme.data {
                        type_ann = TypeAnnotation::LocalType(s);
                    } else {
                        self.add_problem_at(&lexeme, Error::TypeAnnotationExpected);
                        return Err(());
                    }
                } else {
                    self.add_problem_at(&seplex, Error::TypeAnnotationExpected);
                    return Err(());
                }
            }
        }
        Ok(expr.to_expression(position, type_ann, EMPTY))
    }

    fn rule_primary(&mut self) -> PResult<(Position, Expr)> {
        if let Some(lexeme) = self.get_and_advance() {
            match &lexeme.data {
                LexemeData::Integer(n) => {
                    Ok((lexeme.position.clone(), Expr::LitInteger(n.clone())))
                }
                _ => panic!("TODO: error"),
            }
        } else {
            panic!("TODO: error")
        }
    }

    fn add_problem_at<T: Problem + 'static>(&mut self, lexeme: &Lexeme, problem: T) {
        self.problems.add(lexeme.position.clone(), problem);
    }
}

#[derive(Debug)]
enum Error {
    EndOfStatementExpected,
    TypeAnnotationExpected,
}

impl Problem for Error {
    fn severity(&self) -> Severity {
        Severity::Error
    }
}

#[cfg(test)]
mod tests;
