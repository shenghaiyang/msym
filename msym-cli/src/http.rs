use crate::config::symbol_name::SymbolName;
use anstyle_progress::TermProgress;
use anyhow::{Context, Result};
use std::io::Write;

/// Single HTTP GET with chunked progress display. Returns the raw body bytes.
pub async fn fetch_body(
    client: &reqwest::Client,
    url: &str,
    icon_name: &SymbolName,
    show_progress: bool,
) -> Result<Vec<u8>> {
    let mut response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("Failed to download icon '{}'", icon_name))?;

    if !response.status().is_success() {
        anyhow::bail!(
            "HTTP {} when downloading icon '{}' from: {}",
            response.status(),
            icon_name,
            url
        );
    }

    let mut body_bytes = Vec::new();
    if show_progress {
        anstream::eprint!("{}", TermProgress::start());
        let _ = std::io::stderr().flush();
    }

    loop {
        let chunk = response
            .chunk()
            .await
            .with_context(|| format!("Failed to read response body for icon '{}'", icon_name))?;
        match chunk {
            Some(chunk) => {
                body_bytes.extend_from_slice(&chunk);
                if show_progress {
                    anstream::eprint!(
                        "\r  ... {} downloaded",
                        format_size(body_bytes.len() as u64)
                    );
                    let _ = std::io::stderr().flush();
                }
            }
            None => break,
        }
    }

    if show_progress {
        anstream::eprint!("{}", TermProgress::remove());
        let _ = std::io::stderr().flush();
        anstream::eprint!("\r\x1b[2K");
    }

    Ok(body_bytes)
}

/// Format a byte size for human display.
pub fn format_size(bytes: u64) -> String {
    let units = ["B", "kB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut idx = 0;
    while size >= 1000.0 && idx < units.len() - 1 {
        size /= 1000.0;
        idx += 1;
    }
    if idx == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.1} {}", size, units[idx])
    }
}
