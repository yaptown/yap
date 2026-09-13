//! Retry transport failures of single live chat requests.
//!
//! A pooled keep-alive connection the server has already closed surfaces on
//! its next use as "error sending request: connection closed via error".
//! tysm doesn't retry it, and one such blip at the slot-analysis step has
//! aborted runs hours in. Only transport errors are retried; an API error
//! or an unparseable response is returned as is. Batch calls handle their
//! own per-item failures and don't need this.

use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use tysm::chat_completions::{ChatClient, ChatError};

const ATTEMPTS: u32 = 4;

#[allow(async_fn_in_trait)]
pub trait ChatRetry {
    async fn chat_with_system_prompt_retrying<T: DeserializeOwned + JsonSchema>(
        &self,
        system_prompt: impl Into<String>,
        prompt: impl Into<String>,
    ) -> Result<T, ChatError>;
}

impl ChatRetry for ChatClient {
    async fn chat_with_system_prompt_retrying<T: DeserializeOwned + JsonSchema>(
        &self,
        system_prompt: impl Into<String>,
        prompt: impl Into<String>,
    ) -> Result<T, ChatError> {
        let (system_prompt, prompt) = (system_prompt.into(), prompt.into());
        for attempt in 1..=ATTEMPTS {
            match self
                .chat_with_system_prompt::<T>(system_prompt.clone(), prompt.clone())
                .await
            {
                Err(ChatError::RequestError(e)) if attempt < ATTEMPTS => {
                    let delay = std::time::Duration::from_secs(1 << attempt);
                    eprintln!(
                        "WARNING: chat request failed on attempt {attempt}/{ATTEMPTS} ({e}), \
                         retrying in {}s",
                        delay.as_secs()
                    );
                    tokio::time::sleep(delay).await;
                }
                result => return result,
            }
        }
        unreachable!("the last attempt returns")
    }
}
