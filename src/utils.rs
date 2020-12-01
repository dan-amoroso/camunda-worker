use camunda_client::apis::configuration::{BasicAuth, Configuration};
use camunda_client::models::VariableValueDto;
use std::collections::HashMap;
use std::env;

pub fn variable_value(val: serde_json::Value) -> VariableValueDto {
    VariableValueDto {
        value: Some(val),
        _type: None,
        value_info: None,
    }
}

pub fn variables_from_vec(vec: &[(String, VariableValueDto)]) -> HashMap<String, VariableValueDto> {
    vec.iter().cloned().collect()
}

pub fn new_configuration(base_path: &str, basic_auth: &BasicAuth) -> Configuration {
    let def = Configuration::default();
    Configuration {
        base_path: base_path.to_owned(),
        basic_auth: Some(basic_auth.to_owned()),
        client: def.client,
        user_agent: def.user_agent,
        bearer_access_token: None,
        oauth_access_token: None,
        api_key: None,
    }
}

pub fn get_env_var(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("{} env variable is not present", key))
}
