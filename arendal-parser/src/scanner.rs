use arendal_error::{Result, Error};

#[derive(Debug)]
struct Token;


#[derive(Debug)]
struct Indentation {
    tabs : usize,
    spaces : usize,
}

#[derive(Debug)]
struct Line {
    number : usize,
    indentation : Indentation,
    tokens : Vec<Token>,
}

fn scan(input : &str) -> Result<Vec<Line>> {
    let mut lines : Vec<Line> = Vec::new();
    for (i, line) in input.lines().enumerate() {
        lines.push(scan_line(i, line))
    }
    Ok(lines)
}

fn scan_line(index : usize, input : &str) -> Line {
    let mut tabs : usize = 0;
    let mut spaces : usize = 0;
    Line {
        number: index + 1,
        indentation: Indentation { tabs : 0, spaces: 0 },
        tokens : scan_line_tail(input),
    }
}

fn get_indentation(input : &str) -> Result<(Indentation, &str)> {
    let mut tabs : usize = 0;
    let mut spaces : usize = 0;
    let mut tail = "";
    for (i, c) in input.char_indices() {
        if c == '\t' {
            if (spaces > 0){
                return Err(Error{});
            }
            tabs += 1;
        } else if c == ' ' {
            spaces += 1;
        } else {
            tail = &input[i..];
            break;
        }
    }
    Ok((Indentation {
        tabs: tabs,
        spaces: spaces,
    }, tail))

}

fn scan_line_tail(input : &str) -> Vec<Token> {
    let mut tokens : Vec<Token> = Vec::new();
    tokens
}

#[cfg(test)]
mod tests {

    use super::Indentation;
    use super::get_indentation;

    fn indentation_ok(input : &str, tabs : usize, spaces : usize, tail : &str) {
        match get_indentation(input) {
            Ok((Indentation {
                tabs,
                spaces,
            }, tail)) => assert!(true),
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