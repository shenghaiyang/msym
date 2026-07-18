use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// Configuration parsed from msym.toml
#[derive(Debug, Deserialize)]
pub struct Config {
    pub style: Option<String>,
    pub fill: Option<bool>,
    pub weight: Option<u32>,
    pub grade: Option<i32>,
    pub optical_size: Option<u32>,
    pub icons: Vec<String>,
    pub compose_package: String,
    pub compose_output_dir: Option<String>,
}

#[derive(Debug)]
pub struct ResolvedConfig {
    pub style: String,
    pub fill: bool,
    pub weight: u32,
    pub grade: i32,
    pub optical_size: u32,
    pub icons: Vec<String>,
    pub compose_package: String,
    pub compose_output_dir: PathBuf,
}

impl Config {
    pub fn resolve(self) -> Result<ResolvedConfig> {
        let style = self.style.unwrap_or_else(|| "rounded".to_string());

        if !matches!(
            style.to_lowercase().as_str(),
            "outlined" | "rounded" | "sharp"
        ) {
            anyhow::bail!(
                "Invalid style '{}': must be one of Outlined, Rounded, Sharp",
                style
            );
        }

        let fill = self.fill.unwrap_or(false);
        let weight = self.weight.unwrap_or(400);

        if ![100, 200, 300, 400, 500, 600, 700].contains(&weight) {
            anyhow::bail!(
                "Invalid weight '{}': must be one of 100, 200, 300, 400, 500, 600, 700",
                weight
            );
        }

        let grade = self.grade.unwrap_or(0);

        if ![-25, 0, 200].contains(&grade) {
            anyhow::bail!("Invalid grade '{}': must be one of -25, 0, 200", grade);
        }

        let optical_size = self.optical_size.unwrap_or(24);

        if ![20, 24, 40, 48].contains(&optical_size) {
            anyhow::bail!(
                "Invalid optical_size '{}': must be one of 20, 24, 40, 48",
                optical_size
            );
        }

        if self.icons.is_empty() {
            anyhow::bail!("No icons specified in config");
        }

        let compose_output_dir = self
            .compose_output_dir
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        Ok(ResolvedConfig {
            style,
            fill,
            weight,
            grade,
            optical_size,
            icons: self.icons,
            compose_package: self.compose_package,
            compose_output_dir,
        })
    }
}

/// Read and parse the config file.
pub fn read_config(path: &str) -> Result<Config> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path))?;
    let config = toml::from_str(&content).with_context(|| "Failed to parse config file")?;
    Ok(config)
}
