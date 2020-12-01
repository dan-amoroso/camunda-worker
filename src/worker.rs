use crate::engine::Engine;
use crate::engine::Task;
use log::{error, info, warn};

use std::collections::HashMap;
use std::error::Error;
use std::{thread, time};

use camunda_client::models::VariableValueDto;

type VariablesMap = HashMap<String, VariableValueDto>;

pub fn wait(interval_seconds: u64) {
    thread::sleep(time::Duration::from_secs(interval_seconds));
}

pub fn run_topic_handler<F>(topic_str: &str, worker_id: &str, engine: &Engine, handler: F)
where
    F: Fn(Task) -> Result<VariablesMap, Box<dyn Error>>,
{
    loop {
        info!("Fetching task...");

        match &engine.lock_task(topic_str, worker_id, 1) {
            Ok(tasks) => {
                if tasks.is_empty() {
                    wait(10);
                    continue;
                }
                let task = &tasks[0];
                let result = handler(task.to_owned());

                match result {
                    Ok(variables) => {
                        complete_task(&engine, &task, &worker_id, variables);
                    }
                    Err(topic_handler_error) => {
                        handle_topic_handler_failure(&engine, &task, topic_handler_error)
                    }
                }
            }
            Err(lock_task_error) => handle_lock_task_error(lock_task_error),
        }
        wait(10);
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
