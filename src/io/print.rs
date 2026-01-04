
use std::collections::HashMap;

use itertools::Itertools;

use crate::layout::{Cell, House, Known};
use crate::puzzle::{Action, Board, Verdict};
use crate::symbols::{GIVEN, MISSING};


pub enum PrintMode<'a> {
    Givens,
    KnownValues,
    Candidate(Known),
    AllCandidates,
    AllCandidatesWithHighlight(&'a Action),
}



pub fn print_board(board: &Board, mode: PrintMode<'_>) {
    let lines = write_board(board, mode);
    for line in lines {
        println!("{}", line);
    }
}

pub fn write_board(board: &Board, mode: PrintMode<'_>) -> Vec<String> {
    match mode {
        PrintMode::Givens => add_single_value_labels(write_givens(board)),
        PrintMode::KnownValues => add_single_value_labels(write_known_values(board)),
        PrintMode::Candidate(k) => {
            add_single_value_labels(write_candidate(board, k))
        }
        PrintMode::AllCandidates => {
            add_all_candidates_labels(write_candidates(board))
        }
        PrintMode::AllCandidatesWithHighlight(action) => {
            add_all_candidates_labels(
                write_candidates_with_highlight(board, action.collect_verdicts()),
            )
        }
    }
}



pub fn print_givens(board: &Board) {
    print_board(board, PrintMode::Givens);
}

pub fn print_known_values(board: &Board) {
    print_board(board, PrintMode::KnownValues);
}

pub fn print_candidate(board: &Board, candidate: Known) {
    print_board(board, PrintMode::Candidate(candidate));
}

pub fn print_all_and_single_candidates(board: &Board) {
    actually_print_all_and_single_candidates(
        write_candidates(board),
        Known::iter()
            .map(|k| write_candidate(board, k))
            .collect_vec(),
    );
}

pub fn print_all_and_single_candidates_with_highlight(
    board: &Board,
    action: &Action,
) {
    actually_print_all_and_single_candidates(
        write_candidates_with_highlight(board, action.collect_verdicts()),
        Known::iter()
            .map(|k| {
                write_candidate_with_highlight(
                    board,
                    k,
                    action.collect_verdicts_for_known(k),
                )
            })
            .collect_vec(),
    );
}



fn write_givens(board: &Board) -> Vec<String> {
    write_single_value(|cell, line| {
        let value = board.value(cell);
        if value.is_unknown() {
            line.push(' ');
        } else if board.is_given(cell) {
            line.push(value.label());
        } else {
            line.push(MISSING);
        }
    })
}

fn write_known_values(board: &Board) -> Vec<String> {
    write_single_value(|cell, line| {
        let value = board.value(cell);
        if value.is_unknown() {
            line.push(' ');
        } else {
            line.push(value.label());
        }
    })
}

fn write_candidate(board: &Board, candidate: Known) -> Vec<String> {
    write_single_value(|cell, line| {
        if board.is_candidate(cell, candidate) {
            line.push(GIVEN);
        } else {
            let value = board.value(cell);
            if value.is_unknown() {
                line.push(' ');
            } else if value == candidate.value() {
                line.push(value.label());
            } else {
                line.push(' ');
            }
        }
    })
}




fn write_candidate_with_highlight(
    board: &Board,
    candidate: Known,
    verdicts: HashMap<Cell, Verdict>,
) -> Vec<String> {
    write_single_value(|cell, line| {
        let verdict = verdicts.get(&cell).unwrap_or(&Verdict::None);

        if board.is_candidate(cell, candidate) {
            line.push_str(verdict.color_char(GIVEN).as_str());
        } else {
            let value = board.value(cell);
            if value.is_unknown() {
                line.push(' ');
            } else if value == candidate.value() {
                line.push_str(verdict.color_char(value.label()).as_str());
            } else {
                line.push(' ');
            }
        }
    })
}



fn write_single_value(
    append: impl Fn(Cell, &mut String),
) -> Vec<String> {
    let mut lines = Vec::new();

    // top border
    lines.push("┍───────┬───────┬───────┐".to_owned());

    for row in House::rows_iter() {
        if !row.is_top() {
            if row.is_block_top() {
                lines.push("├───────┼───────┼───────┤".to_owned());
            }
        }

        let mut line = String::from("│");
        for cell in row.cells() {
            line.push(' ');
            append(cell, &mut line);

            if cell.column().is_block_right() {
                line.push_str(" │");
            }
        }
        lines.push(line);
    }

    // bottom border
    lines.push("└───────┴───────┴───────┘".to_owned());

    lines
}



fn add_single_value_labels(grid: Vec<String>) -> Vec<String> {
    let mut lines = Vec::new();
    let mut iter = grid.into_iter();

    lines.push("    1 2 3   4 5 6   7 8 9    ".to_owned());
    lines.push(format!("  {}  ", iter.next().unwrap()));

    for row in House::rows_iter() {
        lines.push(format!(
            "{} {} {}",
            row.console_label(),
            iter.next().unwrap(),
            row.console_label()
        ));

        if row.is_block_bottom() {
            lines.push(format!("  {}  ", iter.next().unwrap()));
        }
    }

    lines.push("    1 2 3   4 5 6   7 8 9    ".to_owned());
    lines
}




fn write_candidates(board: &Board) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push(
        "┍───────────────────────┬───────────────────────┬───────────────────────┐".to_string(),
    );

    for row in House::rows_iter() {
        let mut cell_lines = [
            String::from("│ "),
            String::from("│ "),
            String::from("│ "),
        ];

        for column in House::columns_iter() {
            let cell = Cell::from_row_column(row, column);
            let value = board.value(cell);
            let candidates = board.candidates(cell);

            if !value {
                for known in Known::iter() {
                    let line = known.usize() / 3;
                    if candidates[known] {
                        cell_lines[line].push(known.label());
                    } else {
                        cell_lines[line].push(MISSING);
                    }
                    cell_lines[line].push(' ');
                }
            } else {
                cell_lines[0].push_str("      ");
                cell_lines[1].push_str(&format!("  {}   ", value));
                if board.is_given(cell) {
                    cell_lines[2].push_str(&format!("  {}   ", MISSING));
                } else {
                    cell_lines[2].push_str("      ");
                }
            }

            if column.is_right() {
                cell_lines.iter_mut().for_each(|l| l.push('│'));
            } else if column.is_block_right() {
                cell_lines.iter_mut().for_each(|l| l.push_str("│ "));
            } else {
                cell_lines.iter_mut().for_each(|l| l.push_str("  "));
            }
        }

        cell_lines.into_iter().for_each(|l| lines.push(l));

        if row.is_block_bottom() {
            if !row.is_bottom() {
                lines.push(
                    "├───────────────────────┼───────────────────────┼───────────────────────┤"
                        .to_owned(),
                );
            }
        } else {
            lines.push(
                "│                       │                       │                       │"
                    .to_owned(),
            );
        }
    }

    lines.push(
        "└───────────────────────┴───────────────────────┴───────────────────────┘".to_owned(),
    );

    lines
}

fn write_candidates_with_highlight(
    board: &Board,
    verdicts: HashMap<Cell, HashMap<Known, Verdict>>,
) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push(
        "┍───────────────────────┬───────────────────────┬───────────────────────┐".to_string(),
    );

    for row in House::rows_iter() {
        let mut cell_lines = [
            String::from("│ "),
            String::from("│ "),
            String::from("│ "),
        ];

        for column in House::columns_iter() {
            let cell = Cell::from_row_column(row, column);
            let value = board.value(cell);
            let candidates = board.candidates(cell);

            if let Some(known) = value.known() {
                let verdict = verdicts
                    .get(&cell)
                    .and_then(|m| m.get(&known))
                    .unwrap_or(&Verdict::None);

                cell_lines[0].push_str("      ");
                cell_lines[1].push_str(&format!(
                    "  {}   ",
                    verdict.color_char(known.label()).as_str()
                ));

                if board.is_given(cell) {
                    cell_lines[2].push_str(&format!(
                        "  {}   ",
                        verdict.color_char(MISSING).as_str()
                    ));
                } else {
                    cell_lines[2].push_str("      ");
                }
            } else {
                for known in Known::iter() {
                    let line = known.usize() / 3;
                    let label = if candidates[known] {
                        known.label()
                    } else {
                        MISSING
                    };

                    let verdict = verdicts
                        .get(&cell)
                        .and_then(|m| m.get(&known))
                        .unwrap_or(&Verdict::None);

                    cell_lines[line].push_str(verdict.color_char(label).as_str());
                    cell_lines[line].push(' ');
                }
            }

            if column.is_right() {
                cell_lines.iter_mut().for_each(|l| l.push('│'));
            } else if column.is_block_right() {
                cell_lines.iter_mut().for_each(|l| l.push_str("│ "));
            } else {
                cell_lines.iter_mut().for_each(|l| l.push_str("  "));
            }
        }

        cell_lines.into_iter().for_each(|l| lines.push(l));

        if row.is_block_bottom() {
            if !row.is_bottom() {
                lines.push(
                    "├───────────────────────┼───────────────────────┼───────────────────────┤"
                        .to_owned(),
                );
            }
        } else {
            lines.push(
                "│                       │                       │                       │"
                    .to_owned(),
            );
        }
    }

    lines.push(
        "└───────────────────────┴───────────────────────┴───────────────────────┘".to_owned(),
    );

    lines
}



fn add_all_candidates_labels(grid: Vec<String>) -> Vec<String> {
    let mut lines = Vec::new();
    let mut iter = grid.into_iter();

    lines.push(
        "      1       2       3       4       5       6       7       8       9      "
            .to_string(),
    );
    lines.push(format!("  {}  ", iter.next().unwrap()));

    for row in House::rows_iter() {
        lines.push(format!("  {}  ", iter.next().unwrap()));
        lines.push(format!(
            "{} {} {}",
            row.console_label(),
            iter.next().unwrap(),
            row.console_label()
        ));
        lines.push(format!("  {}  ", iter.next().unwrap()));
        lines.push(format!("  {}  ", iter.next().unwrap()));
    }

    lines.push(
        "      1       2       3       4       5       6       7       8       9      "
            .to_string(),
    );

    lines
}

fn actually_print_all_and_single_candidates(
    grid: Vec<String>,
    candidate_grids: Vec<Vec<String>>,
) {
    let mut columns = [Vec::new(), Vec::new(), Vec::new()];

    for (i, grid) in candidate_grids.iter().enumerate() {
        columns[i % 3].extend(grid);
    }

    let mut column_iters = [
        columns[0].iter(),
        columns[1].iter(),
        columns[2].iter(),
    ];

    for line in add_all_candidates_labels(grid) {
        println!(
            "{}    {} {} {}",
            line,
            column_iters[0].next().unwrap(),
            column_iters[1].next().unwrap(),
            column_iters[2].next().unwrap()
        );
    }
}
