pub mod engine {
    extern crate camunda_client;

    use super::config::Config;
    use super::utils::new_configuration;

    use std::collections::HashMap;

    use camunda_client::apis::client::APIClient;
    use camunda_client::apis::Error;
    use camunda_client::models::CompleteExternalTaskDto;
    use camunda_client::models::FetchExternalTaskTopicDto;
    use camunda_client::models::FetchExternalTasksDto;
    use camunda_client::models::LockedExternalTaskDto;
    use camunda_client::models::VariableValueDto;

    pub struct Engine {
        config: Config,
        pub api_client: APIClient,
    }

    fn fetch_and_lock_dto(
        topic: &str,
        worker_id: &str,
        lock_duration: Option<i64>,
        max_tasks: i32,
    ) -> FetchExternalTasksDto {
        return FetchExternalTasksDto {
            worker_id: worker_id.to_owned(),
            max_tasks: Some(max_tasks),
            use_priority: None,
            async_response_timeout: Some(60i64),
            topics: Some(vec![FetchExternalTaskTopicDto::new(
                topic.to_owned(),
                lock_duration,
            )]),
        };
    }

    impl Engine {
        pub fn new(config: &Config) -> Engine {
            let configuration = new_configuration(
                &config.base_path,
                &(
                    config.camunda_username.to_owned(),
                    Some(config.camunda_password.to_owned()),
                ),
            );
            return Engine {
                config: config.clone(),
                api_client: APIClient::new(configuration),
            };
        }

        pub fn lock_task(
            &self,
            topic: &str,
            worker_id: &str,
            max_tasks: i32,
        ) -> Result<Vec<Task>, Error> {
            return self
                .api_client
                .external_task_api()
                .fetch_and_lock(Some(fetch_and_lock_dto(
                    topic,
                    worker_id,
                    self.config.lock_duration,
                    max_tasks,
                )));
        }

        pub fn complete_task(
            &self,
            task: &Task,
            worker_id: &str,
            variables: HashMap<String, VariableValueDto>,
        ) -> Result<(), Error> {
            let complete_external_task_dto = CompleteExternalTaskDto {
                worker_id: Some(worker_id.to_owned()),
                variables: Some(variables),
                local_variables: None,
            };
            self.api_client
                .external_task_api()
                .complete_external_task_resource(
                    &task.id.as_ref().unwrap()[..],
                    Some(complete_external_task_dto),
                )
        }

        pub fn release_task(&self, task: &Task) -> Result<(), Error> {
            return self
                .api_client
                .external_task_api()
                .unlock(&task.id.as_ref().unwrap());
        }
    }

    pub type Task = LockedExternalTaskDto;
}

pub mod config {
    #[derive(Clone, Debug)]
    pub struct Config {
        pub wait_interval: u64,
        pub base_path: String,
        pub camunda_username: String,
        pub camunda_password: String,
        pub topic: String,
        pub lock_duration: Option<i64>,
        pub worker_id: String,
    }

    impl Config {
        pub fn new(
            camunda_base_url: String,
            camunda_username: String,
            camunda_password: String,
            topic: String,
            worker_id: String,
        ) -> Self {
            return Self {
                wait_interval: 60,
                base_path: camunda_base_url,
                camunda_username: camunda_username,
                camunda_password: camunda_password,
                topic: topic,
                lock_duration: Some(60i64),
                worker_id: worker_id,
            };
        }
    }
}

pub mod worker {

    use crate::engine::Engine;
    use crate::engine::Task;

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
            println!("fetching task...");

            match &engine.lock_task(topic_str, worker_id, 1) {
                Ok(tasks) => {
                    if tasks.len() == 0 {
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
            println!("task successfully completed");
        } else if let Err(err) = complete_call {
            println!(
                "failed to complete a task because of failed engine call {:#?}",
                err
            );
        };
    }

    fn handle_topic_handler_failure(engine: &Engine, task: &Task, message: Box<dyn Error>) {
        if let Ok(_result) = engine.release_task(task) {
            println!("releasing task {}", message);
        } else {
            println!("releasing task failed : {}", message);
        };
    }

    fn handle_lock_task_error(error: &camunda_client::apis::Error) {
        println!("Error: {:#?}", error);
    }
}

pub mod utils {

    use camunda_client::apis::configuration::{BasicAuth, Configuration};
    use camunda_client::models::VariableValueDto;
    use std::collections::HashMap;
    use std::env;

    pub fn variable_value(val: serde_json::Value) -> VariableValueDto {
        return VariableValueDto {
            value: Some(val),
            _type: None,
            value_info: None,
        };
    }

    pub fn variables_from_vec(
        vec: &[(String, VariableValueDto)],
    ) -> HashMap<String, VariableValueDto> {
        vec.iter().cloned().collect()
    }

    pub fn new_configuration(base_path: &str, basic_auth: &BasicAuth) -> Configuration {
        let def = Configuration::default();
        return Configuration {
            base_path: base_path.to_owned(),
            basic_auth: Some(basic_auth.to_owned()),
            client: def.client,
            user_agent: def.user_agent,
            bearer_access_token: None,
            oauth_access_token: None,
            api_key: None,
        };
    }

    pub fn get_env_var(key: &str) -> String {
        return env::var(key).expect(&format!("{} env variable is not present", key));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
