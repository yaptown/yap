//! Submit all jobs together; only provider enqueued-token limits are retryable.

use std::{collections::BTreeSet, future::Future, time::Duration};
use tysm::{
    batch::{BatchClient, BatchRequestItem, WaitForBatchError},
    chat_completions::{
        BatchChatError, ChatClient, ChatMessage, ChatRequest, JsonSchemaFormat, ResponseFormat,
    },
};

/// A handful of misses should not become a 24-hour batch.
pub const SMALL_BATCH_THRESHOLD: usize = 16;

pub const MAX_REQUESTS: usize = 40_000;
const MAX_BYTES: usize = 180_000_000;

/// Mirror pinned tysm's private request_for_messages and custom-id construction,
/// then use its public batch serializer (not a prompt-length approximation).
/// Keep this field mapping in sync when updating tysm.
fn request_line(
    client: &ChatClient,
    messages: Vec<ChatMessage>,
    format: ResponseFormat,
) -> Vec<u8> {
    let request = ChatRequest {
        model: client.model.clone(),
        messages,
        response_format: format,
        service_tier: client.service_tier.clone(),
        prompt_cache_key: client.prompt_cache_key.clone(),
        reasoning_effort: client.reasoning_effort.clone(),
        extra_body: client.extra_body.clone(),
    };
    let hash =
        xxhash_rust::const_xxh3::xxh3_64(serde_json::to_string(&request).unwrap().as_bytes());
    BatchClient::from(client).create_batch_content(&[BatchRequestItem::new_chat(
        format!("request-{hash}"),
        request,
    )])
}

fn ranges(
    sizes: &[usize],
    max_requests: usize,
    max_bytes: usize,
) -> anyhow::Result<Vec<std::ops::Range<usize>>> {
    let mut ranges = Vec::new();
    let (mut start, mut bytes) = (0, 0);
    for (i, &size) in sizes.iter().enumerate() {
        anyhow::ensure!(
            size <= max_bytes,
            "batch request {i} is {size} serialized bytes, exceeding {max_bytes}-byte file budget"
        );
        if i - start == max_requests || bytes + size > max_bytes {
            ranges.push(start..i);
            start = i;
            bytes = 0;
        }
        bytes += size;
    }
    if start < sizes.len() {
        ranges.push(start..sizes.len());
    }
    Ok(ranges)
}

/// Partition in input order using complete serialized JSONL lines, including
/// response schema, system prompt, custom id, wrappers and trailing newline.
/// Counts/bytes are before cache lookup; cache hits only shrink uploaded files.
pub fn partition<'a, I, T: schemars::JsonSchema>(
    client: &ChatClient,
    items: &'a [I],
    messages: impl Fn(&I) -> Vec<ChatMessage>,
) -> anyhow::Result<Vec<&'a [I]>> {
    let format = ResponseFormat::JsonSchema {
        json_schema: JsonSchemaFormat::new::<T>(),
    };
    let sizes: Vec<_> = items
        .iter()
        .map(|item| request_line(client, messages(item), format.clone()).len())
        .collect();
    ranges(&sizes, MAX_REQUESTS, MAX_BYTES)?.into_iter().map(|range| {
        let bytes: usize = sizes[range.clone()].iter().sum();
        eprintln!("batch plan: {} requests, {bytes} serialized JSONL bytes before cache; bytes/request min={} mean={} max={}", range.len(), sizes[range.clone()].iter().min().unwrap(), bytes / range.len(), sizes[range.clone()].iter().max().unwrap());
        Ok(&items[range])
    }).collect()
}

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
            let limits = limits?;
            limits.into_iter().max_by_key(|q| {
                (
                    q.requested.is_some_and(|n| n > q.limit),
                    std::cmp::Reverse(q.limit),
                )
            })
        }
        BatchChatError::CreateBatchError(tysm::batch::CreateBatchError::OpenAiError(error)) => {
            message_limit(&error.message)
        }
        // Pinned tysm parses create errors without unwrapping the API's error
        // envelope; recover that structured message, never match arbitrary Debug.
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

/// Preserve job order and successes while retrying only queue-limit failures.
/// Every round is one concurrent submission, never a bounded stream. A fatal
/// error stops retries after the other in-flight jobs settle, so callers can
/// audit their successful results before refusing to merge anything.
pub async fn run<'a, J: 'a, T, F, Fut>(jobs: &'a [J], submit: F) -> Vec<anyhow::Result<T>>
where
    F: Fn(&'a J) -> Fut,
    Fut: Future<Output = Result<T, BatchChatError>>,
{
    run_with_wait(jobs, submit, || {
        tokio::time::sleep(Duration::from_secs(600))
    })
    .await
}

async fn run_with_wait<'a, J: 'a, T, F, Fut, W, Wait>(
    jobs: &'a [J],
    submit: F,
    wait: W,
) -> Vec<anyhow::Result<T>>
where
    F: Fn(&'a J) -> Fut,
    Fut: Future<Output = Result<T, BatchChatError>>,
    W: Fn() -> Wait,
    Wait: Future<Output = ()>,
{
    let mut idle_waits = 0;
    let mut pending: Vec<_> = (0..jobs.len()).collect();
    let mut results: Vec<Option<anyhow::Result<T>>> = (0..jobs.len()).map(|_| None).collect();
    let mut limits = BTreeSet::new();
    while !pending.is_empty() {
        let answers = futures::future::join_all(pending.iter().map(|&i| {
            let request = submit(&jobs[i]);
            async move {
                let result = request.await;
                if let Err(error) = &result {
                    // Other jobs may take hours: expose validation details now,
                    // not only when the whole round is ready for retry handling.
                    eprintln!("batch job {i} failed: {error:?}");
                }
                result
            }
        }))
        .await;
        let mut retry = Vec::new();
        let mut completed = false;
        let mut fatal = false;
        for (i, answer) in pending.into_iter().zip(answers) {
            match answer {
                Ok(answer) => {
                    completed = true;
                    results[i] = Some(Ok(answer));
                }
                Err(error) => {
                    if let Some(queue) = queue_limit(&error) {
                        if limits.insert(queue.limit) {
                            eprintln!(
                                "batch enqueued-input-token limit: {}; job size {}",
                                queue.limit,
                                queue
                                    .requested
                                    .map_or_else(|| "unknown".into(), |n| n.to_string())
                            );
                        }
                        if let Some(requested) = queue.requested.filter(|&n| n > queue.limit) {
                            fatal = true;
                            results[i] = Some(Err(anyhow::anyhow!("oversized batch job {i}: requested {requested} input tokens exceeds queue limit {}", queue.limit)));
                            continue;
                        }
                        retry.push(i);
                    } else {
                        fatal = true;
                    }
                    results[i] = Some(Err(anyhow::anyhow!("{error:?}")));
                }
            }
        }
        if fatal || retry.is_empty() {
            break;
        }
        if completed {
            idle_waits = 0;
        } else {
            if idle_waits == 12 {
                for &i in &retry {
                    results[i] = Some(Err(anyhow::anyhow!("batch job {i}: queue still blocked after 12 ten-minute waits; limits {limits:?}; job size unknown or within reported limit")));
                }
                break;
            }
            idle_waits += 1;
            eprintln!("all {} pending batch jobs hit the queue limit; waiting 10 minutes ({idle_waits}/12) before retry (external jobs may occupy the queue)", retry.len());
            wait().await;
        }
        pending = retry;
    }
    results
        .into_iter()
        .map(|result| result.expect("every job settled"))
        .collect()
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
    fn job_boundaries_obey_both_serialized_bytes_and_request_count() {
        assert!(ranges(&[], MAX_REQUESTS, MAX_BYTES).unwrap().is_empty());
        assert_eq!(ranges(&[60, 40, 1], 40_000, 100).unwrap(), [0..2, 2..3]);
        assert_eq!(ranges(&[1, 1, 1], 2, 100).unwrap(), [0..2, 2..3]);
        assert_eq!(
            ranges(&vec![1; 40_001], MAX_REQUESTS, MAX_BYTES).unwrap(),
            [0..40_000, 40_000..40_001]
        );
        assert_eq!(
            ranges(&[MAX_BYTES, 1], MAX_REQUESTS, MAX_BYTES).unwrap(),
            [0..1, 1..2]
        );
        assert!(ranges(&[MAX_BYTES + 1], MAX_REQUESTS, MAX_BYTES).is_err());
    }

    #[test]
    fn line_bytes_include_schema_unicode_escaping_and_newline() {
        let client = ChatClient::new("not-a-real-key", "test-model");
        let format = ResponseFormat::JsonSchema {
            json_schema: JsonSchemaFormat::new::<super::super::CueSplit>(),
        };
        let line = request_line(
            &client,
            vec![
                ChatMessage::system("system"),
                ChatMessage::user("日本語\n\"quoted\""),
            ],
            format,
        );
        assert_eq!(line.last(), Some(&b'\n'));
        let item: serde_json::Value = serde_json::from_slice(&line).unwrap();
        assert_eq!(item["method"], "POST");
        assert_eq!(item["url"], "/v1/chat/completions");
        assert_eq!(
            item["body"]["messages"][1]["content"][0]["text"],
            "日本語\n\"quoted\""
        );
        assert!(item["body"]["response_format"]["json_schema"].is_object());
        assert_eq!(serde_json::to_vec(&item).unwrap().len() + 1, line.len());
        assert!(line.len() > "日本語\n\"quoted\"".len());
    }

    #[test]
    fn non_queue_errors_are_not_retried_and_successes_are_retained() {
        let calls = std::cell::Cell::new(0);
        let results = futures::executor::block_on(run_with_wait(
            &[0, 1, 2],
            |&i| {
                calls.set(calls.get() + 1);
                async move {
                    match i {
                        0 => Ok(0),
                        1 => Err(BatchChatError::CustomIdNotFound("missing".into())),
                        _ => Err(failed(&[
                            "Enqueued token limit reached for gpt-test. Limit: 8000000.",
                        ])),
                    }
                }
            },
            || async { panic!("fatal error must not retry") },
        ));
        assert_eq!(calls.get(), 3);
        assert_eq!(*results[0].as_ref().unwrap(), 0);
        assert!(results[1].is_err());
        assert!(results[2].is_err());
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
        let results = futures::executor::block_on(run_with_wait(
            &[()],
            |_| async {
                Err::<(), _>(failed(&["Enqueued token limit reached for gpt-test. Limit: 8000000. Requested: 9000000."]))
            },
            || async { panic!("oversized jobs must not wait") },
        ));
        assert!(results[0]
            .as_ref()
            .unwrap_err()
            .to_string()
            .contains("requested 9000000 input tokens exceeds queue limit 8000000"));
    }

    #[test]
    fn unknown_job_size_stops_after_twelve_waits() {
        let waits = std::cell::Cell::new(0);
        let results = futures::executor::block_on(run_with_wait(
            &[()],
            |_| async {
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
        assert!(results[0]
            .as_ref()
            .unwrap_err()
            .to_string()
            .contains("12 ten-minute waits"));
    }

    #[test]
    fn only_queue_jobs_retry_after_other_jobs_finish() {
        let calls = [std::cell::Cell::new(0), std::cell::Cell::new(0)];
        let results = futures::executor::block_on(run_with_wait(
            &[0, 1],
            |&i| {
                calls[i].set(calls[i].get() + 1);
                let first = calls[i].get() == 1;
                async move {
                    if i == 1 && first {
                        Err(failed(&[
                            "Enqueued token limit reached for gpt-test. Limit: 8000000.",
                        ]))
                    } else {
                        Ok(i)
                    }
                }
            },
            || async { panic!("local progress permits immediate retry") },
        ));
        assert_eq!(calls.map(|c| c.get()), [1, 2]);
        assert_eq!(
            results.into_iter().map(Result::unwrap).collect::<Vec<_>>(),
            [0, 1]
        );
    }

    #[test]
    fn all_jobs_start_together_and_results_keep_input_order() {
        use std::{cell::Cell, rc::Rc, task::Poll};
        let started = Rc::new(Cell::new(0));
        let jobs: Vec<_> = (0..9).collect();
        let results = futures::executor::block_on(run(&jobs, |&i| {
            let started = started.clone();
            async move {
                started.set(started.get() + 1);
                futures::future::poll_fn(|cx| {
                    if started.get() == 9 {
                        Poll::Ready(Ok(i))
                    } else {
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                })
                .await
            }
        }));
        assert_eq!(
            results.into_iter().map(Result::unwrap).collect::<Vec<_>>(),
            jobs
        );
    }
}
