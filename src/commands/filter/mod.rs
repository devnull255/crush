mod parser;

use crate::{
    data::{
        Cell,
        CellType,
        Row,
        Argument
    },
    stream::{OutputStream, InputStream},
    commands::{Call, Exec},
    errors::{JobError, argument_error},
    commands::filter::parser::{Condition, Value, parse}
};
use std::iter::Iterator;
use crate::printer::Printer;
use crate::errors::error;
use std::cmp::Ordering;

fn do_match(needle: &Cell, haystack: &Cell) -> Result<bool, JobError> {
    match (needle, haystack) {
        (Cell::Text(s), Cell::Glob(pattern)) => Ok(pattern.matches( s)),
        (Cell::File(f), Cell::Glob(pattern)) => f.to_str().map(|s| Ok(pattern.matches( s))).unwrap_or(Err(error("Invalid filename"))),
        (Cell::Text(s), Cell::Regex(_, pattern)) => Ok(pattern.is_match(s)),
        (Cell::File(f), Cell::Regex(_, pattern)) => match f.to_str().map(|s| pattern.is_match(s)) {
            Some(v) => Ok(v),
            None => Err(error("Invalid filename")),
        },
        _ => Err(error("Invalid match"))
    }
}

fn to_cell<'a>(value: &'a Value, row: &'a Row) -> &'a Cell {
    return match value {
        Value::Cell(c) => &c,
        Value::Field(idx) => &row.cells[*idx],
    };
}

fn evaluate(condition: &Condition, row: &Row) -> Result<bool, JobError> {
    return match condition {
        Condition::Equal(l, r) =>
            Ok(to_cell(&l, row) == to_cell(&r, row)),
        Condition::GreaterThan(l, r) =>
            match to_cell(&l, row).partial_cmp(to_cell(&r, row)) {
                Some(ord) => Ok(ord == Ordering::Greater),
                None => Err(error("Cell types can't be compared")),
            },
        Condition::GreaterThanOrEqual(l, r) =>
            match to_cell(&l, row).partial_cmp(to_cell(&r, row)) {
                Some(ord) => Ok(ord != Ordering::Less),
                None => Err(error("Cell types can't be compared")),
            },
        Condition::LessThan(l, r) =>
            match to_cell(&l, row).partial_cmp(to_cell(&r, row)) {
                Some(ord) => Ok(ord == Ordering::Less),
                None => Err(error("Cell types can't be compared")),
            },
        Condition::LessThanOrEqual(l, r) =>
            match to_cell(&l, row).partial_cmp(to_cell(&r, row)) {
                Some(ord) => Ok(ord != Ordering::Greater),
                None => Err(error("Cell types can't be compared")),
            },
        Condition::NotEqual(l, r) =>
            Ok(to_cell(&l, row) != to_cell(&r, row)),
        Condition::Match(l, r) =>
            do_match(to_cell(&l, row), to_cell(&r, row)),
        Condition::NotMatch(l, r) =>
            do_match(to_cell(&l, row), to_cell(&r, row)).map(|r| !r),
    };
}

fn run(
    input_type: Vec<CellType>,
    arguments: Vec<Argument>,
    input: InputStream,
    output: OutputStream,
    printer: Printer,
) -> Result<(), JobError> {
    let condition = parse(&input_type, &arguments)?;
    loop {
        match input.recv() {
            Ok(row) => {
                match evaluate(&condition, &row) {
                    Ok(val) => if output.send(row).is_err() { break },
                    Err(e) => printer.job_error(e),
                }
            }
            Err(_) => break,
        }
    }
    return Ok(());
}

pub fn filter(input_type: Vec<CellType>, arguments: Vec<Argument>) -> Result<Call, JobError> {
    parse(&input_type, &arguments)?;
    return Ok(Call {
        name: String::from("filter"),
        output_type: input_type.clone(),
        input_type,
        arguments: arguments,
        exec: Exec::Run(run),
    });
}
