//! Retry a whole stage only for provider enqueued-token limits.

use std::{future::Future, time::Duration};
use tysm::{batch::WaitForBatchError, chat_completions::BatchChatError};

/// Misses up to this many go out as live calls instead of a batch. A batch
/// takes hours however small it is, so the small follow-up rounds of a
/// pipeline (retries, verification, escape repairs) would otherwise each add
/// hours of wall time; live prices on a couple of thousand requests cost cents.
pub const SMALL_BATCH_THRESHOLD: usize = 2_000;

#[derive(Debug, PartialEq, Eq)]
struct QueueLimit {
    limit: u64,
    requested: Option<u64>,
}

fn token_count(text: &str) -> Option<u64> {
    text.trim_start()
        .split(|c: char| !c.is_ascii_digit() && c != ',')
        .next()?
        .replace(',', "")
        .parse()
        .ok()
}

fn message_limit(message: &str) -> Option<QueueLimit> {
    let rest = message.strip_prefix("Enqueued token limit reached for ")?;
    let (_, limit) = rest.split_once("Limit:")?;
    Some(QueueLimit {
        limit: token_count(limit)?,
        // Only use an explicit upstream count; bytes are not tokens.
        requested: rest
            .split_once("Requested:")
            .and_then(|(_, n)| token_count(n)),
    })
}

fn queue_limit(error: &BatchChatError) -> Option<QueueLimit> {
    match error {
        BatchChatError::WaitForBatchError(WaitForBatchError::BatchFailed { error, .. }) => {
            let errors: serde_json::Value = serde_json::from_str(error).ok()?;
            let errors = errors.get("data")?.as_array()?;
            // Mixed validation errors must not become an infinite retry.
            let limits: Option<Vec<_>> = errors
                .iter()
                .map(|error| message_limit(error.get("message")?.as_str()?))
                .collect();
            limits?.into_iter().max_by_key(|q| {
                (
                    q.requested.is_some_and(|n| n > q.limit),
                    std::cmp::Reverse(q.limit),
                )
            })
        }
        BatchChatError::CreateBatchError(tysm::batch::CreateBatchError::OpenAiError(error)) => {
            message_limit(&error.message)
        }
        BatchChatError::CreateBatchError(tysm::batch::CreateBatchError::JsonParseError(
            _,
            body,
        )) => {
            let body: serde_json::Value = serde_json::from_str(body).ok()?;
            message_limit(body.get("error")?.get("message")?.as_str()?)
        }
        _ => None,
    }
}

/// Submit the whole stage; tysm owns partitioning and caches successful requests.
/// Other stages/processes can still occupy the provider's enqueued-token queue.
pub async fn run<T, F, Fut>(submit: F) -> anyhow::Result<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, BatchChatError>>,
{
    run_with_wait(submit, || tokio::time::sleep(Duration::from_secs(600))).await
}

async fn run_with_wait<T, F, Fut, W, Wait>(submit: F, wait: W) -> anyhow::Result<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, BatchChatError>>,
    W: Fn() -> Wait,
    Wait: Future<Output = ()>,
{
    for waits in 0..=12 {
        let error = match submit().await {
            Ok(answer) => return Ok(answer),
            Err(error) => error,
        };
        let Some(queue) = queue_limit(&error) else {
            return Err(error.into());
        };
        if let Some(requested) = queue.requested.filter(|&n| n > queue.limit) {
            anyhow::bail!(
                "oversized batch: requested {requested} input tokens exceeds queue limit {}",
                queue.limit
            );
        }
        if waits == 12 {
            anyhow::bail!(
                "batch queue still blocked after 12 ten-minute waits; last error: {error:?}"
            );
        }
        eprintln!(
            "batch enqueued-token queue blocked: {error:?}; waiting 10 minutes ({}/12) before retry",
            waits + 1
        );
        wait().await;
    }
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failed(messages: &[&str]) -> BatchChatError {
        BatchChatError::WaitForBatchError(WaitForBatchError::BatchFailed {
            id: "batch_test".into(),
            error: serde_json::json!({"data": messages.iter().map(|message| serde_json::json!({"message":message})).collect::<Vec<_>>()}).to_string(),
        })
    }

    #[test]
    fn non_queue_errors_are_not_retried() {
        let result = futures::executor::block_on(run_with_wait(
            || async { Err::<(), _>(BatchChatError::CustomIdNotFound("missing".into())) },
            || async { panic!("fatal error must not retry") },
        ));
        assert!(result.is_err());
    }

    #[test]
    fn only_enqueued_token_validation_errors_are_retryable() {
        let queue = "Enqueued token limit reached for gpt-test in organization org-test. Limit: 8000000 enqueued tokens.";
        assert_eq!(
            queue_limit(&failed(&[queue])),
            Some(QueueLimit {
                limit: 8_000_000,
                requested: None
            })
        );
        assert_eq!(message_limit("Enqueued token limit reached for gpt-test. Limit: 8,000,000. Requested: 9,000,000."), Some(QueueLimit { limit: 8_000_000, requested: Some(9_000_000) }));
        assert_eq!(queue_limit(&failed(&[queue, "Invalid JSON"])), None);
        assert_eq!(queue_limit(&failed(&[])), None);
        assert_eq!(queue_limit(&failed(&["CustomIdNotFound"])), None);
        assert_eq!(queue_limit(&failed(&["Rate limit reached"])), None);
        assert_eq!(
            message_limit("Enqueued token limit reached for gpt-test"),
            None
        );
    }

    #[test]
    fn oversized_jobs_stop_without_waiting() {
        let result = futures::executor::block_on(run_with_wait(
            || async {
                Err::<(), _>(failed(&["Enqueued token limit reached for gpt-test. Limit: 8000000. Requested: 9000000."]))
            },
            || async { panic!("oversized jobs must not wait") },
        ));
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requested 9000000 input tokens exceeds queue limit 8000000"));
    }

    #[test]
    fn unknown_job_size_stops_after_twelve_waits() {
        let waits = std::cell::Cell::new(0);
        let result = futures::executor::block_on(run_with_wait(
            || async {
                Err::<(), _>(failed(&[
                    "Enqueued token limit reached for gpt-test. Limit: 8000000.",
                ]))
            },
            || {
                waits.set(waits.get() + 1);
                std::future::ready(())
            },
        ));
        assert_eq!(waits.get(), 12);
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("12 ten-minute waits"));
    }

    #[test]
    fn queue_failure_retries_the_stage_after_waiting() {
        let calls = std::cell::Cell::new(0);
        let waits = std::cell::Cell::new(0);
        let result = futures::executor::block_on(run_with_wait(
            || {
                calls.set(calls.get() + 1);
                let first = calls.get() == 1;
                async move {
                    if first {
                        Err(failed(&[
                            "Enqueued token limit reached for gpt-test. Limit: 8000000.",
                        ]))
                    } else {
                        Ok(vec![0, 1, 2])
                    }
                }
            },
            || {
                waits.set(waits.get() + 1);
                std::future::ready(())
            },
        ));
        assert_eq!(calls.get(), 2);
        assert_eq!(waits.get(), 1);
        assert_eq!(result.unwrap(), [0, 1, 2]);
    }
}
