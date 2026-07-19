use anyhow::{Context, Result, bail};
use raw_config::RawConfig;
use std::path::PathBuf;
use std::str::FromStr;
use symbol_grade::SymbolGrade;
use symbol_name::SymbolName;
use symbol_optical_size::SymbolOpticalSize;
use symbol_style::SymbolStyle;
use symbol_weight::SymbolWeight;

mod raw_config;
pub mod symbol_grade;
pub mod symbol_name;
pub mod symbol_optical_size;
pub mod symbol_style;
pub mod symbol_weight;

/// Validated configuration with all defaults applied and icon names normalized.
/// Every field is a concrete value — no Options remain.
#[derive(Debug)]
pub struct Config {
    pub style: SymbolStyle,
    pub fill: bool,
    pub weight: SymbolWeight,
    pub grade: SymbolGrade,
    pub optical_size: SymbolOpticalSize,
    pub symbol_names: Vec<SymbolName>,
    pub compose_package: String,
    pub compose_output_dir: PathBuf,
    pub compose_upper_camel_fields: bool,
    pub compose_extension_class: Option<String>,
}

impl Config {
    pub async fn parse_file(path: &str) -> Result<Self> {
        let raw = RawConfig::parse_file(path).await?;
        Self::from_raw(raw)
    }

    fn from_raw(raw: RawConfig) -> Result<Self> {
        let style = raw.style.map_or(Ok(SymbolStyle::Rounded), |s| {
            SymbolStyle::from_str(&s).with_context(|| {
                format!("Invalid style: '{s}', must be one of Outlined, Rounded, Sharp")
            })
        })?;

        let fill = raw.fill.unwrap_or(false);

        let weight = raw.weight.map_or(Ok(SymbolWeight::W400), |w| {
            SymbolWeight::try_from(w).with_context(|| {
                format!("Invalid weight: '{w}', must be one of 100, 200, 300, 400, 500, 600, 700")
            })
        })?;

        let grade = raw.grade.map_or(Ok(SymbolGrade::Normal), |g| {
            SymbolGrade::try_from(g)
                .with_context(|| format!("Invalid grade: '{g}', must be one of -25, 0, 200"))
        })?;

        let optical_size = raw.optical_size.map_or(Ok(SymbolOpticalSize::Dp24), |s| {
            SymbolOpticalSize::try_from(s).with_context(|| {
                format!("Invalid optical_size: '{s}', must be one of 20, 24, 40, 48")
            })
        })?;

        let symbol_names: Vec<SymbolName> = raw
            .icons
            .into_iter()
            .filter(|s| !s.trim().is_empty())
            .map(SymbolName::new)
            .collect();
        if symbol_names.is_empty() {
            bail!("Empty icon names");
        }

        let compose_output_dir = raw
            .compose_output_dir
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        Ok(Config {
            style,
            fill,
            weight,
            grade,
            optical_size,
            symbol_names,
            compose_package: raw.compose_package,
            compose_output_dir,
            compose_upper_camel_fields: raw.compose_upper_camel_fields.unwrap_or(false),
            compose_extension_class: raw.compose_extension_class,
        })
    }
}
