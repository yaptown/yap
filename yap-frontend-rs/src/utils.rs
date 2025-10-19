use opfs::{DirectoryHandle as _, FileHandle as _, WritableFileStream as _, persistent};

/// Performance instrumentation helper that measures time from creation to drop.
/// Logs the duration to the console when dropped.
pub struct PerfTimer {
    label: String,
    start_time: f64,
}

impl PerfTimer {
    /// Create a new performance timer with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        let start_time = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);

        Self {
            label: label.into(),
            start_time,
        }
    }
}

impl Drop for PerfTimer {
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            if let Some(performance) = window.performance() {
                let duration = performance.now() - self.start_time;
                log::info!("[PERF] {}: {:.2}ms", self.label, duration);
            }
        }
    }
}

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub(crate) async fn get_or_create_device_id(
    weapon_dir: &persistent::DirectoryHandle,
    user_id: &Option<String>,
) -> Result<String, persistent::Error> {
    let file_name = if user_id.is_some() {
        "device-id"
    } else {
        "device-id-logged-out"
    };

    let device_id_file = weapon_dir
        .get_file_handle_with_options(file_name, &opfs::GetFileHandleOptions { create: false })
        .await;

    match device_id_file {
        Ok(file_handle) => {
            // Read existing device ID
            let bytes = file_handle.read().await?;
            let device_id = String::from_utf8(bytes).unwrap_or_else(|_| {
                log::error!("Device ID file contained invalid UTF-8 data");
                eyedee::get_uuid()
            });
            Ok(device_id)
        }
        Err(_) => {
            // Generate new device ID
            let device_id = eyedee::get_uuid();

            // Save it to OPFS
            let mut file_handle = weapon_dir
                .get_file_handle_with_options(
                    file_name,
                    &opfs::GetFileHandleOptions { create: true },
                )
                .await?;

            let mut writable = file_handle
                .create_writable_with_options(&opfs::CreateWritableOptions {
                    keep_existing_data: false,
                })
                .await?;

            writable
                .write_at_cursor_pos(device_id.as_bytes().to_vec())
                .await?;

            writable.close().await?;

            Ok(device_id)
        }
    }
}

pub async fn hit_ai_server(
    method: fetch_happen::Method,
    path: &str,
    request: Option<impl serde::Serialize>,
    access_token: Option<&String>,
) -> Result<fetch_happen::Response, fetch_happen::Error> {
    let client = fetch_happen::Client;
    let url = if cfg!(feature = "local-backend") {
        "http://localhost:8080"
    } else {
        "https://yap-ai-backend.fly.dev"
    };
    // Always include an Authorization header - use "anonymous" as dummy token when not logged in
    let token = access_token.map(|t| t.as_str()).unwrap_or("anonymous");

    let full_url = format!("{url}{path}");

    let mut req = match method {
        fetch_happen::Method::GET => client.get(&full_url),
        fetch_happen::Method::POST => client.post(&full_url),
        fetch_happen::Method::PATCH => client.patch(&full_url),
        fetch_happen::Method::PUT => client.put(&full_url),
        fetch_happen::Method::DELETE => client.delete(&full_url),
        _ => panic!("Unsupported HTTP method"),
    };

    req = req.header("Authorization", format!("Bearer {token}"));

    if let Some(body) = request {
        req = req.json(&body)?;
    }

    let response = req.send().await?;
    Ok(response)
}
