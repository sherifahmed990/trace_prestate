use serde::Deserialize;
use std::{error::Error, fmt};

#[derive(Debug, Deserialize)]
pub struct JsonRpcSuccessfulResponse<Result> {
    pub id: Option<serde_json::value::Number>,
    pub result: Result,
}

#[derive(Debug, Deserialize)]
pub struct ErrorDetails {
    pub code: i16,
    pub message: Option<String>
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcError {
    pub id: Option<serde_json::value::Number>,
    pub error: ErrorDetails
}
impl fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, 
            "Error code: {}, message: {}",
            self.error.code,
            self.error.message.clone().unwrap_or_default()
        )
    }
}

impl Error for JsonRpcError {}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcResponse <T>{
    Result(JsonRpcSuccessfulResponse<T>),
    Error(JsonRpcError)
}
