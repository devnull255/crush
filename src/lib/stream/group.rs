use crate::lang::ExecutionContext;
use std::collections::HashMap;
use crate::{
    lib::command_util::find_field_from_str,
    errors::{argument_error},
    lang::{
        Argument,
        Row,
        Stream,
        ValueType,
        Value,
    },
    stream::{OutputStream, unlimited_streams},
};
use crate::lang::{ColumnType, RowsReader};
use crate::errors::{CrushResult, error};
use crate::lib::command_util::find_field;
use crate::stream::{Readable};

pub struct Config {
    input_type: Vec<ColumnType>,
    name: Box<str>,
    column: usize,
}

pub fn parse(input_type: Vec<ColumnType>, arguments: Vec<Argument>) -> CrushResult<Config> {
    if arguments.len() != 1 {
        return argument_error("No comparison key specified");
    }
    let arg = &arguments[0];
    let name = arg.name.clone().unwrap_or(Box::from("group"));
    match &arg.value {
        Value::Text(cell_name) =>
            Ok(Config {
                column: find_field_from_str(cell_name, &input_type)?,
                input_type,
                name,
            }),
        Value::Field(cell_name) =>
            Ok(Config {
                column: find_field(cell_name, &input_type)?,
                input_type,
                name,
            }),
        _ => argument_error("Bad comparison key"),
    }
}

pub fn run(
    config: Config,
    mut input: impl Readable,
    output: OutputStream,
) -> CrushResult<()> {
    let mut groups: HashMap<Value, OutputStream> = HashMap::new();

    loop {
        match input.read() {
            Ok(row) => {
                let key = row.cells()[config.column].clone();
                let val = groups.get(&key);
                match val {
                    None => {
                        let (output_stream, input_stream) = unlimited_streams(config.input_type.clone());
                        let out_row = Row::new(vec![key.clone(), Value::Stream(Stream { stream: input_stream })]);
                        output.send(out_row)?;
                        output_stream.send(row);
                        groups.insert(key, output_stream);
                    }
                    Some(output_stream) => {
                        output_stream.send(row);
                    }
                }
            }
            Err(_) => break,
        }
    }
    return Ok(());
}

pub fn perform(context: ExecutionContext) -> CrushResult<()> {
    match context.input.recv()? {
        Value::Stream(s) => {
            let input = s.stream;
            let config = parse(input.types().clone(), context.arguments)?;
            let output_type= vec![
                input.types()[config.column].clone(),
                ColumnType {
                    name: Some(config.name.clone()),
                    cell_type: ValueType::Stream(input.types().clone())
                }
            ];
            let output = context.output.initialize(output_type)?;
            run(config, input, output)
        }
        Value::Rows(r) => {
            let input = RowsReader::new(r);
            let config = parse(input.types().clone(), context.arguments)?;
            let output_type= vec![
                input.types()[config.column].clone(),
                ColumnType {
                    name: Some(config.name.clone()),
                    cell_type: ValueType::Stream(input.types().clone())
                }
            ];
            let output = context.output.initialize(output_type)?;
            run(config, input, output)
        }
        _ => error("Expected a stream"),
    }
}
