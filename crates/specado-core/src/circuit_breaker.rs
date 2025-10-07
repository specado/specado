use crate::error::{Error, Result};
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_threshold: usize,
    timeout: Duration,
    half_open_max_requests: usize,
}

#[derive(Debug)]
enum CircuitState {
    Closed {
        failure_count: usize,
    },
    Open {
        opened_at: Instant,
    },
    HalfOpen {
        trial_count: usize,
        success_count: usize,
    },
}

impl CircuitBreaker {
    pub fn new(failure_threshold: usize, timeout: Duration, half_open_max_requests: usize) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed { failure_count: 0 })),
            failure_threshold: failure_threshold.max(1),
            timeout,
            half_open_max_requests: half_open_max_requests.max(1),
        }
    }

    pub async fn call<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>> + Send,
        T: Send,
    {
        self.before_call().await?;
        let outcome = operation().await;
        match outcome {
            Ok(value) => {
                self.on_success().await;
                Ok(value)
            }
            Err(err) => {
                self.on_failure().await;
                Err(err)
            }
        }
    }

    async fn before_call(&self) -> Result<()> {
        let mut state = self.state.lock().await;
        loop {
            match &mut *state {
                CircuitState::Closed { .. } => {
                    break;
                }
                CircuitState::Open { opened_at } => {
                    if opened_at.elapsed() >= self.timeout {
                        *state = CircuitState::HalfOpen {
                            trial_count: 0,
                            success_count: 0,
                        };
                        continue;
                    } else {
                        return Err(Error::CircuitBreakerOpen);
                    }
                }
                CircuitState::HalfOpen { trial_count, .. } => {
                    if *trial_count >= self.half_open_max_requests {
                        return Err(Error::CircuitBreakerHalfOpen);
                    }
                    *trial_count += 1;
                    break;
                }
            }
        }
        Ok(())
    }

    async fn on_success(&self) {
        let mut state = self.state.lock().await;
        match &mut *state {
            CircuitState::Closed { failure_count } => {
                *failure_count = 0;
            }
            CircuitState::HalfOpen { success_count, .. } => {
                *success_count += 1;
                if *success_count >= self.half_open_max_requests {
                    *state = CircuitState::Closed { failure_count: 0 };
                }
            }
            CircuitState::Open { .. } => {
                *state = CircuitState::Closed { failure_count: 0 };
            }
        }
    }

    async fn on_failure(&self) {
        let mut state = self.state.lock().await;
        match &mut *state {
            CircuitState::Closed { failure_count } => {
                *failure_count += 1;
                if *failure_count >= self.failure_threshold {
                    *state = CircuitState::Open {
                        opened_at: Instant::now(),
                    };
                }
            }
            CircuitState::HalfOpen { .. } => {
                *state = CircuitState::Open {
                    opened_at: Instant::now(),
                };
            }
            CircuitState::Open { .. } => {}
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(5, Duration::from_secs(30), 3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    #[tokio::test]
    async fn opens_after_threshold_failures() {
        let breaker = CircuitBreaker::new(2, Duration::from_secs(30), 1);

        assert!(breaker
            .call(|| async { Err::<(), _>(Error::Transform("fail".into())) })
            .await
            .is_err());
        assert!(breaker
            .call(|| async { Err::<(), _>(Error::Transform("fail".into())) })
            .await
            .is_err());

        let err = breaker
            .call(|| async { Ok::<(), Error>(()) })
            .await
            .unwrap_err();
        assert!(matches!(err, Error::CircuitBreakerOpen));
    }

    #[tokio::test]
    async fn half_open_allows_recovery() {
        let breaker = CircuitBreaker::new(1, Duration::from_millis(0), 1);

        let _ = breaker
            .call(|| async { Err::<(), _>(Error::Transform("fail".into())) })
            .await;

        // timeout is zero so breaker should move to half-open and allow a probe request
        let result = breaker.call(|| async { Ok::<_, Error>("success") }).await;
        assert!(result.is_ok());

        // subsequent requests should flow normally
        let result = breaker.call(|| async { Ok::<_, Error>("again") }).await;
        assert!(result.is_ok());
    }
}
