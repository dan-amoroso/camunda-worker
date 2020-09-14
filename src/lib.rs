pub mod engine {
    extern crate rust_camunda_client;

    use super::config::Config;
    use super::utils::new_configuration;

    use std::collections::HashMap;

    use rust_camunda_client::apis::client::APIClient;
    use rust_camunda_client::apis::Error;
    use rust_camunda_client::models::CompleteExternalTaskDto;
    use rust_camunda_client::models::FetchExternalTaskTopicDto;
    use rust_camunda_client::models::FetchExternalTasksDto;
    use rust_camunda_client::models::LockedExternalTaskDto;
    use rust_camunda_client::models::VariableValueDto;

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
    #[derive(Clone)]
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
        pub fn new() -> Self {
            return Self {
                wait_interval: 60,
                base_path: "".to_string(),
                camunda_username: "demo".to_string(),
                camunda_password: "demo".to_string(),
                topic: "".to_string(),
                lock_duration: Some(60i64),
                worker_id: "rust-worker-1".to_string(),
            };
        }
    }

    pub fn get_config() -> Config {
        return Config::new();
    }
}

pub mod worker {

    use crate::engine::Engine;
    use crate::engine::Task;

    use std::{thread, time};

    use rust_camunda_client::models::VariableValueDto;
    use std::collections::HashMap;

    use std::error::Error;

    pub fn wait(interval_seconds: u64) {
        thread::sleep(time::Duration::from_secs(interval_seconds));
    }

    pub fn run_topic_handler<F>(
        topic_string: &str,
        worker_id: &str,
        camunda_engine: &Engine,
        handler: F,
    ) where
        F: Fn(Task) -> Result<HashMap<String, VariableValueDto>, Box<dyn Error>>,
    {
        loop {
            let maybe_task = &camunda_engine.lock_task(topic_string, worker_id, 1);

            let task = if let Ok(tasks) = maybe_task {
                if tasks.len() == 0 {
                    continue;
                }
                println!("no tasks... skipping");
                &tasks[0]
            } else {
                wait(10);
                continue;
            };
            let result = handler(task.to_owned());

            match result {
                Ok(variables) => {
                    let complete_call = camunda_engine.complete_task(&task, &worker_id, variables);
                    if let Ok(_result) = complete_call {
                        println!("task successfully completed");
                    } else if let Err(err) = complete_call {
                        println!(
                            "failed to complete a task because of failed engine call {:#?}",
                            err
                        );
                    };
                }
                Err(_message) => {
                    if let Ok(_result) = camunda_engine.release_task(&task) {
                        println!("releasing task {}", _message);
                    } else {
                        println!("releasing task failed : {}", _message);
                    };
                }
            }
            wait(10);
        }
    }
}

pub mod utils {

    use rust_camunda_client::apis::configuration::{BasicAuth, Configuration};
    use rust_camunda_client::models::VariableValueDto;
    use std::collections::HashMap;

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
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
