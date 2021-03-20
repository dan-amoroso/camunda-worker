## Camunda-worker

helper library for ergonimic camunda worker development in Rust

### Usage Example

```Rust
extern crate camunda_worker;

use camunda_worker::{config, engine, worker};
use config::Config;
use engine::Task;
use worker::{Handler, HandlerResult};
use std::collections::HashMap;

fn main() {
    // In a real worker these values would probably be parsed from a config file
    let config = Config::new(
        "http://localhost:8080/engine-rest".to_owned(),
        "demo".to_owned(),
        "demo".to_owned(),
        "worker-id-007".to_owned(),
        );

    // run_topic_handler is used to run a single topic worker
    // it expects the config to connect to the engine
    // and a Handler, which is a topic string and the correspondent
    // closure to handle the external task, packaged together in a struct
    worker::run_topic_handler(
        &config, 
        Handler{
            topic:"SomeTopic".to_owned(), 
            // the handler function gets passed a task object
            // containing process variables and other data necessary to execute
            // the external task
            handler_function: |_task: Task| -> HandlerResult {
                // in this example we do nothing other than complete the task
                // the hashmap is normally used to apply changes to the process
                // variables, this is how a camunda worker communicates back
                // with the process
                Ok(HashMap::new())
        }} 
        );
}
```
