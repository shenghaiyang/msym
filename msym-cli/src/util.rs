/// Capitalize the first letter of a string (for Kotlin class names).
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Convert an icon name to the Kotlin file name (PascalCase).
pub fn icon_to_filename(icon: &str) -> String {
    let parts: Vec<&str> = icon.split(&['-', '_'][..]).collect();
    parts
        .iter()
        .map(|p| capitalize(p))
        .fold(String::new(), |mut acc, p| {
            acc.push_str(&p);
            acc
        })
}
