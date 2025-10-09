use crate::error::{Error, Result};
use crate::router::traits::Router;
use crate::types::{PromptSpec, UniformResponse};
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type BoxedExecutor = Arc<dyn Fn(PromptSpec, String) -> ExecutorFuture + Send + Sync>;

pub type ExecutorFuture = Pin<Box<dyn Future<Output = Result<UniformResponse>> + Send>>;

pub struct PrimaryFallbackRouter {
    primary: String,
    fallbacks: Vec<String>,
    executor: BoxedExecutor,
}

impl PrimaryFallbackRouter {
    pub fn new(
        primary: impl Into<String>,
        fallbacks: Vec<String>,
        executor: BoxedExecutor,
    ) -> Self {
        Self {
            primary: primary.into(),
            fallbacks,
            executor,
        }
    }
}

#[async_trait]
impl Router for PrimaryFallbackRouter {
    async fn route(&self, prompt: PromptSpec) -> Result<UniformResponse> {
        let executor = &self.executor;
        let mut last_error: Option<Error> = None;

        for provider in std::iter::once(&self.primary).chain(self.fallbacks.iter()) {
            let provider_path = provider.clone();
            match executor(prompt.clone(), provider_path).await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    last_error = Some(err);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            Error::Config("No providers configured for PrimaryFallbackRouter".into())
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::types::{Extensions, FinishReason, LossinessReport, StrictMode};

    fn sample_response(model: &str) -> UniformResponse {
        UniformResponse {
            content: "hi".into(),
            tool_calls: Vec::new(),
            finish_reason: FinishReason::Stop,
            model: model.into(),
            provider_used: model.into(),
            usage: None,
            extensions: Extensions {
                lossiness: LossinessReport::new(StrictMode::Warn),
                provider_capabilities: None,
            },
        }
    }

    fn make_prompt() -> PromptSpec {
        PromptSpec {
            version: "1".into(),
            messages: Vec::new(),
            sampling: Default::default(),
            response: Default::default(),
            tools: Vec::new(),
            tool_choice: None,
            strict_mode: StrictMode::Warn,
            metadata: Default::default(),
        }
    }

    #[tokio::test]
    async fn uses_primary_when_successful() {
        let executor: BoxedExecutor = Arc::new(|prompt, provider| {
            Box::pin(async move {
                let _ = prompt;
                Ok(sample_response(&provider))
            })
        });

        let router = PrimaryFallbackRouter::new("primary", vec!["fallback".into()], executor);
        let response = router.route(make_prompt()).await.unwrap();
        assert_eq!(response.provider_used, "primary");
    }

    #[tokio::test]
    async fn falls_back_when_primary_fails() {
        let failures = Arc::new(std::sync::Mutex::new(0));
        let executor: BoxedExecutor = {
            let failures = failures.clone();
            Arc::new(move |prompt, provider| {
                let failures = failures.clone();
                Box::pin(async move {
                    let _ = prompt;
                    if provider == "primary" {
                        *failures.lock().unwrap() += 1;
                        Err(Error::Provider {
                            provider: provider.clone(),
                            kind: crate::error::ProviderErrorKind::ServerError,
                        })
                    } else {
                        Ok(sample_response(&provider))
                    }
                })
            })
        };

        let router = PrimaryFallbackRouter::new("primary", vec!["secondary".into()], executor);
        let response = router.route(make_prompt()).await.unwrap();
        assert_eq!(response.provider_used, "secondary");
        assert_eq!(*failures.lock().unwrap(), 1);
    }
}
