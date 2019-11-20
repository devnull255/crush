use crate::commands::CompileContext;
use crate::errors::JobResult;
use crate::data::ValueType;
use crate::data::Row;
use crate::data::Value;
use crate::data::ColumnType;
use crate::env::get_cwd;

pub fn compile_and_run(context: CompileContext) -> JobResult<()> {
    context.output.send(Value::File(get_cwd()?))
}
