use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct Case {
    pub description: String,
    pub schema: Value,
    pub tests: Vec<Test>,
}

#[derive(Deserialize, Debug)]
pub struct Test {
    pub description: String,
    pub data: Value,
    pub valid: bool,
}
