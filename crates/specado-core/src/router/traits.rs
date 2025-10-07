use crate::error::Result;
use crate::types::{PromptSpec, UniformResponse};
use async_trait::async_trait;

#[async_trait]
pub trait Router: Send + Sync {
    async fn route(&self, prompt: PromptSpec) -> Result<UniformResponse>;
}
