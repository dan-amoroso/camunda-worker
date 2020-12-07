#[derive(Clone, Debug)]
pub struct Config {
    pub wait_interval: u64,
    pub base_path: String,
    pub camunda_username: String,
    pub camunda_password: String,
    pub topics: Vec<TopicConfig>,
    pub lock_duration: Option<i64>,
    pub worker_id: String,
}

impl Config {
    pub fn new(
        camunda_base_url: String,
        camunda_username: String,
        camunda_password: String,
        topics: Vec<TopicConfig>,
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

#[derive(Clone, Debug)]
pub struct TopicConfig {
    pub topic_id: String,
    pub wait_interval: Option<u64>,
}

impl TopicConfig {
    pub fn new(topic_id: String) -> Self {
        Self {
            topic_id,
            wait_interval: None,
        }
    }
}
