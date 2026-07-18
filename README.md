# Msym

[![Crates.io](https://img.shields.io/crates/v/msym.svg)](https://crates.io/crates/msym)
[![CI](https://github.com/shenghaiyang/msym/actions/workflows/ci.yml/badge.svg)](https://github.com/shenghaiyang/msym/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE-APACHE)

A CLI tool for downloading [Material Symbols](https://fonts.google.com/icons) Compose code.

## Installation

### Homebrew

```sh
brew install shenghaiyang/tap/msym-cli
```

### Cargo

```sh
cargo install msym-cli
```

## Quick Start

**Step 1 — Create a config file** (`msym.toml`):

```toml
style = "rounded"
fill = false
weight = 400
grade = 0
optical_size = 24

icons = [
    "home",
    "add",
    "arrow-forward",
]

compose_package = "com.example.icons"
compose_output_dir = "src/main/kotlin/com/example/icons"
```

**Step 2 — Run:**

```bash
msym
```

That's it. Icons are downloaded to `compose_output_dir` as Kotlin files. Only new icons are fetched on subsequent runs;
use `-f` to force re-download.

## Command Reference

```
msym [OPTIONS] [CONFIG]
```

| Option           | Description                                    |
|------------------|------------------------------------------------|
| `-f, --force`    | Re-download all icons, even if already present |
| `-j, --jobs <N>` | Concurrent downloads (1–32) \[default: 4\]     |
| `-v, --verbose`  | Show URLs and retry details                    |
| `-h, --help`     | Print help                                     |

```bash
msym                           # default config (msym.toml), skip existing
msym my-icons.toml             # custom config
msym -j 1 -v                   # single-threaded with progress bar + verbose
msym -f -j 8                   # force, 8 parallel downloads
```

## Configuration Reference

All parameters are optional except `icons`, `compose_package`.

| Parameter            | Default      | Options                                         |
|----------------------|--------------|-------------------------------------------------|
| `style`              | `"rounded"`  | `rounded`, `outlined`, `sharp`                  |
| `fill`               | `false`      | `true`, `false`                                 |
| `weight`             | `400`        | `100`, `200`, `300`, `400`, `500`, `600`, `700` |
| `grade`              | `0`          | `-25`, `0`, `200`                               |
| `optical_size`       | `24`         | `20`, `24`, `40`, `48`                          |
| `icons`              | *(required)* | list of icon names                              |
| `compose_package`    | *(required)* | Kotlin package declaration                      |
| `compose_output_dir` | `.`          | output directory path                           |

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE)
  or [https://www.apache.org/licenses/LICENSE-2.0](https://www.apache.org/licenses/LICENSE-2.0))
- MIT license ([LICENSE-MIT](LICENSE-MIT) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT))

at your option.

