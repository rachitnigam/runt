use colored::Colorize;
use difference;
use difference::{Changeset, Difference};
use std::fmt;

/// Track the mode of difference printing.
#[derive(PartialEq, Debug, Clone)]
enum Mode {
    Same,
    Add,
    Rem,
}

// ======== line number display ==========
#[derive(PartialEq, Debug, Clone)]
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

#[derive(PartialEq, Debug, Clone)]
struct PrintInfo<'a>(Mode, Lineno, Lineno, &'a str);

impl PrintInfo<'_> {
    fn to_diff(&self) -> String {
        let PrintInfo(node, line_a, line_b, line) = self;
        let (diff_sign, col_line) = match node {
            Mode::Add => ("+".green(), line.green()),
            Mode::Rem => ("-".red(), line.red()),
            Mode::Same => {
                let trimmed = line
                    .get(..80)
                    .map(|sl| sl.to_owned() + " ...")
                    .unwrap_or(line.to_string());
                (" ".normal(), trimmed.dimmed())
            }
        };
        format!("{:>5} {:>3}│{} {}\n", line_a, line_b, diff_sign, col_line)
    }
}

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

/// Chunk up the diff so that unchanged lines are only showed in the context
/// of another changed lines.
fn get_chunks<'a>(print_info: &'a [PrintInfo]) -> Vec<Vec<PrintInfo<'a>>> {
    use Mode as M;
    let mut printable: Vec<Vec<PrintInfo>> = Vec::new();

    let ctx_size = 2;
    let mut running = false;
    let mut end_window = ctx_size;
    let mut cur_slice: Vec<PrintInfo> = Vec::new();

    for (idx, node) in print_info.iter().enumerate() {
        // print!("{:>5} {} {:?} - ", running, end_window, node.0);
        // If `running` is false and this is a `Same` node, skip it.
        if !running && node.0 == M::Same {
            // println!("{}", "skip");
        }
        // If this is not a `Same` node and `running` is false, add the
        // last `ctx_size` nodes which are expected to the `Same` and
        // set running to true.
        else if !running && node.0 != M::Same {
            // println!("{}", "b1");
            running = true;
            print_info[idx.saturating_sub(ctx_size)..idx]
                .iter()
                .for_each(|el| cur_slice.push(el.clone()));
            cur_slice.push(node.clone());
        }
        // If `running` is `true` and this node is not `Same`, add this
        // node and set `end_window` = `ctx_size`.
        else if running && node.0 != M::Same {
            // println!("{}", "b2");
            cur_slice.push(node.clone());
            end_window = ctx_size;
        }
        // If `running` is `true` and this node is `Same` and
        // `end_window` is not zero, decrement `end_window` and add
        // this node.
        else if running && node.0 == M::Same && end_window != 0 {
            // println!("{}", "b3");
            end_window -= 1;
            cur_slice.push(node.clone());
        }
        // If `end_window` is zero, set `running` to `false`.
        else if end_window == 0 {
            // println!("{}", "b4");
            running = false;
            printable.push(cur_slice);
            cur_slice = Vec::new();
        } else {
            // println!("{}", "imp");
        }
    }
    if !cur_slice.is_empty() {
        printable.push(cur_slice);
    }
    printable
}

/// Generate a rich string representation to show diffs with line number
/// information.
pub fn gen_diff(org: &str, new: &str) -> String {
    // Generate a changeset for the strings and get line number information.
    let changes = &Changeset::new(org, new, "\n");
    let print_info = diff_with_lineno(changes);
    let print_chunks = get_chunks(&print_info);

    let mut str_buf = String::new();
    str_buf.push_str(format!("{:>9}~\n", " ").as_str());
    for chunk in print_chunks {
        for info in chunk {
            str_buf.push_str(&info.to_diff());
        }
        str_buf.push_str(format!("{:>9}~\n", " ").as_str());
    }

    str_buf.trim_end().to_string()
}
