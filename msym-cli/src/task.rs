use crate::config::Config;
use crate::config::symbol_name::SymbolName;
use crate::kotlin::TransformConfig;
use std::path::PathBuf;

/// A self-contained download job.
#[derive(Clone)]
pub struct Task {
    pub symbol_name: SymbolName,
    pub url: String,
    pub output_path: PathBuf,
    pub transform: TransformConfig,
}

/// Build download tasks from config.
pub fn build_tasks(config: &Config) -> Vec<Task> {
    let output_dir = config.compose_output_dir.clone();
    let package = config.compose_package.clone();
    config
        .symbol_names
        .iter()
        .map(|symbol| {
            let filename = format!("{}.kt", symbol.to_filename());
            Task {
                symbol_name: symbol.clone(),
                url: build_url(config, symbol.to_url_name()),
                output_path: output_dir.join(&filename),
                transform: TransformConfig {
                    package: package.clone(),
                    field_name: symbol.to_url_name(),
                    upper_camel_fields: config.compose_upper_camel_fields,
                    extension_class: config.compose_extension_class.clone(),
                },
            }
        })
        .collect()
}

/// Build the Google Fonts download URL for a given snake_case icon name.
fn build_url(config: &Config, symbol_name: String) -> String {
    let fill_val = if config.fill { 1 } else { 0 };
    format!(
        "https://fonts.gstatic.com/render/v1/Material+Symbols+{style}/{dp}dp/{symbol_name}.kt?var=opsz,wght,FILL,GRAD,ROND@{opsz},{wght},{fill},{grade},50",
        style = config.style,
        dp = u32::from(config.optical_size),
        opsz = u32::from(config.optical_size),
        wght = u32::from(config.weight),
        fill = fill_val,
        grade = i32::from(config.grade),
    )
}
