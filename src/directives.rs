//! Directives: `tbf …` lines in the header, the lines above the first command.

use crate::error::{Error, Pos};
use crate::settings::{Eof, Settings, Width};

pub const COMMANDS: &[u8] = b"+-<>[].,";
const KNOWN: &str = "cell, eof";

/// Reads the header's directives. A line holding any command ends the header.
pub fn read(src: &[u8]) -> Result<Settings, Error> {
    let mut settings = Settings::default();
    let mut cell_line = None;
    let mut eof_line = None;

    for (i, line) in src.split(|&b| b == b'\n').enumerate() {
        if line.iter().any(|b| COMMANDS.contains(b)) {
            break;
        }
        let words = words(line);
        let Some(&(col, first)) = words.first() else { continue };
        if first != b"tbf" {
            continue;
        }
        let line_no = i + 1;
        let pos = |col| Pos { line: line_no, col };

        let Some(&(name_col, name)) = words.get(1) else {
            return Err(Error::at(format!("empty directive; known: {KNOWN}"), pos(col)));
        };
        let name_text = String::from_utf8_lossy(name);
        let (seen, values): (&mut Option<usize>, &str) = match name {
            b"cell" => (&mut cell_line, "u8, u16, u32"),
            b"eof" => (&mut eof_line, "0, keep, max"),
            _ => {
                return Err(Error::at(
                    format!("unknown directive \"{name_text}\"; known: {KNOWN}"),
                    pos(name_col),
                ));
            }
        };
        if let Some(first_line) = *seen {
            return Err(Error::at(
                format!("directive \"{name_text}\" given twice, on lines {first_line} and {line_no}"),
                pos(name_col),
            ));
        }
        *seen = Some(line_no);

        let value = match &words[2..] {
            [v] => v,
            [] => {
                return Err(Error::at(
                    format!("directive \"{name_text}\" needs a value; known: {values}"),
                    pos(name_col),
                ));
            }
            [_, extra, ..] => {
                return Err(Error::at(
                    format!("directive \"{name_text}\" takes one value; known: {values}"),
                    pos(extra.0),
                ));
            }
        };
        let unknown = || {
            Error::at(
                format!(
                    "unknown {name_text} \"{}\"; known: {values}",
                    String::from_utf8_lossy(value.1)
                ),
                pos(value.0),
            )
        };
        match name {
            b"cell" => {
                settings.cell = match value.1 {
                    b"u8" => Width::U8,
                    b"u16" => Width::U16,
                    b"u32" => Width::U32,
                    _ => return Err(unknown()),
                }
            }
            _ => {
                settings.eof = match value.1 {
                    b"0" => Eof::Zero,
                    b"keep" => Eof::Keep,
                    b"max" => Eof::Max,
                    _ => return Err(unknown()),
                }
            }
        }
    }
    Ok(settings)
}

/// The line's words, each with its 1-based column.
fn words(line: &[u8]) -> Vec<(usize, &[u8])> {
    let mut out = Vec::new();
    let mut start = None;
    for (i, &b) in line.iter().chain(std::iter::once(&b' ')).enumerate() {
        match (b.is_ascii_whitespace(), start) {
            (false, None) => start = Some(i),
            (true, Some(s)) => {
                out.push((s + 1, &line[s..i]));
                start = None;
            }
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err(src: &str) -> String {
        read(src.as_bytes()).unwrap_err().render("t.bf")
    }

    #[test]
    fn defaults_without_directives() {
        assert_eq!(read(b"+.").unwrap(), Settings::default());
    }

    #[test]
    fn reads_both() {
        let s = read(b"my program\ntbf cell u16\n  tbf eof max\r\n+[.]").unwrap();
        assert_eq!(s, Settings { cell: Width::U16, eof: Eof::Max });
    }

    #[test]
    fn a_directive_after_the_first_command_is_a_comment() {
        assert_eq!(read(b"+\ntbf cel u16").unwrap(), Settings::default());
        assert_eq!(read(b"tbf cell u16 +").unwrap(), Settings::default());
    }

    #[test]
    fn errors() {
        assert_eq!(err("tbf cel u16"), "error: unknown directive \"cel\"; known: cell, eof\n  at t.bf:1:5");
        assert_eq!(
            err("tbf cell u8\ntbf cell u32"),
            "error: directive \"cell\" given twice, on lines 1 and 2\n  at t.bf:2:5"
        );
        assert_eq!(err("tbf cell u64"), "error: unknown cell \"u64\"; known: u8, u16, u32\n  at t.bf:1:10");
        assert_eq!(err("tbf eof"), "error: directive \"eof\" needs a value; known: 0, keep, max\n  at t.bf:1:5");
        assert_eq!(err("tbf eof 0 keep"), "error: directive \"eof\" takes one value; known: 0, keep, max\n  at t.bf:1:11");
        assert_eq!(err(" tbf"), "error: empty directive; known: cell, eof\n  at t.bf:1:2");
    }
}
