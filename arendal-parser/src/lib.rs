mod lexer;

use ast::{
    AST, Binary, Expr, Expression, Q, Statement, TypeExpr,
    common::BinaryOp,
    input::StringInput,
    problem::{ErrorType, Output, Result},
};
use lexer::{Lexeme, LexemeData, Lexemes, Separator};

pub fn parse(input: &str) -> Output<AST> {
    parse_statements(input).and_then(|stmts| stmts_to_ast(stmts))
}

fn stmts_to_ast(stmts: Vec<Statement>) -> Output<AST> {
    let mut output: Output<AST> = Output::new();
    let mut ast = AST { expression: None };
    for s in stmts {
        match s {
            Statement::Expression(e) => match ast.expression {
                None => ast.expression = Some(e),
                _ => output.add_error(Error::OnlyOneExpressionAllowed.at(e.position)),
            },
        }
    }
    output.replace(ast);
    output
}

pub fn parse_statements(input: &str) -> Output<Vec<Statement>> {
    let input = StringInput::from_str(input);
    lexer::lex(input).and_then(|lexemes| parse_lexemes(&lexemes))
}

type EResult = Output<Expression>;

fn parse_lexemes(lexemes: &Lexemes) -> Output<Vec<Statement>> {
    let mut output: Output<Vec<Statement>> = Output::empty();
    let mut index: usize = 0;
    while index < lexemes.len() {
        output.add_output(rule_statement(&mut index, lexemes));
    }
    output
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

fn rule_statement(index: &mut usize, lexemes: &Lexemes) -> Output<Statement> {
    rule_expression(index, lexemes).and_then(|e| {
        if !is_eos(*index, lexemes) {
            Error::EndOfStatementExpected.to_output(lexemes.get(*index).unwrap())
        } else {
            Output::ok(Statement::Expression(e))
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
                .merge_to_tuple(rule(index, lexemes))
                .and_then(|(expr1, expr2)| {
                    Output::ok(
                        Expr::Binary(Binary {
                            op: bop,
                            expr1: expr1.into(),
                            expr2: expr2.into(),
                        })
                        .to_expression(position, None),
                    )
                })
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
            LexemeData::Integer(n) => {
                Output::ok((lexeme.position.clone(), Expr::LitInteger(n.clone())))
            }
            _ => panic!("TODO: error"),
        }
    } else {
        panic!("TODO: error")
    }
    .and_then(|(position, expr)| {
        rule_type_ann(index, lexemes).map(|type_expr| expr.to_expression(position, type_expr))
    })
}

fn rule_type_ann(index: &mut usize, lexemes: &Lexemes) -> Output<Option<TypeExpr>> {
    if let Some(seplex) = lexemes.get(*index) {
        if LexemeData::TypeAnnSeparator == seplex.data {
            *index += 1;
            if let Some(lexeme) = lexemes.get(*index) {
                if let LexemeData::TSymbol(s) = &lexeme.data {
                    *index += 1;
                    Output::ok(Some(TypeExpr::Type(Q::of(s.clone()))))
                } else {
                    Error::TypeAnnotationExpected.to_output(&lexeme)
                }
            } else {
                Error::TypeAnnotationExpected.to_output(&seplex)
            }
        } else {
            Output::ok(None)
        }
    } else {
        Output::ok(None)
    }
}

#[derive(Debug)]
enum Error {
    EndOfStatementExpected,
    TypeAnnotationExpected,
    OnlyOneExpressionAllowed,
    UnexpectedStatement,
}

impl Error {
    fn to_output<T>(self, lexeme: &Lexeme) -> Output<T> {
        self.at(lexeme.position.clone()).into()
    }
}

impl ErrorType for Error {}

#[cfg(test)]
mod tests;
