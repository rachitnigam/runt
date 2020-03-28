use difference;
use difference::{Changeset, Difference};
use std::fmt;
use std::io::Write;
use colored::Colorize;

/// Track the mode of difference printing.
#[derive(PartialEq, Debug)]
enum Mode {
    Same,
    Add,
    Rem,
}

// ======== line number display ==========
#[derive(PartialEq, Debug)]
struct Lineno(Option<usize>);

impl fmt::Display for Lineno {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            None => f.pad(""),
            Some(lineno) => fmt::Display::fmt(&lineno.to_string().dimmed(), f),
        }
    }
}
// =======================================

#[derive(PartialEq, Debug)]
struct PrintInfo<'a>(Mode, Lineno, Lineno, &'a str);

/// Given a changeset, generate a vector with a diff representation that tracks
/// line numbers.
fn diff_with_lineno(changes: &Changeset) -> Vec<PrintInfo> {
    // Track line number for original and new strings.
    let mut line_a = 0;
    let mut line_b = 0;

    changes
        .diffs
        .iter()
        .flat_map(|diff| match diff {
            // When there is no difference between lines, increase both the
            // line numbers.
            Difference::Same(x) => x
                .split('\n')
                .map(|line| {
                    line_a += 1;
                    line_b += 1;
                    PrintInfo(
                        Mode::Same,
                        Lineno(Some(line_a)),
                        Lineno(Some(line_b)),
                        line.trim_end(),
                    )
                })
                .collect::<Vec<PrintInfo>>(),
            // When a new line was added, increase the line number of the new
            // string.
            Difference::Add(x) => x
                .split('\n')
                .map(|line| {
                    line_b += 1;
                    PrintInfo(
                        Mode::Add,
                        Lineno(None),
                        Lineno(Some(line_b)),
                        line.trim_end(),
                    )
                })
                .collect::<Vec<PrintInfo>>(),
            // When a line was removed, increase the line number of the old
            // string.
            Difference::Rem(x) => x
                .split('\n')
                .map(|line| {
                    line_a += 1;
                    PrintInfo(
                        Mode::Rem,
                        Lineno(Some(line_a)),
                        Lineno(None),
                        line.trim_end(),
                    )
                })
                .collect::<Vec<PrintInfo>>(),
        })
        .collect()
}

/// Generate a rich string representation to show diffs with line number
/// information.
pub fn gen_diff<'a>(
    org: &str,
    new: &str
) -> Result<String, std::io::Error> {
    use colored::*;
    // Generate a changeset for the strings and get line number information.

    let changes = &Changeset::new(org, new, "\n");
    let print_info = diff_with_lineno(changes);
    let mut str_buf = String::new();

    for diff in print_info {
        match diff {
            PrintInfo(Mode::Add, line_a, line_b, line) => {
                str_buf.push_str(format!(
                    "{:>3} {:>3}│{}{}\n",
                    line_a,
                    line_b,
                    "+".green(),
                    line.green()
                ).as_str())
            }
            PrintInfo(Mode::Rem, line_a, line_b, line) => {
                str_buf.push_str(format!(
                    "{:>3} {:>3}│{}{}\n",
                    line_a,
                    line_b,
                    "-".red(),
                    line.red()
                ).as_str())
            }
            PrintInfo(Mode::Same, line_a, line_b, line) => {
                str_buf.push_str(format!(
                    "{:>3} {:>3}│{}{}\n",
                    line_a,
                    line_b,
                    " ",
                    line.dimmed()
                ).as_str())
            }
        }
    }

    Ok(str_buf.trim_end().to_string())
}
