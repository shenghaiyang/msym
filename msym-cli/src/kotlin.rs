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
