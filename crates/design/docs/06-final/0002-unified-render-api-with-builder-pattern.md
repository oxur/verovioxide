---
number: 2
title: "Unified Render API with Builder Pattern"
author: "Duncan McGreggor"
component: All
tags: [change-me]
created: 2026-02-02
updated: 2026-02-02
state: Final
supersedes: null
superseded-by: null
version: 1.0
---

# Unified Render API with Builder Pattern

## Overview

Add unified `render()` and `render_to()` methods that use consistent builder patterns for all output formats, replacing the current fragmented API.

## API Design

### In-Memory Rendering: `render()`

```rust
// SVG
let svg = voxide.render(Svg::page(1))?;                    // String
let svg = voxide.render(Svg::page(3).with_declaration())?; // String
let pages = voxide.render(Svg::all_pages())?;              // Vec<String>
let pages = voxide.render(Svg::pages(2, 5))?;              // Vec<String>

// Other formats
let midi = voxide.render(Midi)?;                           // String (base64)
let pae = voxide.render(Pae)?;                             // String
let timemap = voxide.render(Timemap)?;                     // String (JSON)
let timemap = voxide.render(Timemap::with_options()
    .include_measures(true)
    .include_rests(false))?;                               // String (JSON)
let expansion = voxide.render(ExpansionMap)?;              // String (JSON)
let mei = voxide.render(Mei)?;                             // String
let mei = voxide.render(Mei::with_options()
    .remove_ids(true))?;                                   // String
let humdrum = voxide.render(Humdrum)?;                     // String
```

### File Rendering: `render_to()` and `render_to_as()`

```rust
// Simple (infer format from extension, use defaults)
voxide.render_to("output.svg")?;                           // page 1
voxide.render_to("output.mid")?;
voxide.render_to("output.mei")?;

// With explicit format
voxide.render_to_as("output.svg", Svg::page(3))?;
voxide.render_to_as("output.svg", Svg::all_pages())?;      // creates output/ directory
voxide.render_to_as("output.json", Timemap)?;              // disambiguate .json
voxide.render_to_as("output.json", Timemap::with_options().include_measures(true))?;
voxide.render_to_as("score.mei", Mei::with_options().remove_ids(true))?;
```

## Type Definitions

### Format Builders

```rust
// SVG format with page selection
pub struct Svg;
pub struct SvgPage { page: u32, declaration: bool }
pub struct SvgPages { start: u32, end: u32, declaration: bool }
pub struct SvgAllPages { declaration: bool }

impl Svg {
    pub fn page(n: u32) -> SvgPage;
    pub fn pages(start: u32, end: u32) -> SvgPages;
    pub fn all_pages() -> SvgAllPages;
}

impl SvgPage {
    pub fn with_declaration(self) -> Self;
}

// Simple formats (unit structs)
pub struct Midi;
pub struct Pae;
pub struct ExpansionMap;
pub struct Humdrum;

// Timemap with typed options
pub struct Timemap;
pub struct TimemapWithOptions { include_measures: bool, include_rests: bool }

impl Timemap {
    pub fn with_options() -> TimemapOptionsBuilder;
}

pub struct TimemapOptionsBuilder { ... }
impl TimemapOptionsBuilder {
    pub fn include_measures(self, v: bool) -> Self;
    pub fn include_rests(self, v: bool) -> Self;
    pub fn build(self) -> TimemapWithOptions;
}

// MEI with typed options
pub struct Mei;
pub struct MeiWithOptions { remove_ids: bool, ... }

impl Mei {
    pub fn with_options() -> MeiOptionsBuilder;
}

pub struct MeiOptionsBuilder { ... }
impl MeiOptionsBuilder {
    pub fn remove_ids(self, v: bool) -> Self;
    pub fn build(self) -> MeiWithOptions;
}
```

### Traits

```rust
/// Trait for in-memory rendering with associated output type
pub trait RenderOutput {
    type Output;
    fn render(self, toolkit: &Toolkit) -> Result<Self::Output>;
}

// Implementations
impl RenderOutput for SvgPage { type Output = String; ... }
impl RenderOutput for SvgAllPages { type Output = Vec<String>; ... }
impl RenderOutput for SvgPages { type Output = Vec<String>; ... }
impl RenderOutput for Midi { type Output = String; ... }
impl RenderOutput for Timemap { type Output = String; ... }
impl RenderOutput for TimemapWithOptions { type Output = String; ... }
// etc.

/// Trait for format specification in render_to_as()
pub trait RenderSpec {
    fn render_to_file(self, toolkit: &Toolkit, path: &Path) -> Result<()>;
}
```

### Extension Mapping

| Extension | Inferred Format |
|-----------|-----------------|
| `.svg` | `Svg::page(1)` |
| `.mid`, `.midi` | `Midi` |
| `.pae` | `Pae` |
| `.mei` | `Mei` |
| `.krn`, `.hmd` | `Humdrum` |
| `.json` | Error: ambiguous, requires explicit format |

### Multi-Page File Output

When `Svg::all_pages()` or `Svg::pages(start, end)` is used with `render_to()`:

1. Create directory: `<filename_without_extension>/`
2. Write files: `page-001.svg`, `page-002.svg`, etc.

```rust
// input.svg -> input/page-001.svg, input/page-002.svg, ...
voxide.render_to(("output.svg", Svg::all_pages()))?;

// With page range
voxide.render_to(("output.svg", Svg::pages(2, 5)))?;
// Creates: output/page-002.svg, output/page-003.svg, output/page-004.svg, output/page-005.svg
```

## Files to Modify

| File | Changes |
|------|---------|
| `crates/verovioxide/src/render.rs` | **New file**: Format types, builders, traits |
| `crates/verovioxide/src/toolkit.rs` | Add `render()` and `render_to()` methods |
| `crates/verovioxide/src/lib.rs` | Add `mod render`, export types |
| `crates/verovioxide/tests/integration_test.rs` | Add comprehensive tests |
| `README.md` | Update rendering examples |

## Implementation Steps

### Step 1: Create render.rs with format types

- `Svg`, `SvgPage`, `SvgPages`, `SvgAllPages` with builders
- `Midi`, `Pae`, `ExpansionMap`, `Humdrum` unit structs
- `Timemap`, `TimemapOptionsBuilder`, `TimemapWithOptions`
- `Mei`, `MeiOptionsBuilder`, `MeiWithOptions`

### Step 2: Implement RenderOutput trait

- Define trait with associated `Output` type
- Implement for each format type
- Each impl calls appropriate existing toolkit method

### Step 3: Implement RenderSpec trait and file helpers

- Define `RenderSpec` trait for file output
- Implement for each format type
- Add `infer_format(path) -> Box<dyn RenderSpec>` helper
- Add extension inference logic
- Add multi-page directory creation helper

### Step 4: Add Toolkit methods

```rust
impl Toolkit {
    /// In-memory rendering with format-specific output type
    pub fn render<R: RenderOutput>(&self, format: R) -> Result<R::Output> {
        format.render(self)
    }

    /// File rendering with format inferred from extension
    pub fn render_to(&self, path: impl AsRef<Path>) -> Result<()> {
        let format = infer_format(path.as_ref())?;
        format.render_to_file(self, path.as_ref())
    }

    /// File rendering with explicit format
    pub fn render_to_as<F: RenderSpec>(&self, path: impl AsRef<Path>, format: F) -> Result<()> {
        format.render_to_file(self, path.as_ref())
    }
}
```

### Step 5: Add tests

- Test each format type's builder methods
- Test in-memory `render()` for all formats
- Test `render_to()` with extension inference
- Test `render_to_as()` with explicit formats
- Test multi-page output (directory creation)
- Test error cases (ambiguous .json, invalid page)

### Step 6: Update lib.rs exports

```rust
pub use render::{
    Svg, SvgPage, SvgPages, SvgAllPages,
    Midi, Pae, ExpansionMap, Humdrum,
    Timemap, TimemapWithOptions, TimemapOptionsBuilder,
    Mei, MeiWithOptions, MeiOptionsBuilder,
    RenderOutput, RenderTarget,
};
```

### Step 7: Update README

- Replace old render examples with new unified API
- Show both simple and advanced usage patterns

## Verification

1. `cargo build` - Compilation
2. `cargo test` - All tests pass
3. `cargo clippy` - No warnings
4. `cargo tarpaulin` - Maintain 95%+ coverage
5. Manual test: render a MusicXML file to SVG, MIDI, MEI
