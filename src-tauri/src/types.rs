use async_openai::error::OpenAIError;
use futures::Stream;
use serde_json::Value;
use std::pin::Pin;

pub type LlmStreaming = Pin<Box<dyn Stream<Item = Result<Value, OpenAIError>> + Send>>;
