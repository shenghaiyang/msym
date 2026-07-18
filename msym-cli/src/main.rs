mod cli;
mod config;
mod download;
mod executor;
mod http;
mod kotlin;
mod task;

use crate::cli::Cli;
use crate::config::Config;
use crate::executor::Executor;
use crate::task::build_tasks;
use anstyle::AnsiColor;
use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Stage 1: Parse & resolve config
    let config = Config::parse_file(&cli.config).await?;

    // Stage 2: Build download tasks
    let tasks = build_tasks(&config);

    // Print summary
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
        tasks.len(),
        dim.render(),
        rst.render(),
        cli.jobs,
    );

    // Stage 3: Execute
    let executor = Executor::new(cli.jobs as usize, cli.force, cli.verbose)?;
    let result = executor.execute(&tasks, &config.compose_output_dir).await;

    // Stage 4: Report
    if !result.errors.is_empty() {
        let r = AnsiColor::Red.on_default();
        anstream::eprintln!("\n  {}error(s):{}", r.render(), rst.render());
        for (icon, e) in &result.errors {
            anstream::eprintln!("  {}✗{} {}: {:#}", r.render(), rst.render(), icon, e);
        }
    }

    let g = AnsiColor::Green.on_default();
    anstream::println!(
        "{}Done:{} {} downloaded{}.",
        g.render(),
        rst.render(),
        result.downloaded,
        if result.skipped > 0 {
            format!(", {} skipped", result.skipped)
        } else {
            String::new()
        }
    );

    Ok(())
}
