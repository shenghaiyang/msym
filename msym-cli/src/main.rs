mod cli;
mod config;
mod download;
mod util;

use crate::cli::Cli;
use crate::config::read_config;
use crate::download::download_icon;
use crate::util::icon_to_filename;
use anstyle::AnsiColor;
use anyhow::{Context, Result};
use clap::Parser;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = read_config(&cli.config)?;
    let config = Arc::new(config.resolve()?);

    let dim = AnsiColor::BrightBlack.on_default();
    let rst = AnsiColor::White.on_default();

    anstream::println!(
        "  {}style:{} {}  {}output:{} {}  {}package:{} {}",
        dim.render(),
        rst.render(),
        config.style,
        dim.render(),
        rst.render(),
        config.compose_output_dir.display(),
        dim.render(),
        rst.render(),
        config.compose_package,
    );
    anstream::println!(
        "  {}icons:{} {}  ·  {}jobs:{} {}\n",
        dim.render(),
        rst.render(),
        config.icons.len(),
        dim.render(),
        rst.render(),
        cli.jobs,
    );

    tokio::fs::create_dir_all(&config.compose_output_dir).await?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("Failed to create HTTP client")?;

    let concurrency = cli.jobs as usize;
    let show_progress = concurrency == 1;

    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut handles = Vec::new();
    let mut skipped = 0u32;

    let y = AnsiColor::Yellow.on_default();
    for icon in &config.icons {
        let filename = format!("{}.kt", icon_to_filename(icon));
        let output_path = config.compose_output_dir.join(&filename);

        if !cli.force && output_path.exists() {
            anstream::println!("  {}-{} {:<18} (skipped)", y.render(), rst.render(), icon);
            skipped += 1;
            continue;
        }

        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("semaphore closed");

        let client = client.clone();
        let config = config.clone();
        let icon = icon.clone();
        let verbose = cli.verbose;

        handles.push(tokio::spawn(async move {
            let _permit = permit;
            let result = download_icon(&client, &config, &icon, show_progress, verbose).await;
            (icon, result)
        }));
    }

    let downloaded = handles.len() as u32;
    let r = AnsiColor::Red.on_default();
    let mut errors: Vec<(String, anyhow::Error)> = Vec::new();
    for handle in handles {
        match handle.await {
            Ok((_icon, Ok(()))) => {}
            Ok((icon, Err(e))) => errors.push((icon, e)),
            Err(join_err) => {
                anstream::eprintln!(
                    "  {}Task panicked:{} {:#}",
                    r.render(),
                    rst.render(),
                    join_err
                );
            }
        }
    }

    if !errors.is_empty() {
        anstream::eprintln!("\n  {}error(s):{}", r.render(), rst.render());
        for (icon, e) in &errors {
            anstream::eprintln!("  {}✗{} {}: {:#}", r.render(), rst.render(), icon, e);
        }
    }

    let g = AnsiColor::Green.on_default();
    anstream::println!(
        "{}Done:{} {} downloaded{}.",
        g.render(),
        rst.render(),
        downloaded,
        if skipped > 0 {
            format!(", {} skipped", skipped)
        } else {
            String::new()
        }
    );

    Ok(())
}
