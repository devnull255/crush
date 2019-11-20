use crate::commands::CompileContext;
use crate::errors::{JobResult, error};
use crate::{
    data::{
        Row,
        ValueType,
        Value
    },
    stream::{InputStream},
};
use crate::data::{ColumnType, Argument};
use crate::commands::command_util::find_field_from_str;

pub fn parse(input_type: &Vec<ColumnType>, arguments: &Vec<Argument>) -> JobResult<usize> {
    match arguments.len() {
        0 => if input_type.len() == 1 && input_type[0].cell_type == ValueType::Integer{
            Ok(0)
        } else {
            Err(error("Unexpected input format, expected a single column of integers"))
        },
        1 => {
            if let Value::Field(f) = &arguments[0].value {
                match f.len() {
                    1 => {
                        Ok(find_field_from_str(f[0].as_ref(), input_type)?)
                    }
                    _ => {
                        Err(error("Path contains too many elements"))
                    }
                }
            } else {
                Err(error("Unexpected cell type, expected field"))
            }
        },
        _ => Err(error("Expected exactly one argument, a field defintition"))
    }
}

fn sum_rows(s: &InputStream, column: usize) -> JobResult<Value> {
    let mut res: i128 = 0;
    loop {
        match s.recv() {
            Ok(row) => match row.cells[column] {
                Value::Integer(i) => res += i,
                _ => return Err(error("Invalid cell value, expected an integer"))
            },
            Err(_) => break,
        }
    }
    Ok(Value::Integer(res))
}

pub fn compile_and_run(context: CompileContext) -> JobResult<()> {
    let output = context.output.initialize(vec![ColumnType::named("sum", ValueType::Integer)])?;
    let input = context.input.initialize_stream()?;
    let column = parse(input.get_type(), &context.arguments)?;
    output.send(Row { cells: vec![sum_rows(&input, column)?]})
}
