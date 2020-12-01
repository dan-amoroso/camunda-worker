use crate::config::Config;
use crate::engine::Engine;
use crate::engine::Task;
use log::{error, info, warn};

use std::collections::HashMap;
use std::error::Error;
use std::{thread, time};

use camunda_client::models::VariableValueDto;

pub type VariablesMap = HashMap<String, VariableValueDto>;
pub type HandlerResult = Result<VariablesMap, Box<dyn Error>>;

pub fn wait(interval_seconds: u64) {
    thread::sleep(time::Duration::from_secs(interval_seconds));
}

pub struct Handler<A> {
    pub topic: String,
    pub handler_function: A,
}

pub async fn run_topic_handlers<F>(config: &Config, handlers: Vec<Handler<F>>)
where
    F: Fn(Task) -> HandlerResult + Send + 'static,
{
    let mut task_handles = Vec::new();

    for handler in handlers {
        let config = config.clone();
        let task_handle = tokio::spawn(async move { run_topic_handler(&config, handler) });
        task_handles.push(task_handle);
    }

    for task_handle in task_handles {
        task_handle.await.unwrap();
    }
}

pub fn run_topic_handler<F>(config: &Config, handler: Handler<F>)
where
    F: Fn(Task) -> HandlerResult,
{
    let engine = Engine::new(config);

    loop {
        info!("Fetching task...");

        match &engine.lock_task(&handler.topic, &config.worker_id, 1) {
            Ok(tasks) => {
                if tasks.is_empty() {
                    wait(config.wait_interval);
                    continue;
                }
                let task = &tasks[0];
                let result = (handler.handler_function)(task.to_owned());

                match result {
                    Ok(variables) => {
                        complete_task(&engine, &task, &config.worker_id, variables);
                    }
                    Err(topic_handler_error) => {
                        handle_topic_handler_failure(&engine, &task, topic_handler_error)
                    }
                }
            }
            Err(lock_task_error) => handle_lock_task_error(lock_task_error),
        }
        wait(config.wait_interval);
    }
}

fn complete_task(engine: &Engine, task: &Task, worker_id: &str, vars: VariablesMap) {
    let complete_call = engine.complete_task(&task, &worker_id, vars);
    if let Ok(_result) = complete_call {
        info!("Task successfully completed.");
    } else if let Err(err) = complete_call {
        error!(
            "Failed to complete a task because of failed engine call: {:#?}",
            err
        );
    };
}

fn handle_topic_handler_failure(engine: &Engine, task: &Task, message: Box<dyn Error>) {
    if let Ok(_result) = engine.release_task(task) {
        warn!("Releasing task: {:#?}", message);
    } else {
        error!("Failed to release task: {:#?}", message);
    };
}

fn handle_lock_task_error(error: &camunda_client::apis::Error) {
    error!("Failed to lock task: {:#?}", error);
}
