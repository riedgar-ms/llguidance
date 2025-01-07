use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct TestResult {
    pub valid: bool,
    pub success: bool,
}

#[derive(Serialize, Debug)]
pub struct CaseResult {
    pub category: String,
    pub index: usize,
    pub tests: Vec<TestResult>,
}
