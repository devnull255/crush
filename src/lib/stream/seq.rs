use crate::lang::execution_context::{ExecutionContext, ArgumentVector};
use crate::lang::errors::{CrushResult};
use crate::{
    lang::{
        table::Row,
        value::ValueType,
        value::Value
    }
};
use crate::lang::table::ColumnType;

pub fn perform(mut context: ExecutionContext) -> CrushResult<()> {
    let c  = context.arguments.optional_integer(0)?.unwrap_or(i128::max_value());
    let output = context.output.initialize(vec![
        ColumnType::new("value", ValueType::Integer)])?;

    for i in 0..c {
        output.send(Row::new(vec![Value::Integer(i)]))?;
    }
    Ok(())
}
