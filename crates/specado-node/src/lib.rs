#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use specado_core::{execute, PromptSpec, UniformResponse};

#[napi]
pub struct Client {
    provider_path: String,
}

#[napi]
impl Client {
    #[napi(constructor)]
    pub fn new(provider_path: String) -> Result<Self> {
        Ok(Self { provider_path })
    }

    #[napi]
    pub async fn complete(&self, prompt: serde_json::Value) -> Result<serde_json::Value> {
        let prompt_spec: PromptSpec = serde_json::from_value(prompt)
            .map_err(|e| Error::from_reason(format!("Invalid prompt spec: {e}")))?;

        let response: UniformResponse = execute(prompt_spec, &self.provider_path)
            .await
            .map_err(|e| Error::from_reason(format!("Execution failed: {e}")))?;

        serde_json::to_value(&response)
            .map_err(|e| Error::from_reason(format!("Serialization failed: {e}")))
    }
}
