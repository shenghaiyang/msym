use crate::http::{fetch_body, format_size};
use crate::kotlin::transform;
use crate::task::Task;
use anstyle::AnsiColor;
use anyhow::{Context, Result};

/// Download a single icon and write the processed Kotlin file.
pub async fn download_task(
    client: &reqwest::Client,
    task: &Task,
    show_progress: bool,
    verbose: bool,
) -> Result<()> {
    if verbose {
        let dim = AnsiColor::BrightBlack.on_default();
        let rst = AnsiColor::White.on_default();
        anstream::eprintln!("  {}{}{}", dim.render(), task.url, rst.render());
    }

    let body_bytes = fetch_body(client, &task.url, &task.symbol_name, show_progress).await?;

    let body = String::from_utf8_lossy(&body_bytes).into_owned();
    let modified = transform(&body, &task.transform);

    if let Some(parent) = task.output_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| "Failed to create output directory")?;
    }

    tokio::fs::write(&task.output_path, modified)
        .await
        .with_context(|| format!("Failed to write file: {}", task.output_path.display()))?;

    let size = format_size(body_bytes.len() as u64);
    let green = AnsiColor::Green.on_default();
    let rst = AnsiColor::White.on_default();
    anstream::println!(
        "  {}✓{} {:<18} {:<6}  →  {}",
        green.render(),
        rst.render(),
        task.symbol_name,
        size,
        task.output_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy(),
    );
    Ok(())
}
