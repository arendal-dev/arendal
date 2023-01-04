mod errors;

use arendal_error::{Error, Result, merge_options};
use errors::*;

#[derive(Debug)]
struct Token;

#[derive(Debug)]
struct Indentation {
    tabs: usize,
    spaces: usize,
}

#[inline]
fn indentation(tabs : usize, spaces: usize) -> Indentation {
    Indentation { tabs, spaces }
}

#[derive(Debug)]
struct Line<'a> {
    number: usize,
    indentation: Indentation,
    line: &'a str,
    tokens: Vec<Token>,
}

fn line(number: usize, indentation: Indentation, line: &str) -> Line {
    Line {
        number,
        indentation,
        line,
        tokens: Vec::new(),
    }
}

fn scan(input: &str) -> Result<Vec<Line>> {
    let mut lines : Vec<Line> = Vec::new();
    let error : Option<Error> = input.lines().enumerate().fold(None, |error_acc, (i, line)| {
        let (line, error) = scan_line(i, line);
        lines.push(line);
        merge_options(error_acc, error)
    });
    match error {
        None => Ok(lines),
        Some(e) => Err(e),
    }
}

fn scan_line(index: usize, input: &str) -> (Line, Option<Error>) {
    let number = index + 1;
    match get_indentation(input) {
        Ok((indentation, tail)) => scan_indented_line(line(number, indentation, tail)),
        Err(error) => (line(number, indentation(0, 0), input), Some(error))
    }
}

fn get_indentation(input: &str) -> Result<(Indentation, &str)> {
    let mut tabs: usize = 0;
    let mut spaces: usize = 0;
    let mut tail = "";
    for (i, c) in input.char_indices() {
        if c == '\t' {
            if (spaces > 0) {
                return Err(indentation_error());
            }
            tabs += 1;
        } else if c == ' ' {
            spaces += 1;
        } else {
            tail = &input[i..];
            break;
        }
    }
    Ok((
        Indentation {
            tabs: tabs,
            spaces: spaces,
        },
        tail,
    ))
}

struct Scanner<'a> {
    line: Line<'a>,
    graphemes: Vec<(usize, &'a str)>,
    column: usize,
    index: usize,
}

impl<'a> Scanner<'a> {
    fn scan(&mut self) -> Option<Error> {
        None
    }
}

fn scan_indented_line(line : Line) -> (Line, Option<Error>) {
    /*
    let mut scanner = Scanner {
        graphemes: unicode_segmentation::UnicodeSegmentation::grapheme_indices(line, true)
            .collect(),
        column: 1 + (indentation.tabs) * 4 + indentation.spaces,
        index: 0,
    };
    scanner.scan()
    */
    (line, None)
}

#[cfg(test)]
mod tests {

    use super::get_indentation;
    use super::Indentation;

    fn indentation_ok(input: &str, tabs: usize, spaces: usize, tail: &str) {
        match get_indentation(input) {
            Ok((Indentation { tabs, spaces }, tail)) => assert!(true),
            Ok(_) => assert!(false, "Unexpected result"),
            Err(_) => assert!(false, "Indentation error"),
        }
    }

    #[test]
    fn indentation() {
        indentation_ok("1", 0, 0, "1");
        indentation_ok("\t1", 1, 0, "1");
        indentation_ok("\t\t1", 2, 0, "1");
        indentation_ok("\t\t 1", 2, 1, "1");
        indentation_ok("\t\t  1", 2, 2, "1");
    }
}
