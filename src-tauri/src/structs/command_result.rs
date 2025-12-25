use crate::enums::CommandResultCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CommandResult {
    message: String,
    code: CommandResultCode,
    success: bool,
    data: Option<serde_json::Value>,
}

impl Default for CommandResult {
    fn default() -> Self {
        CommandResult {
            message: "ok".to_string(),
            code: CommandResultCode::SUCCESS,
            success: true,
            data: None,
        }
    }
}

impl CommandResult {
    pub fn new(code: CommandResultCode, message: String, data: Option<serde_json::Value>) -> Self {
        let success = code == CommandResultCode::SUCCESS;
        CommandResult {
            message,
            code,
            success,
            data,
        }
    }

    pub fn success(message: String, data: Option<serde_json::Value>) -> Self {
        CommandResult {
            message,
            code: CommandResultCode::SUCCESS,
            success: true,
            data,
        }
    }

    pub fn error(code: CommandResultCode, message: String) -> Self {
        CommandResult {
            message,
            code,
            success: false,
            data: None,
        }
    }
}
