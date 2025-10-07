use crate::error::Result;
use std::future::Future;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    max_attempts: usize,
    base_delay: Duration,
    max_delay: Duration,
}

impl RetryPolicy {
    pub fn new(max_attempts: usize, base_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_attempts: max_attempts.max(1),
            base_delay,
            max_delay,
        }
    }

    pub fn max_attempts(&self) -> usize {
        self.max_attempts
    }

    fn backoff_delay(&self, attempt: usize) -> Duration {
        if attempt <= 1 {
            return self.base_delay.min(self.max_delay);
        }

        let mut delay = self.base_delay;
        for _ in 1..attempt {
            delay = delay.checked_mul(2).unwrap_or(self.max_delay);
            if delay >= self.max_delay {
                return self.max_delay;
            }
        }

        delay.min(self.max_delay)
    }

    pub async fn execute<F, Fut, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T>> + Send,
        T: Send,
    {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match operation().await {
                Ok(value) => return Ok(value),
                Err(err) => {
                    if attempt >= self.max_attempts {
                        return Err(err);
                    }
                    let delay = self.backoff_delay(attempt);
                    if !delay.is_zero() {
                        tokio::time::sleep(delay).await;
                    } else {
                        tokio::task::yield_now().await;
                    }
                }
            }
        }
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn retries_until_success() {
        let policy = RetryPolicy::new(3, Duration::from_millis(0), Duration::from_millis(0));
        let attempts = Arc::new(Mutex::new(0));
        let result = policy
            .execute({
                let attempts = attempts.clone();
                move || {
                    let attempts = attempts.clone();
                    Box::pin(async move {
                        let mut guard = attempts.lock().unwrap();
                        *guard += 1;
                        if *guard < 2 {
                            Err(Error::Transform("fail".into()))
                        } else {
                            Ok("ok")
                        }
                    })
                }
            })
            .await;

        assert_eq!(*attempts.lock().unwrap(), 2);
        assert_eq!(result.unwrap(), "ok");
    }

    #[tokio::test]
    async fn stops_after_max_attempts() {
        let policy = RetryPolicy::new(2, Duration::from_millis(0), Duration::from_millis(0));
        let attempts = Arc::new(Mutex::new(0));
        let result: Result<()> = policy
            .execute({
                let attempts = attempts.clone();
                move || {
                    let attempts = attempts.clone();
                    Box::pin(async move {
                        let mut guard = attempts.lock().unwrap();
                        *guard += 1;
                        Err(Error::Transform("still failing".into()))
                    })
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(*attempts.lock().unwrap(), 2);
    }

    #[test]
    fn backoff_caps_at_max_delay() {
        let policy = RetryPolicy::new(5, Duration::from_millis(100), Duration::from_millis(350));

        assert_eq!(policy.backoff_delay(1), Duration::from_millis(100));
        assert_eq!(policy.backoff_delay(2), Duration::from_millis(200));
        assert_eq!(policy.backoff_delay(3), Duration::from_millis(350));
        assert_eq!(policy.backoff_delay(4), Duration::from_millis(350));
    }
}
