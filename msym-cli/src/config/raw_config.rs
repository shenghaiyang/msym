use anyhow::Context;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RawConfig {
    pub style: Option<String>,
    pub fill: Option<bool>,
    pub weight: Option<u32>,
    pub grade: Option<i32>,
    pub optical_size: Option<u32>,
    pub icons: Vec<String>,
    pub compose_package: String,
    pub compose_output_dir: Option<String>,
}

impl RawConfig {
    pub async fn parse_file(path: &str) -> anyhow::Result<Self> {
        let content = tokio::fs::read_to_string(&path)
            .await
            .with_context(|| format!("Failed to read config file: {}", path))?;
        let raw: RawConfig =
            toml::from_str(&content).with_context(|| "Failed to parse config file")?;
        Ok(raw)
    }
}
