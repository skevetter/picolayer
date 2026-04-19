use anyhow::Result;
use log::warn;
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

use crate::cli::RetryConfig;

/// Execute a function with retry logic and exponential backoff
pub async fn retry_async<F, Fut, T>(
    config: &RetryConfig,
    operation_name: &str,
    mut operation: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    if config.max_retries == 0 {
        return operation().await;
    }

    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                last_error = Some(err);

                if attempt < config.max_retries {
                    let delay_ms = (config.initial_delay_ms as f64
                        * config.backoff_multiplier.powi(attempt as i32))
                        as u64;

                    warn!(
                        "{} failed (attempt {}/{}), retrying in {}ms: {}",
                        operation_name,
                        attempt + 1,
                        config.max_retries + 1,
                        delay_ms,
                        last_error.as_ref().unwrap()
                    );

                    sleep(Duration::from_millis(delay_ms)).await;
                } else {
                    warn!(
                        "{} failed after {} attempts",
                        operation_name,
                        config.max_retries + 1
                    );
                }
            }
        }
    }

    Err(last_error.unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn test_config(max_retries: u32) -> RetryConfig {
        RetryConfig {
            max_retries,
            initial_delay_ms: 1, // 1ms for fast tests
            backoff_multiplier: 1.0,
        }
    }

    #[tokio::test]
    async fn retry_succeeds_on_first_attempt() {
        let result = retry_async(&test_config(3), "test", || async {
            Ok::<_, anyhow::Error>(42)
        })
        .await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn retry_succeeds_after_failures() {
        let attempts = AtomicU32::new(0);
        let result = retry_async(&test_config(3), "test", || {
            let count = attempts.fetch_add(1, Ordering::SeqCst);
            async move {
                if count < 2 {
                    Err(anyhow::anyhow!("transient failure"))
                } else {
                    Ok(42)
                }
            }
        })
        .await;
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 3); // 2 failures + 1 success
    }

    #[tokio::test]
    async fn retry_exhausts_all_attempts() {
        let attempts = AtomicU32::new(0);
        let result: Result<i32> = retry_async(&test_config(2), "test", || {
            attempts.fetch_add(1, Ordering::SeqCst);
            async { Err(anyhow::anyhow!("persistent failure")) }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 3); // initial + 2 retries
    }

    #[tokio::test]
    async fn retry_zero_retries_runs_once() {
        let attempts = AtomicU32::new(0);
        let result: Result<i32> = retry_async(&test_config(0), "test", || {
            attempts.fetch_add(1, Ordering::SeqCst);
            async { Err(anyhow::anyhow!("failure")) }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }
}
