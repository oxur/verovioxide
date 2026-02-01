# verovioxide-data

[![][crate-badge]][crate]
[![][docs-badge]][docs]

Bundled SMuFL fonts and resources for [verovioxide](https://crates.io/crates/verovioxide).

## Overview

This crate provides the SMuFL music fonts and resource files required by Verovio. It's automatically included as a dependency of `verovioxide` when the `bundled-data` feature is enabled (default).

Most users don't need to use this crate directly.

## Included Fonts

| Font | Feature | Default | Description |
|------|---------|---------|-------------|
| Leipzig | `font-leipzig` | Yes | Default Verovio font, traditional engraving style |
| Bravura | `font-bravura` | No | Reference SMuFL font by Steinberg |
| Gootville | `font-gootville` | No | Handwritten style font |
| Leland | `font-leland` | No | MuseScore's default font |
| Petaluma | `font-petaluma` | No | Handwritten jazz style font |

Note: Bravura baseline data is always included as it's required for Verovio's glyph name table.

## Features

```toml
[dependencies]
# Default: Leipzig font only
verovioxide-data = "0.1"

# Specific fonts
verovioxide-data = { version = "0.1", features = ["font-bravura", "font-leland"] }

# All fonts
verovioxide-data = { version = "0.1", features = ["all-fonts"] }
```

## Usage

```rust
use verovioxide_data::{resource_dir, available_fonts};

// Extract resources to a temporary directory
let dir = resource_dir()?;
println!("Resources at: {}", dir.path().display());

// List available fonts
for font in available_fonts() {
    println!("Font: {}", font);
}
```

## Related Crates

- [`verovioxide`](https://crates.io/crates/verovioxide) - High-level safe Rust API
- [`verovioxide-sys`](https://crates.io/crates/verovioxide-sys) - Raw FFI bindings

## License

This project is licensed under the Apache License 2.0.

SMuFL fonts have their own licenses:
- Leipzig: SIL Open Font License
- Bravura: SIL Open Font License
- Gootville: SIL Open Font License
- Leland: SIL Open Font License
- Petaluma: SIL Open Font License

[crate]: https://crates.io/crates/verovioxide-data
[crate-badge]: https://img.shields.io/crates/v/verovioxide-data.svg
[docs]: https://docs.rs/verovioxide-data/
[docs-badge]: https://img.shields.io/badge/rust-documentation-blue.svg
