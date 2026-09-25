//! Single-part S3 uploads to R2, with SDK-managed retries and listing ETags.

use std::{collections::HashMap, time::Duration};

use anyhow::{ensure, Context, Result};
use futures::TryStreamExt;
use object_store::{
    aws::{AmazonS3, AmazonS3Builder},
    path::Path,
    Attribute, Attributes, BackoffConfig, ClientOptions, ObjectStore, PutOptions, RetryConfig,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

pub(crate) struct R2(AmazonS3);

impl R2 {
    pub(crate) async fn from_env(bucket: &str) -> Result<Self> {
        let account = std::env::var("CLOUDFLARE_ACCOUNT_ID")
            .context("CLOUDFLARE_ACCOUNT_ID is required for R2 upload")?;
        let token = std::env::var("CLOUDFLARE_API_TOKEN")
            .context("CLOUDFLARE_API_TOKEN is required for R2 upload")?;
        #[derive(Deserialize)]
        struct Verification {
            result: Token,
        }
        #[derive(Deserialize)]
        struct Token {
            id: String,
        }
        // Do not include the response body or credentials in errors/logging.
        let verification: Verification = reqwest::Client::new()
            .get(format!(
                "https://api.cloudflare.com/client/v4/accounts/{account}/tokens/verify"
            ))
            .bearer_auth(&token)
            .timeout(Duration::from_secs(60))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
            .context("decoding Cloudflare account token verification")?;
        ensure!(
            verification.result.id.len() == 32
                && verification
                    .result
                    .id
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit()),
            "Cloudflare token verification returned an invalid token id"
        );
        let store = AmazonS3Builder::new()
            .with_endpoint(format!("https://{account}.r2.cloudflarestorage.com"))
            .with_region("auto")
            .with_bucket_name(bucket)
            .with_access_key_id(verification.result.id)
            .with_secret_access_key(format!("{:x}", Sha256::digest(token.as_bytes())))
            .with_virtual_hosted_style_request(false)
            // The default per-request timeout is 30 s. With UPLOAD_JOBS puts
            // sharing the uplink a 30 MB hi.mp4 can't finish in that, and it
            // then fails identically on every retry (2026-09-20, fra).
            .with_client_options(ClientOptions::new().with_timeout(Duration::from_secs(900)))
            .with_retry(RetryConfig {
                max_retries: 10,
                retry_timeout: Duration::from_secs(600),
                backoff: BackoffConfig {
                    init_backoff: Duration::from_secs(1),
                    max_backoff: Duration::from_secs(60),
                    base: 2.0,
                },
            })
            .build()?;
        Ok(Self(store))
    }

    pub(crate) async fn put(
        &self,
        key: &str,
        bytes: Vec<u8>,
        content_type: &str,
        cache_control: &str,
    ) -> Result<()> {
        let mut attributes = Attributes::new();
        attributes.insert(Attribute::ContentType, content_type.to_owned().into());
        attributes.insert(Attribute::CacheControl, cache_control.to_owned().into());
        self.0
            .put_opts(
                &Path::from(key),
                bytes.into(),
                PutOptions {
                    attributes,
                    ..Default::default()
                },
            )
            .await
            .with_context(|| format!("uploading {key}"))?;
        Ok(())
    }

    pub(crate) async fn delete(&self, keys: Vec<String>) -> Result<()> {
        let paths = futures::stream::iter(keys.into_iter().map(|k| Ok(Path::from(k))));
        self.0
            .delete_stream(Box::pin(paths))
            .try_collect::<Vec<_>>()
            .await
            .context("deleting objects")?;
        Ok(())
    }

    pub(crate) async fn list_etags(&self, prefix: &str) -> Result<HashMap<String, String>> {
        let path = Path::from(prefix);
        let mut objects = self.0.list(Some(&path));
        let mut etags = HashMap::new();
        while let Some(object) = objects.try_next().await? {
            if let Some(etag) = object.e_tag {
                etags.insert(
                    object.location.to_string(),
                    etag.trim_matches('"').to_owned(),
                );
            }
        }
        Ok(etags)
    }
}
