//! Source to a tree of runs, I/O and loops. Fold 1: a run of `+`/`-` or `<`/`>` is one node.

use crate::error::{Error, Pos};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    /// Adds this much to the current cell (wrapped later, by width).
    Add(i64),
    Move(i64),
    Out,
    In,
    Loop(Vec<Node>),
}

pub fn parse(src: &[u8]) -> Result<Vec<Node>, Error> {
    let mut stack: Vec<(Vec<Node>, Pos)> = Vec::new();
    let mut block = Vec::new();
    let (mut line, mut col) = (1, 0);

    for &b in src {
        col += 1;
        let here = Pos { line, col };
        match b {
            b'+' => add(&mut block, 1),
            b'-' => add(&mut block, -1),
            b'>' => step(&mut block, 1),
            b'<' => step(&mut block, -1),
            b'.' => block.push(Node::Out),
            b',' => block.push(Node::In),
            b'[' => stack.push((std::mem::take(&mut block), here)),
            b']' => {
                let Some((outer, _)) = stack.pop() else {
                    return Err(Error::at("this ] has no [", here));
                };
                let body = std::mem::replace(&mut block, outer);
                block.push(Node::Loop(body));
            }
            b'\n' => {
                line += 1;
                col = 0;
            }
            _ => {}
        }
    }
    if let Some((_, open)) = stack.pop() {
        return Err(Error::at("this [ is never closed", open));
    }
    Ok(block)
}

fn add(block: &mut Vec<Node>, d: i64) {
    match block.last_mut() {
        Some(Node::Add(n)) => {
            *n += d;
            if *n == 0 {
                block.pop();
            }
        }
        _ => block.push(Node::Add(d)),
    }
}

fn step(block: &mut Vec<Node>, d: i64) {
    match block.last_mut() {
        Some(Node::Move(n)) => {
            *n += d;
            if *n == 0 {
                block.pop();
            }
        }
        _ => block.push(Node::Move(d)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Node::*;

    #[test]
    fn runs_fold() {
        assert_eq!(parse(b"+++--a>>>.<,").unwrap(), vec![Add(1), Move(3), Out, Move(-1), In]);
        assert_eq!(parse(b"+-><").unwrap(), vec![]);
    }

    #[test]
    fn loops_nest() {
        assert_eq!(parse(b"[-[>]]").unwrap(), vec![Loop(vec![Add(-1), Loop(vec![Move(1)])])]);
    }

    #[test]
    fn unmatched() {
        assert_eq!(parse(b"+\n  ]").unwrap_err().render("t.bf"), "error: this ] has no [\n  at t.bf:2:3");
        assert_eq!(parse(b"[[]").unwrap_err().render("t.bf"), "error: this [ is never closed\n  at t.bf:1:1");
    }
}
