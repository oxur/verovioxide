---
number: 4
title: "PNG Rendering Support"
author: "default
bundled"
component: All
tags: [change-me]
created: 2026-02-02
updated: 2026-02-02
state: Active
supersedes: null
superseded-by: null
version: 1.0
---

# PNG Rendering Support

## Overview

Add PNG rendering support using the resvg library, mirroring the existing SVG API pattern. This enables in-memory PNG bytes (for viuer terminal rendering) and file output.

## API Design

```rust
use verovioxide::Png;

// Render single page to PNG bytes
let png_bytes: Vec<u8> = voxide.render(Png::page(1))?;

// With options (chainable)
let png_bytes = voxide.render(
    Png::page(1)
        .width(800)           // Scale to 800px width
        .scale(2.0)           // Or use zoom factor
        .white_background()   // White instead of transparent
)?;

// Render all pages
let all_pngs: Vec<Vec<u8>> = voxide.render(Png::all_pages())?;

// Render page range
let pngs: Vec<Vec<u8>> = voxide.render(Png::pages(2, 5))?;

// File rendering
voxide.render_to("output.png")?;                    // Single page
voxide.render_to_as("output.png", Png::all_pages())?;  // Creates output/ directory

// viuer integration (user code)
let img = image::load_from_memory(&png_bytes)?;
viuer::print(&img, &Config::default())?;
```

## Type Definitions

| Type | Description | Output Type |
|------|-------------|-------------|
| `Png` | Namespace with static methods | - |
| `PngPage` | Single page spec | `Vec<u8>` |
| `PngPages` | Page range spec | `Vec<Vec<u8>>` |
| `PngAllPages` | All pages spec | `Vec<Vec<u8>>` |
| `PngOptions` | Width/height/scale/background | - |

## PNG Options

| Method | Description |
|--------|-------------|
| `.width(u32)` | Target width (scales proportionally) |
| `.height(u32)` | Target height (scales proportionally) |
| `.scale(f32)` | Zoom factor (1.0 = original) |
| `.background(r, g, b, a)` | RGBA background color |
| `.white_background()` | Convenience for opaque white |

## Files to Modify

| File | Changes |
|------|---------|
| `crates/verovioxide/Cargo.toml` | Add resvg, usvg, tiny-skia (optional `png` feature) |
| `crates/verovioxide/src/render.rs` | Add PNG types, traits, `svg_to_png` helper |
| `crates/verovioxide/src/lib.rs` | Feature-gated exports for PNG types |
| `crates/verovioxide/tests/integration_test.rs` | Add PNG integration tests |
| `README.md` | Document PNG feature |
| `workbench/RELEASE-NOTES-0.3.0.md` | Add PNG to release notes |

## Implementation Steps

### Step 1: Add dependencies to Cargo.toml

```toml
[features]
default = ["bundled-data", "png"]  # PNG enabled by default
bundled-data = ["verovioxide-data", "tempfile"]
png = ["resvg", "usvg", "tiny-skia"]

[dependencies]
resvg = { version = "0.43", optional = true }
usvg = { version = "0.43", optional = true }
tiny-skia = { version = "0.11", optional = true }

[dev-dependencies]
image = "0.25"  # For viuer compatibility tests
```

### Step 2: Add PNG types to render.rs

Add after SVG section:

- `Png` namespace struct
- `PngOptions` struct with fields: width, height, scale, background
- `PngPage`, `PngPages`, `PngAllPages` structs
- Chainable option methods on each
- `RenderOutput` and `RenderSpec` trait implementations

### Step 3: Add helper functions

```rust
/// Convert SVG string to PNG bytes
fn svg_to_png(svg: &str, options: &PngOptions) -> Result<Vec<u8>> {
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default())?;
    let (width, height, scale_x, scale_y) = calculate_png_dimensions(&tree, options);
    let mut pixmap = tiny_skia::Pixmap::new(width, height)?;
    if let Some(bg) = options.background {
        pixmap.fill(bg);
    }
    let transform = tiny_skia::Transform::from_scale(scale_x, scale_y);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    pixmap.encode_png()
}

/// Calculate target dimensions based on options
fn calculate_png_dimensions(...) -> (u32, u32, f32, f32) { ... }
```

### Step 4: Update format inference

Add `.png` extension handling to `infer_format_and_render()`:

```rust
#[cfg(feature = "png")]
Some("png") => PngPage { page: 1, options: PngOptions::default() }
    .render_to_file(toolkit, path),
```

### Step 5: Update lib.rs exports

```rust
#[cfg(feature = "png")]
pub use render::{Png, PngAllPages, PngOptions, PngPage, PngPages};
```

### Step 6: Add unit tests (in render.rs)

Feature-gated tests for:

- Builder patterns
- Dimension calculations
- Debug/Clone impls
- Send/Sync markers

### Step 7: Add integration tests

Feature-gated tests for:

- `render(Png::page(1))` returns valid PNG bytes
- PNG magic bytes verification: `[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]`
- Options (width, scale, background)
- All pages rendering
- File rendering creates correct files
- viuer compatibility via `image::load_from_memory()`

### Step 8: Update documentation

- README: Add PNG section with feature flag and examples
- Release notes: Add PNG rendering to 0.3.0 highlights

## Verification

1. `cargo build --features png` - Compilation with PNG
2. `cargo build --no-default-features --features bundled-data` - Without PNG
3. `cargo test --features png` - All tests pass
4. `cargo clippy --features png` - No warnings
5. Verify PNG magic bytes in test output
6. Test viuer compatibility with `image::load_from_memory()`

## Key Design Decisions

1. **Optional feature**: PNG is opt-in via `png` feature (enabled by default)
2. **Same file**: PNG types go in render.rs with other formats
3. **Chainable options**: Methods like `.width(800).scale(2.0)` on spec types
4. **Vec<u8> output**: Raw PNG bytes for viuer compatibility
5. **SVG-first**: Render to SVG internally, then convert to PNG
