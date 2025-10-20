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
