# Changelog

## v0.3.1

- Update Rust crate `clap` to v4.6.3.
- Update Rust crate `tokio` to v1.53.1.

## v0.3.0

- Add `compose_upper_camel_fields` config option to rename icon fields from
  snake_case to UpperCamelCase (e.g. `arrow_back` → `ArrowBack`).
- Add `compose_extension_class` config option to generate icon properties as
  extensions of a given class instead of top-level `val`s. The fully-qualified
  class name is imported as-is and the last segment is used as the receiver
  type (e.g. `compose_extension_class = "com.example.icons.Symbols.Rounded"`
  produces `public val Rounded.Home: ImageVector`).

## v0.2.0

- Refactor internal architecture.
- Pin Rust crate `heck` to `=0.5.0`.
- Update Rust toolchain to v1.97.1.
- Fix badge link in README.
- Add shell installation instructions.

## v0.1.0

Initial release.
