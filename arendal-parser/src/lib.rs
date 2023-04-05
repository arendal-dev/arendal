pub mod lexer;
pub mod parser;
pub mod tokenizer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enclosure {
    Parens,
    Square,
    Curly,
}
