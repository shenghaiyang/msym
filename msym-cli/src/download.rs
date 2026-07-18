use crate::config::ResolvedConfig;
use crate::util::{capitalize, icon_to_filename};
use anstyle::AnsiColor;
use anstyle_progress::TermProgress;
use anyhow::{Context, Result};
use std::io::Write;
use std::time::Duration;
use tokio::time::sleep;

/// Build the download URL for a given icon.
pub fn build_url(config: &ResolvedConfig, icon_name: &str) -> String {
    let style_name = capitalize(&config.style);
    let fill_val = if config.fill { 1 } else { 0 };
    let rond = if config.style.to_lowercase() == "rounded" {
        50
    } else {
        0
    };

    format!(
        "https://fonts.gstatic.com/render/v1/Material+Symbols+{}/{dp}dp/{icon}.kt?var=opsz,wght,FILL,GRAD,ROND@{opsz},{wght},{fill},{grade},{rond}",
        style_name,
        dp = config.optical_size,
        icon = icon_name,
        opsz = config.optical_size,
        wght = config.weight,
        fill = fill_val,
        grade = config.grade,
        rond = rond,
    )
}

/// Replace the package declaration in Kotlin source, or insert one if missing.
fn set_kotlin_package(source: &str, package: &str) -> String {
    let package_line = format!("package {}", package);

    if let Some(line_start) = source
        .lines()
        .position(|line| line.trim_start().starts_with("package "))
    {
        let lines: Vec<&str> = source.lines().collect();
        let mut new_lines = lines.clone();
        new_lines[line_start] = &package_line;
        return new_lines.join("\n") + "\n";
    }

    format!("{}\n{}", package_line, source)
}

/// Download a single icon and save it (with retry).
pub async fn download_icon(
    client: &reqwest::Client,
    config: &ResolvedConfig,
    icon_name: &str,
    show_progress: bool,
    verbose: bool,
) -> Result<()> {
    let url = build_url(config, icon_name);
    let filename = format!("{}.kt", icon_to_filename(icon_name));

    if verbose {
        let dim = AnsiColor::BrightBlack.on_default();
        let rst = AnsiColor::White.on_default();
        anstream::eprintln!("  {}{}{}", dim.render(), url, rst.render());
    }

    let body_bytes = download_with_retry(client, &url, icon_name, show_progress, verbose).await?;

    let body = String::from_utf8_lossy(&body_bytes).into_owned();
    let modified = set_kotlin_package(&body, &config.compose_package);

    tokio::fs::create_dir_all(&config.compose_output_dir)
        .await
        .with_context(|| "Failed to create output directory")?;

    let output_path = config.compose_output_dir.join(&filename);
    tokio::fs::write(&output_path, modified)
        .await
        .with_context(|| format!("Failed to write file: {}", output_path.display()))?;

    let size = format_size(body_bytes.len() as u64);
    let green = AnsiColor::Green.on_default();
    let rst = AnsiColor::White.on_default();
    anstream::println!(
        "  {}✓{} {:<18} {:<6}  →  {}",
        green.render(),
        rst.render(),
        icon_name,
        size,
        filename,
    );
    Ok(())
}

/// Download with up to 3 retries and exponential backoff (1s, 2s, 4s).
async fn download_with_retry(
    client: &reqwest::Client,
    url: &str,
    icon_name: &str,
    show_progress: bool,
    verbose: bool,
) -> Result<Vec<u8>> {
    let mut last_err = None;
    for attempt in 0..3 {
        if attempt > 0 {
            let delay = Duration::from_secs(1 << (attempt - 1));
            if verbose {
                let y = AnsiColor::Yellow.on_default();
                let rst = AnsiColor::White.on_default();
                anstream::eprintln!(
                    "  {}⚠{} retrying {} (attempt {}/3) in {}s…",
                    y.render(),
                    rst.render(),
                    icon_name,
                    attempt + 1,
                    delay.as_secs()
                );
            }
            sleep(delay).await;
        }

        match try_download(client, url, icon_name, show_progress).await {
            Ok(body) => return Ok(body),
            Err(e) => {
                if verbose {
                    let r = AnsiColor::Red.on_default();
                    let rst = AnsiColor::White.on_default();
                    anstream::eprintln!(
                        "  {}✗{} {} attempt {}: {:#}",
                        r.render(),
                        rst.render(),
                        icon_name,
                        attempt + 1,
                        e,
                    );
                }
                last_err = Some(e);
            }
        }
    }
    Err(last_err.unwrap())
}

/// Single download attempt — returns the raw (decompressed) body bytes.
async fn try_download(
    client: &reqwest::Client,
    url: &str,
    icon_name: &str,
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
    anstream::eprint!("{}", TermProgress::start());
    let _ = std::io::stderr().flush();

    loop {
        let chunk = response
            .chunk()
            .await
            .with_context(|| format!("Failed to read response body for icon '{}'", icon_name))?;

        match chunk {
            Some(chunk) => {
                body_bytes.extend_from_slice(&chunk);
                if show_progress {
                    print_progress(body_bytes.len() as u64);
                }
            }
            None => break,
        }
    }

    anstream::eprint!("{}", TermProgress::remove());
    let _ = std::io::stderr().flush();

    if show_progress {
        anstream::eprint!("\r\x1b[2K");
    }

    Ok(body_bytes)
}

/// Print download progress to stderr (bytes so far).
fn print_progress(bytes: u64) {
    anstream::eprint!("\r  ... {} downloaded", format_size(bytes));
    let _ = std::io::stderr().flush();
}

/// Format a byte size for human display.
fn format_size(bytes: u64) -> String {
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
