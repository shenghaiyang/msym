use crate::config::symbol_name::SymbolName;
use crate::download::download_task;
use crate::task::Task;
use anstyle::AnsiColor;
use anyhow::{Context, Result};
use reqwest::Client;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

/// Result of executing a batch of download tasks.
pub struct ExecutionResult {
    pub downloaded: u32,
    pub skipped: u32,
    pub errors: Vec<(SymbolName, anyhow::Error)>,
}

/// Executes download tasks concurrently with configurable concurrency.
pub struct Executor {
    client: Client,
    concurrency: usize,
    force: bool,
    verbose: bool,
}

impl Executor {
    pub fn new(concurrency: usize, force: bool, verbose: bool) -> Result<Self> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(3))
            .read_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(10))
            .connection_verbose(verbose)
            .build()
            .context("Failed to create HTTP client")?;
        Ok(Executor {
            client,
            concurrency,
            force,
            verbose,
        })
    }

    pub async fn execute(&self, tasks: &[Task], output_dir: &Path) -> Result<ExecutionResult> {
        tokio::fs::create_dir_all(output_dir)
            .await
            .context("Failed to create output directory")?;

        let show_progress = self.concurrency == 1;
        let semaphore = Arc::new(Semaphore::new(self.concurrency));
        let mut handles = Vec::new();
        let mut skipped = 0u32;

        let y = AnsiColor::Yellow.on_default();
        let rst = AnsiColor::White.on_default();

        for task in tasks {
            if !self.force && task.output_path.exists() {
                anstream::println!(
                    "  {}-{} {:<18} (skipped)",
                    y.render(),
                    rst.render(),
                    task.symbol_name,
                );
                skipped += 1;
                continue;
            }

            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .expect("semaphore closed");

            let client = self.client.clone();
            let task = task.clone();
            let verbose = self.verbose;

            handles.push(tokio::spawn(async move {
                let _permit = permit;
                let name = task.symbol_name.clone();
                let result = download_task(&client, &task, show_progress, verbose).await;
                (name, result)
            }));
        }

        let r = AnsiColor::Red.on_default();
        let mut errors: Vec<(SymbolName, anyhow::Error)> = Vec::new();
        let mut succeeded = 0u32;

        for handle in handles {
            match handle.await {
                Ok((_icon, Ok(()))) => succeeded += 1,
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

        Ok(ExecutionResult {
            downloaded: succeeded,
            skipped,
            errors,
        })
    }
}
