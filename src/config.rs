use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Config {
    pub wait_interval: u64,
    pub base_path: String,
    pub camunda_username: String,
    pub camunda_password: String,
    pub topics: Vec<HashMap<String, String>>,
    pub lock_duration: Option<i64>,
    pub worker_id: String,
}

impl Config {
    pub fn new(
        camunda_base_url: String,
        camunda_username: String,
        camunda_password: String,
        topics: Vec<HashMap<String, String>>,
        worker_id: String,
    ) -> Self {
        Self {
            wait_interval: 60,
            base_path: camunda_base_url,
            camunda_username,
            camunda_password,
            topics,
            lock_duration: Some(60i64),
            worker_id,
        }
    }
}
