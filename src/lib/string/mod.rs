use crate::lang::scope::Scope;
use crate::lang::errors::{CrushResult, argument_error};
use crate::lang::{command::ExecutionContext, value::ValueType, list::List};
use crate::lib::parse_util::{single_argument, two_arguments, single_argument_field, single_argument_text};
use crate::lang::{value::Value, command::SimpleCommand, argument::Argument};
use nix::sys::ptrace::cont;
use crate::lang::command::CrushCommand;

mod format;

fn upper(mut context: ExecutionContext) -> CrushResult<()> {
    context.output.send(Value::String(
        single_argument_text(context.arguments)?
            .to_uppercase()
            .into_boxed_str()))
}

fn lower(mut context: ExecutionContext) -> CrushResult<()> {
    context.output.send(Value::String(
        single_argument_text(context.arguments)?
            .to_lowercase()
            .into_boxed_str()))
}

fn split(mut context: ExecutionContext) -> CrushResult<()> {
    two_arguments(&context.arguments)?;

    let mut separator = None;
    let mut text = None;

    for arg in context.arguments.drain(..) {
        match (arg.name.as_deref(), arg.value) {
            (Some("separator"), Value::String(t)) => {
                if t.len() != 1 {
                    return argument_error("Separator must be single character");
                }
                separator = Some(t.chars().next().unwrap());
            }
            (Some("text"), Value::String(t)) => text = Some(t),
            _ => return argument_error("Unknown argument"),
        }
    }

    match (separator, text) {
        (Some(s), Some(t)) => {
            context.output.send(Value::List(List::new(ValueType::String,
                                                      t.split(s)
                                                          .map(|s| Value::String(Box::from(s)))
                                                          .collect())))
        }
        _ => argument_error("Missing arguments")
    }
}

fn trim(mut context: ExecutionContext) -> CrushResult<()> {
    context.output.send(Value::String(
        Box::from(single_argument_text(context.arguments)?
            .trim())))
}

pub fn declare(root: &Scope) -> CrushResult<()> {
    let env = root.create_namespace("string")?;
    env.declare("upper", Value::Command(SimpleCommand::new(upper, false).boxed()))?;
    env.declare("lower", Value::Command(SimpleCommand::new(lower, false).boxed()))?;
    env.declare("format", Value::Command(SimpleCommand::new(format::format, false).boxed()))?;
    env.declare("split", Value::Command(SimpleCommand::new(split, false).boxed()))?;
    env.declare("trim", Value::Command(SimpleCommand::new(trim, false).boxed()))?;
    env.readonly();
    Ok(())
}