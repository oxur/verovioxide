# Verovioxide

[![CI](https://github.com/oxur/verovioxide/actions/workflows/ci.yml/badge.svg)](https://github.com/oxur/verovioxide/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/verovioxide.svg)](https://crates.io/crates/verovioxide)
[![docs.rs](https://docs.rs/verovioxide/badge.svg)](https://docs.rs/verovioxide)

Safe Rust bindings to the [Verovio](https://www.verovio.org/) music notation engraving library.

## Features

- Render MusicXML, MEI, ABC, Humdrum, and Plaine & Easie to SVG
- Bundled SMuFL fonts (Leipzig, Bravura, Gootville, Leland, Petaluma)
- Type-safe Options API with serde serialization
- No runtime dependencies (statically linked Verovio)
- Safe Rust wrapper over C FFI

## Installation

```bash
cargo add verovioxide
```

## Quick Start

```rust
use verovioxide::{Toolkit, Options, Result};

fn main() -> Result<()> {
    // Create a toolkit with bundled resources
    let mut toolkit = Toolkit::new()?;

    // Load MEI data
    let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
    <mei xmlns="http://www.music-encoding.org/ns/mei">
      <music><body><mdiv><score>
        <scoreDef><staffGrp>
          <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
        </staffGrp></scoreDef>
        <section><measure><staff n="1"><layer n="1">
          <note pname="c" oct="4" dur="4"/>
        </layer></staff></measure></section>
      </score></mdiv></body></music>
    </mei>"#;

    toolkit.load_data(mei)?;

    // Configure rendering options
    let options = Options::builder()
        .scale(100)
        .adjust_page_height(true)
        .build();
    toolkit.set_options(&options)?;

    // Render to SVG
    let svg = toolkit.render_to_svg(1)?;
    println!("Rendered {} bytes of SVG", svg.len());

    Ok(())
}
```

### Loading from Files

```rust
use verovioxide::Toolkit;
use std::path::Path;

let mut toolkit = Toolkit::new()?;
toolkit.load_file(Path::new("score.musicxml"))?;

let svg = toolkit.render_to_svg(1)?;
```

### Configuring Options

```rust
use verovioxide::{Options, BreakMode, HeaderMode};

let options = Options::builder()
    .scale(80)                          // 80% scale
    .page_width(2100)                   // A4 width in MEI units
    .page_height(2970)                  // A4 height
    .adjust_page_height(true)           // Fit content
    .font("Bravura")                    // Use Bravura font
    .breaks(BreakMode::Auto)            // Automatic page breaks
    .header(HeaderMode::None)           // No header
    .build();
```

## Supported Input Formats

| Format | Extensions | Notes |
|--------|------------|-------|
| MusicXML | `.musicxml`, `.xml`, `.mxl` | Standard music interchange format |
| MEI | `.mei` | Music Encoding Initiative XML |
| ABC | `.abc` | Text-based notation format |
| Humdrum | `.krn`, `.hmd` | Kern and other Humdrum formats |
| PAE | - | Plaine & Easie Code (RISM) |

Format detection is automatic based on file content.

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `bundled-data` | Yes | Include bundled SMuFL fonts and resources |
| `font-leipzig` | Yes | Leipzig SMuFL font (default font) |
| `font-bravura` | No | Bravura SMuFL font |
| `font-gootville` | No | Gootville SMuFL font |
| `font-leland` | No | Leland SMuFL font |
| `font-petaluma` | No | Petaluma SMuFL font |
| `all-fonts` | No | Enable all fonts |

Note: Bravura baseline data is always included as it is required for Verovio's glyph name table.

To enable additional fonts:

```toml
[dependencies]
verovioxide = { version = "0.0.1", features = ["font-bravura", "font-leland"] }
```

To disable bundled data and provide your own resource path:

```toml
[dependencies]
verovioxide = { version = "0.0.1", default-features = false }
```

Then use `Toolkit::with_resource_path()`:

```rust
use verovioxide::Toolkit;
use std::path::Path;

let toolkit = Toolkit::with_resource_path(Path::new("/path/to/verovio/data"))?;
```

## Building from Source

Clone with submodules (Verovio is included as a Git submodule):

```bash
git clone --recursive https://github.com/oxur/verovioxide.git
cd verovioxide
```

Build and test:

```bash
cargo build
cargo test
```

Run an example:

```bash
cargo run --example render_abc
cargo run --example render_musicxml -- input.musicxml output.svg
```

## Crate Structure

- **verovioxide** - High-level safe Rust API
- **verovioxide-sys** - Low-level FFI bindings to Verovio C API
- **verovioxide-data** - Bundled SMuFL fonts and resources

## License

This project is licensed under the Apache License 2.0.

**Dependencies:**

- [Verovio](https://www.verovio.org/) is licensed under the LGPL-3.0
- SMuFL fonts have their own licenses (see the respective font directories)

## Acknowledgments

- [Verovio](https://www.verovio.org/) - The music notation engraving library
- [SMuFL](https://www.smufl.org/) - Standard Music Font Layout specification
- The music encoding community for MEI, MusicXML, and other formats
