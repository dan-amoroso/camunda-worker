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
    FetchExternalTasksDto {
        worker_id: worker_id.to_owned(),
        max_tasks: Some(max_tasks),
        use_priority: None,
        async_response_timeout: Some(60i64),
        topics: Some(vec![FetchExternalTaskTopicDto::new(
            topic.to_owned(),
            lock_duration,
        )]),
    }
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
        Engine {
            config: config.clone(),
            api_client: APIClient::new(configuration),
        }
    }

    pub fn lock_task(
        &self,
        topic: &str,
        worker_id: &str,
        max_tasks: i32,
    ) -> Result<Vec<Task>, Error> {
        self.api_client
            .external_task_api()
            .fetch_and_lock(Some(fetch_and_lock_dto(
                topic,
                worker_id,
                self.config.lock_duration,
                max_tasks,
            )))
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
        self.api_client
            .external_task_api()
            .unlock(&task.id.as_ref().unwrap())
    }
}

pub type Task = LockedExternalTaskDto;
