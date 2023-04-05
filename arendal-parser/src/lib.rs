pub mod lexer;
pub mod parser;
pub mod tokenizer;

use std::fmt;

use core::error::Loc;
use core::{ArcStr, Substr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enclosure {
    Parens,
    Square,
    Curly,
}
