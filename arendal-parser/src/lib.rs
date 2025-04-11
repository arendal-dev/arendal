mod lexer;

use ast::{
    input::StringInput,
    problem::{Problem, Problems, Result, Severity},
    stmt::{Expr, Expression, Statement},
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

    fn get_and_advance(&mut self) -> Option<&Lexeme> {
        let current = self.index;
        self.index += 1;
        self.lexemes.get(current)
    }

    fn rule_statement(&mut self) -> PResult<Statement> {
        let result = self.rule_primary().map(Expression::to_statement);
        if !self.is_eos() {
            self.problems.add(
                self.peek().unwrap().position.clone(),
                Error::EndOfStatementExpected,
            );
        }
        result
    }

    fn rule_primary(&mut self) -> EResult {
        if let Some(lexeme) = self.get_and_advance() {
            match &lexeme.data {
                LexemeData::Integer(n) => {
                    Ok(Expr::LitInteger(n.clone()).to_expression(&lexeme.position))
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
}

impl Problem for Error {
    fn severity(&self) -> Severity {
        Severity::Error
    }
}

#[cfg(test)]
mod tests;
