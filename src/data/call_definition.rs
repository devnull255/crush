use crate::commands::{CompileContext, JobJoinHandle};
use crate::data::{ArgumentDefinition, ArgumentVecCompiler, Value};
use crate::env::Env;
use crate::errors::{error, JobResult};
use crate::printer::Printer;
use crate::stream::{ValueReceiver, ValueSender};
use crate::thread_util::{handle, build};

#[derive(Clone)]
#[derive(Debug)]
pub struct CallDefinition {
    name: Vec<Box<str>>,
    arguments: Vec<ArgumentDefinition>,
}

pub fn format_name(name: &Vec<Box<str>>) -> String {
    name.join(".")
}


impl CallDefinition {
    pub fn new(name: Vec<Box<str>>, arguments: Vec<ArgumentDefinition>) -> CallDefinition {
        CallDefinition { name, arguments }
    }

    pub fn spawn_and_execute(
        &self,
        env: &Env,
        printer: &Printer,
        input: ValueReceiver,
        output: ValueSender,
    ) -> JobResult<JobJoinHandle> {
        let local_printer = printer.clone();
        let local_arguments = self.arguments.clone();
        let local_env = env.clone();
        let cmd = env.get(&self.name);
        match cmd {
            Some(Value::Command(command)) => {
                let c = command.call;
                Ok(handle(build(format_name(&self.name)).spawn(
                    move || {
                        let mut deps: Vec<JobJoinHandle> = Vec::new();
                        let arguments = local_arguments.compile(&mut deps, &local_env, &local_printer)?;
                        let res = c(CompileContext {
                            input,
                            output,
                            arguments,
                            env: local_env,
                            printer: local_printer.clone(),
                        });
                        if !deps.is_empty() {
                            local_printer.join(JobJoinHandle::Many(deps));
                        }
                        res
                    })))
            }

            Some(Value::Closure(closure_definition)) => {
                Ok(handle(build(format_name(&self.name)).spawn(
                    move || {
                        let mut deps: Vec<JobJoinHandle> = Vec::new();
                        let arguments = local_arguments.compile(&mut deps, &local_env, &local_printer)?;

                        closure_definition.spawn_and_execute(
                            CompileContext {
                                input,
                                output,
                                arguments,
                                env: local_env.clone(),
                                printer: local_printer.clone(),
                            })?;
                        JobJoinHandle::Many(deps).join(&local_printer);
                        Ok(())
                    })))
            }
            _ => {
                Err(error(format!("Unknown command name {}", format_name(&self.name)).as_str()))
            }
        }
    }
}

impl ToString for CallDefinition {
    fn to_string(&self) -> String {
        self.name.last().unwrap().to_string()
    }
}
