use heck::ToUpperCamelCase;

/// All inputs needed to turn a raw downloaded Kotlin source into the final
/// output file. Grouped so the transform pipeline is self-contained and
/// testable without HTTP I/O.
#[derive(Debug, Clone)]
pub struct TransformConfig {
    pub package: String,
    /// The snake_case icon name as it appears in the downloaded source.
    pub field_name: String,
    pub upper_camel_fields: bool,
    /// Fully-qualified receiver class; the last segment is used in code and
    /// the full value is imported.
    pub extension_class: Option<String>,
}

/// Apply the full Kotlin source transform pipeline in order:
/// package → rename → extension → import.
pub fn transform(source: &str, cfg: &TransformConfig) -> String {
    let mut s = set_package(source, &cfg.package);
    if cfg.upper_camel_fields {
        s = rename_field_upper_camel(&s, &cfg.field_name);
    }
    if let Some(class) = &cfg.extension_class {
        let receiver = class.rsplit('.').next().unwrap_or(class);
        s = set_extension_class(&s, receiver);
        s = add_import(&s, class);
    }
    s
}

/// Replace the package declaration in Kotlin source, or insert one if missing.
pub fn set_package(source: &str, package: &str) -> String {
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

/// Insert an import statement, keeping it within the existing import block and
/// avoiding duplicates. If no import block exists, the import is added right
/// after the package declaration.
pub fn add_import(source: &str, import_path: &str) -> String {
    let import_line = format!("import {}", import_path);
    let lines: Vec<&str> = source.lines().collect();

    // Locate the contiguous import block.
    let mut start = None;
    let mut end = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("import ") {
            if start.is_none() {
                start = Some(i);
            }
            end = Some(i + 1);
        } else if start.is_some() && !trimmed.is_empty() {
            break;
        }
    }

    // Skip if already present.
    if let Some(s) = start {
        for line in lines.iter().take(end.unwrap()).skip(s) {
            if line.trim_start() == import_line {
                return source.to_string();
            }
        }
    }

    let mut new_lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    if start.is_some() {
        new_lines.insert(end.unwrap(), import_line);
    } else if let Some(pkg) = package_line_index(&lines) {
        new_lines.insert(pkg + 1, import_line);
    } else {
        new_lines.insert(0, import_line);
    }

    new_lines.join("\n") + "\n"
}

fn package_line_index(lines: &[&str]) -> Option<usize> {
    lines.iter().position(|line| line.trim_start().starts_with("package "))
}

/// Turn the top-level icon property into an extension property of `receiver`.
///
/// `public val home: ImageVector` becomes
/// `public val Rounded.home: ImageVector`. `receiver` is the receiver type as
/// used in code (without a trailing dot). The backing field (`private var _home`)
/// is left untouched — extension properties cannot have backing fields, so the
/// top-level private cache remains.
pub fn set_extension_class(source: &str, receiver: &str) -> String {
    const MARKER: &str = "public val ";
    let pos = match source.find(MARKER) {
        Some(p) => p + MARKER.len(),
        None => return source.to_string(),
    };
    let (head, tail) = source.split_at(pos);
    format!("{}{}.{}", head, receiver, tail)
}

/// Rename the icon property and its backing field from snake_case to UpperCamelCase.
///
/// `snake_name` is the field name as it appears in the downloaded source
/// (e.g. `arrow_back`). The backing field `_{snake_name}` is renamed first so
/// its inner `{snake_name}` substring is not touched by the subsequent pass.
pub fn rename_field_upper_camel(source: &str, snake_name: &str) -> String {
    let pascal = snake_name.to_upper_camel_case();
    if pascal == snake_name {
        return source.to_string();
    }

    let backing = format!("_{}", snake_name);
    let backing_pascal = format!("_{}", pascal);
    let after_backing = replace_ident(source, &backing, &backing_pascal);
    replace_ident(&after_backing, snake_name, &pascal)
}

/// Replace every occurrence of `needle` that forms a complete Kotlin identifier
/// (i.e. not adjacent to other identifier characters on either side).
fn replace_ident(source: &str, needle: &str, replacement: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let needle_chars: Vec<char> = needle.chars().collect();
    let n = needle_chars.len();
    let mut out = String::with_capacity(source.len());
    let mut i = 0;
    while i < chars.len() {
        if i + n <= chars.len() && chars[i..i + n] == needle_chars[..] {
            let prev = if i == 0 { '\0' } else { chars[i - 1] };
            let next = if i + n >= chars.len() { '\0' } else { chars[i + n] };
            if !is_ident_char(prev) && !is_ident_char(next) {
                out.push_str(replacement);
                i += n;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn is_ident_char(c: char) -> bool {
    c == '_' || c.is_ascii_alphanumeric()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_word() {
        let src = "public val home: ImageVector\nprivate var _home: ImageVector? = null\nname = \"home\"";
        let out = rename_field_upper_camel(src, "home");
        assert!(out.contains("public val Home: ImageVector"));
        assert!(out.contains("private var _Home: ImageVector? = null"));
        assert!(out.contains("name = \"Home\""));
    }

    #[test]
    fn multi_word() {
        let src = "public val arrow_back: ImageVector\nif (_arrow_back != null) return _arrow_back!!\nname = \"arrow_back\"";
        let out = rename_field_upper_camel(src, "arrow_back");
        assert!(out.contains("public val ArrowBack: ImageVector"));
        assert!(out.contains("if (_ArrowBack != null) return _ArrowBack!!"));
        assert!(out.contains("name = \"ArrowBack\""));
        // The inner `arrow_back` of `_arrow_back` must not be replaced separately.
        assert!(!out.contains("_arrow_back"));
        assert!(!out.contains("_ArrowBack_arrow"));
    }

    #[test]
    fn does_not_touch_substrings() {
        // `homeless` and `homepage` must not be affected.
        let src = "public val home: ImageVector\nval homeless = 1\nval homepage = 2";
        let out = rename_field_upper_camel(src, "home");
        assert!(out.contains("public val Home: ImageVector"));
        assert!(out.contains("val homeless = 1"));
        assert!(out.contains("val homepage = 2"));
    }

    #[test]
    fn already_pascal_is_noop() {
        let src = "public val Home: ImageVector";
        let out = rename_field_upper_camel(src, "Home");
        assert_eq!(out, src);
    }

    #[test]
    fn extension_class_inserts_receiver() {
        let src = "@Suppress(\"CheckReturnValue\")\npublic val home: ImageVector\n  get() = _home!!\n\nprivate var _home: ImageVector? = null";
        let out = set_extension_class(src, "Rounded");
        assert!(out.contains("public val Rounded.home: ImageVector"));
        // backing field untouched
        assert!(out.contains("private var _home: ImageVector? = null"));
    }

    #[test]
    fn extension_class_after_upper_camel() {
        let src = "public val arrow_back: ImageVector\nprivate var _arrow_back: ImageVector? = null";
        let renamed = rename_field_upper_camel(src, "arrow_back");
        let out = set_extension_class(&renamed, "Rounded");
        assert!(out.contains("public val Rounded.ArrowBack: ImageVector"));
        assert!(out.contains("private var _ArrowBack: ImageVector? = null"));
    }

    #[test]
    fn extension_class_single_segment() {
        let src = "public val home: ImageVector";
        let out = set_extension_class(src, "Symbols");
        assert!(out.contains("public val Symbols.home: ImageVector"));
    }

    #[test]
    fn add_import_appends_to_block() {
        let src = "package com.example\n\nimport androidx.compose.ui.graphics.Color\nimport androidx.compose.ui.unit.dp\n\npublic val home: ImageVector";
        let out = add_import(src, "com.example.icons.Symbols.Rounded");
        let lines: Vec<&str> = out.lines().collect();
        let import_idx = lines.iter().position(|l| *l == "import com.example.icons.Symbols.Rounded").unwrap();
        // stays within the import block, before the blank line / body
        assert!(import_idx < lines.iter().rposition(|l| l.starts_with("import ")).unwrap()
            || lines[import_idx + 1].is_empty());
        assert!(out.contains("import androidx.compose.ui.unit.dp"));
    }

    #[test]
    fn add_import_dedupes() {
        let src = "package com.example\n\nimport com.example.icons.Symbols.Rounded\n\npublic val home: ImageVector";
        let out = add_import(src, "com.example.icons.Symbols.Rounded");
        assert_eq!(out, src);
    }

    #[test]
    fn add_import_without_block_after_package() {
        let src = "package com.example\n\npublic val home: ImageVector";
        let out = add_import(src, "com.example.icons.Symbols.Rounded");
        assert!(out.contains("package com.example\nimport com.example.icons.Symbols.Rounded"));
    }

    #[test]
    fn transform_full_pipeline() {
        let src = "package com.google.fonts\n\npublic val arrow_back: ImageVector\n  get() = _arrow_back!!\n\nprivate var _arrow_back: ImageVector? = null";
        let cfg = TransformConfig {
            package: "com.example.icons".to_string(),
            field_name: "arrow_back".to_string(),
            upper_camel_fields: true,
            extension_class: Some("com.example.icons.Symbols.Rounded".to_string()),
        };
        let out = transform(src, &cfg);
        assert!(out.contains("package com.example.icons"));
        assert!(out.contains("import com.example.icons.Symbols.Rounded"));
        assert!(out.contains("public val Rounded.ArrowBack: ImageVector"));
        assert!(out.contains("private var _ArrowBack: ImageVector? = null"));
    }

    #[test]
    fn transform_package_only() {
        let src = "package com.google.fonts\n\npublic val home: ImageVector";
        let cfg = TransformConfig {
            package: "com.example.icons".to_string(),
            field_name: "home".to_string(),
            upper_camel_fields: false,
            extension_class: None,
        };
        let out = transform(src, &cfg);
        assert!(out.contains("package com.example.icons"));
        assert!(!out.contains("import com.example"));
        assert!(out.contains("public val home: ImageVector"));
    }
}
