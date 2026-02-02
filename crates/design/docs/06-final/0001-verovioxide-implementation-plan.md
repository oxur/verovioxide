---
number: 1
title: "`verovioxide` Implementation Plan"
author: "RISM Digital"
component: All
tags: [change-me]
created: 2026-01-31
updated: 2026-02-02
state: Final
supersedes: null
superseded-by: null
version: 1.0
---

# `verovioxide` Implementation Plan

## Project Overview

Create Rust bindings for the Verovio music notation engraving library, enabling Rust applications to render MusicXML, MEI, ABC, and Humdrum notation to SVG.

**Repository:** `oxur/verovioxide` (GitHub and Codeberg)
**License:** MIT OR Apache-2.0 (dual-licensed)

---

## Directory Structure

```
verovioxide/
├── .github/
│   └── workflows/
│       └── ci.yml
├── crates/
│   ├── verovioxide/           # High-level safe Rust API
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── toolkit.rs     # Safe wrapper around Toolkit
│   │   │   ├── options.rs     # Typed options with serde
│   │   │   ├── error.rs       # Error types
│   │   │   └── input.rs       # Input format helpers
│   │   └── examples/
│   │       ├── render_musicxml.rs
│   │       ├── render_mei.rs
│   │       └── render_abc.rs
│   │
│   ├── verovioxide-sys/       # Raw FFI bindings
│   │   ├── Cargo.toml
│   │   ├── build.rs           # Compiles verovio, handles source fetching
│   │   ├── src/
│   │   │   ├── lib.rs         # Raw unsafe bindings
│   │   │   └── bindings.rs    # Generated or hand-written FFI
│   │   └── wrapper.h          # C header for bindgen (if used)
│   │
│   └── verovioxide-data/      # Bundled fonts and resources
│       ├── Cargo.toml
│       ├── build.rs           # Copies/embeds font data
│       ├── src/
│       │   └── lib.rs         # Resource path helpers
│       └── data/              # Extracted from verovio/data
│           ├── Leipzig/
│           ├── Bravura/
│           ├── Gootville/
│           ├── Leland/
│           ├── Petaluma/
│           └── text/
│
├── verovio/                   # Git submodule (optional for dev)
├── test-fixtures/             # Test MusicXML, MEI, ABC files
│   ├── musicxml/
│   ├── mei/
│   └── abc/
├── Cargo.toml                 # Workspace manifest
├── LICENSE-MIT
├── LICENSE-APACHE
├── README.md
└── NOTICE                     # Attribution for verovio (LGPL) and fonts (OFL)
```

---

## Implementation Steps

### Phase 1: Project Scaffolding

1. **Create workspace `Cargo.toml`**

   ```toml
   [workspace]
   resolver = "2"
   members = ["crates/*"]

   [workspace.package]
   version = "0.1.0"
   edition = "2024"
   license = "MIT OR Apache-2.0"
   repository = "https://github.com/oxur/verovioxide"
   authors = ["oxur contributors"]
   ```

2. **Create license files**
   - `LICENSE-MIT`
   - `LICENSE-APACHE`
   - `NOTICE` file with attribution:

     ```
     This project includes code from:

     Verovio - Music notation engraving library
     Copyright (c) RISM Digital
     Licensed under LGPL-2.1-or-later
     https://github.com/rism-digital/verovio

     SMuFL Fonts (Leipzig, Bravura, Gootville, Leland, Petaluma)
     Licensed under SIL Open Font License 1.1
     ```

---

### Phase 2: `verovioxide-data` Crate

1. **Create `crates/verovioxide-data/Cargo.toml`**

   ```toml
   [package]
   name = "verovioxide-data"
   version.workspace = true
   edition.workspace = true
   license.workspace = true
   description = "Bundled SMuFL fonts and resources for verovioxide"

   [features]
   default = ["font-leipzig"]
   font-leipzig = []
   font-bravura = []
   font-gootville = []
   font-leland = []
   font-petaluma = []
   all-fonts = ["font-leipzig", "font-bravura", "font-gootville", "font-leland", "font-petaluma"]

   [dependencies]
   include_dir = "0.7"
   tempfile = "3"
   thiserror = "2"

   [build-dependencies]
   # None needed if we use include_dir! macro
   ```

2. **Create `crates/verovioxide-data/src/lib.rs`**
   - Define `extract_resources() -> Result<PathBuf, DataError>`
   - Define `resource_dir() -> &'static Dir` for in-memory access
   - Feature-gated font inclusion

3. **Copy font data from `verovio/data/`**
   - Only the processed font data (not source `.sfd` files)
   - Include metadata JSON files

---

### Phase 3: `verovioxide-sys` Crate

1. **Create `crates/verovioxide-sys/Cargo.toml`**

   ```toml
   [package]
   name = "verovioxide-sys"
   version.workspace = true
   edition.workspace = true
   license.workspace = true
   description = "Raw FFI bindings to the Verovio music engraving library"
   links = "verovio"

   [features]
   default = ["bundled"]
   bundled = []           # Compile verovio from source
   system = []            # Link to system libverovio
   humdrum = []           # Include Humdrum support (larger binary)

   [build-dependencies]
   cc = "1"
   cmake = "0.1"
   pkg-config = "0.3"
   reqwest = { version = "0.12", features = ["blocking"], optional = true }
   flate2 = { version = "1", optional = true }
   tar = { version = "0.4", optional = true }
   ```

2. **Create `crates/verovioxide-sys/build.rs`**
   - Implement `find_verovio_source()`:
     - Check `VEROVIO_SOURCE_DIR` env var
     - Check for `../../verovio` submodule
     - Download release tarball if neither exists
   - Compile verovio as static library using `cmake` crate
   - Generate/emit linker flags

3. **Create `crates/verovioxide-sys/src/lib.rs`**
   - Define raw FFI functions matching `c_wrapper.h`:

     ```rust
     extern "C" {
         pub fn vrvToolkit_constructor() -> *mut c_void;
         pub fn vrvToolkit_constructorResourcePath(path: *const c_char) -> *mut c_void;
         pub fn vrvToolkit_destructor(tk: *mut c_void);
         pub fn vrvToolkit_loadData(tk: *mut c_void, data: *const c_char) -> bool;
         pub fn vrvToolkit_renderToSVG(tk: *mut c_void, page: c_int, options: *const c_char) -> *const c_char;
         pub fn vrvToolkit_getPageCount(tk: *mut c_void) -> c_int;
         pub fn vrvToolkit_setOptions(tk: *mut c_void, options: *const c_char) -> bool;
         pub fn vrvToolkit_getOptions(tk: *mut c_void) -> *const c_char;
         pub fn vrvToolkit_getVersion(tk: *mut c_void) -> *const c_char;
         pub fn vrvToolkit_getMEI(tk: *mut c_void, options: *const c_char) -> *const c_char;
         pub fn vrvToolkit_renderToMIDI(tk: *mut c_void) -> *const c_char;
         pub fn vrvToolkit_getLog(tk: *mut c_void) -> *const c_char;
         // ... additional functions as needed
     }
     ```

---

### Phase 4: `verovioxide` Crate (Safe API)

1. **Create `crates/verovioxide/Cargo.toml`**

   ```toml
   [package]
   name = "verovioxide"
   version.workspace = true
   edition.workspace = true
   license.workspace = true
   description = "Safe Rust bindings to the Verovio music notation engraving library"

   [features]
   default = ["bundled-data"]
   bundled-data = ["verovioxide-data"]

   [dependencies]
   verovioxide-sys = { path = "../verovioxide-sys" }
   verovioxide-data = { path = "../verovioxide-data", optional = true }
   thiserror = "2"
   serde = { version = "1", features = ["derive"] }
   serde_json = "1"
   ```

2. **Create `crates/verovioxide/src/error.rs`**

   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum Error {
       #[error("Failed to initialize toolkit: {0}")]
       InitializationError(String),
       #[error("Failed to load data: {0}")]
       LoadError(String),
       #[error("Failed to render: {0}")]
       RenderError(String),
       #[error("Invalid options: {0}")]
       OptionsError(String),
       #[error("Resource error: {0}")]
       ResourceError(#[from] verovioxide_data::DataError),
       #[error("IO error: {0}")]
       IoError(#[from] std::io::Error),
   }
   ```

3. **Create `crates/verovioxide/src/options.rs`**

   ```rust
   #[derive(Debug, Clone, Default, serde::Serialize)]
   #[serde(rename_all = "camelCase")]
   pub struct Options {
       #[serde(skip_serializing_if = "Option::is_none")]
       pub scale: Option<u32>,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub page_width: Option<u32>,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub page_height: Option<u32>,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub adjust_page_height: Option<bool>,
       #[serde(skip_serializing_if = "Option::is_none")]
       pub font: Option<String>,
       // ... more options from Verovio documentation
   }

   impl Options {
       pub fn new() -> Self { Self::default() }
       pub fn scale(mut self, scale: u32) -> Self { self.scale = Some(scale); self }
       pub fn page_width(mut self, width: u32) -> Self { self.page_width = Some(width); self }
       pub fn page_height(mut self, height: u32) -> Self { self.page_height = Some(height); self }
       pub fn adjust_page_height(mut self, adjust: bool) -> Self { self.adjust_page_height = Some(adjust); self }
       pub fn font(mut self, font: impl Into<String>) -> Self { self.font = Some(font.into()); self }
       // ... additional builder methods
   }
   ```

4. **Create `crates/verovioxide/src/toolkit.rs`**

   ```rust
   use std::ffi::{CStr, CString};
   use std::os::raw::c_void;
   use std::path::Path;
   use crate::{Error, Options};
   use verovioxide_sys::*;

   pub struct Toolkit {
       ptr: *mut c_void,
       _resource_dir: Option<tempfile::TempDir>, // Keep alive if extracted
   }

   impl Toolkit {
       /// Create toolkit with bundled resources (requires bundled-data feature)
       #[cfg(feature = "bundled-data")]
       pub fn new() -> Result<Self, Error> {
           let resource_dir = verovioxide_data::extract_resources()?;
           Self::with_resource_path(resource_dir.path())
               .map(|mut tk| { tk._resource_dir = Some(resource_dir); tk })
       }

       /// Create toolkit with explicit resource path
       pub fn with_resource_path(path: impl AsRef<Path>) -> Result<Self, Error> {
           let path_str = path.as_ref().to_str()
               .ok_or_else(|| Error::InitializationError("Invalid path".into()))?;
           let path_cstr = CString::new(path_str)
               .map_err(|e| Error::InitializationError(e.to_string()))?;

           let ptr = unsafe { vrvToolkit_constructorResourcePath(path_cstr.as_ptr()) };
           if ptr.is_null() {
               return Err(Error::InitializationError("Failed to create toolkit".into()));
           }
           Ok(Self { ptr, _resource_dir: None })
       }

       /// Load music notation data (MusicXML, MEI, ABC, Humdrum)
       pub fn load_data(&mut self, data: &str) -> Result<(), Error> {
           let data_cstr = CString::new(data)
               .map_err(|e| Error::LoadError(e.to_string()))?;

           let success = unsafe { vrvToolkit_loadData(self.ptr, data_cstr.as_ptr()) };
           if !success {
               let log = self.get_log();
               return Err(Error::LoadError(log));
           }
           Ok(())
       }

       /// Render a page to SVG
       pub fn render_to_svg(&self, page: u32) -> Result<String, Error> {
           let options_json = CString::new("{}").unwrap();
           let svg_ptr = unsafe {
               vrvToolkit_renderToSVG(self.ptr, page as i32, options_json.as_ptr())
           };

           if svg_ptr.is_null() {
               return Err(Error::RenderError("Render returned null".into()));
           }

           let svg = unsafe { CStr::from_ptr(svg_ptr) }
               .to_string_lossy()
               .into_owned();

           Ok(svg)
       }

       /// Get the number of pages in the loaded document
       pub fn page_count(&self) -> u32 {
           unsafe { vrvToolkit_getPageCount(self.ptr) as u32 }
       }

       /// Set rendering options
       pub fn set_options(&mut self, options: &Options) -> Result<(), Error> {
           let json = serde_json::to_string(options)
               .map_err(|e| Error::OptionsError(e.to_string()))?;
           let json_cstr = CString::new(json)
               .map_err(|e| Error::OptionsError(e.to_string()))?;

           let success = unsafe { vrvToolkit_setOptions(self.ptr, json_cstr.as_ptr()) };
           if !success {
               return Err(Error::OptionsError("Failed to set options".into()));
           }
           Ok(())
       }

       /// Get the Verovio version string
       pub fn version(&self) -> String {
           let version_ptr = unsafe { vrvToolkit_getVersion(self.ptr) };
           if version_ptr.is_null() {
               return String::new();
           }
           unsafe { CStr::from_ptr(version_ptr) }
               .to_string_lossy()
               .into_owned()
       }

       /// Get the log output from the last operation
       pub fn get_log(&self) -> String {
           let log_ptr = unsafe { vrvToolkit_getLog(self.ptr) };
           if log_ptr.is_null() {
               return String::new();
           }
           unsafe { CStr::from_ptr(log_ptr) }
               .to_string_lossy()
               .into_owned()
       }

       /// Export the loaded data as MEI
       pub fn get_mei(&self) -> Result<String, Error> {
           let options_json = CString::new("{}").unwrap();
           let mei_ptr = unsafe { vrvToolkit_getMEI(self.ptr, options_json.as_ptr()) };

           if mei_ptr.is_null() {
               return Err(Error::RenderError("getMEI returned null".into()));
           }

           let mei = unsafe { CStr::from_ptr(mei_ptr) }
               .to_string_lossy()
               .into_owned();

           Ok(mei)
       }
   }

   impl Drop for Toolkit {
       fn drop(&mut self) {
           unsafe { vrvToolkit_destructor(self.ptr); }
       }
   }

   // Safety: Toolkit can be sent between threads, but not shared
   // Each Toolkit instance has its own internal state
   unsafe impl Send for Toolkit {}
   ```

5. **Create `crates/verovioxide/src/lib.rs`**

   ```rust
   //! Safe Rust bindings to the Verovio music notation engraving library.
   //!
   //! # Example
   //!
   //! ```no_run
   //! use verovioxide::{Toolkit, Options};
   //!
   //! fn main() -> verovioxide::Result<()> {
   //!     let mut toolkit = Toolkit::new()?;
   //!
   //!     let options = Options::new()
   //!         .scale(40)
   //!         .adjust_page_height(true);
   //!     toolkit.set_options(&options)?;
   //!
   //!     let musicxml = std::fs::read_to_string("score.musicxml")?;
   //!     toolkit.load_data(&musicxml)?;
   //!
   //!     let svg = toolkit.render_to_svg(1)?;
   //!     std::fs::write("output.svg", &svg)?;
   //!
   //!     Ok(())
   //! }
   //! ```

   pub mod error;
   pub mod options;
   pub mod toolkit;

   pub use error::Error;
   pub use options::Options;
   pub use toolkit::Toolkit;

   /// Convenience Result type for verovioxide operations
   pub type Result<T> = std::result::Result<T, Error>;
   ```

---

### Phase 5: Test Fixtures and Tests

1. **Create `test-fixtures/` directory structure**

   ```
   test-fixtures/
   ├── musicxml/
   │   ├── simple.musicxml        # User-provided: single staff, few measures
   │   ├── multi-page.musicxml    # User-provided: longer piece for pagination
   │   └── piano.musicxml         # User-provided: grand staff (two staves)
   ├── mei/
   │   └── simple.mei             # Basic MEI file
   └── abc/
       └── simple.abc             # Basic ABC notation
   ```

2. **Create integration tests: `crates/verovioxide/tests/integration_test.rs`**

   ```rust
   use verovioxide::{Toolkit, Options};

   #[test]
   fn test_version() {
       let tk = Toolkit::new().unwrap();
       let version = tk.version();
       assert!(version.starts_with("5."), "Expected version 5.x, got {}", version);
   }

   #[test]
   fn test_render_simple_musicxml() {
       let mut tk = Toolkit::new().unwrap();
       let musicxml = include_str!("../../../test-fixtures/musicxml/simple.musicxml");
       tk.load_data(musicxml).unwrap();

       assert!(tk.page_count() >= 1, "Expected at least 1 page");

       let svg = tk.render_to_svg(1).unwrap();
       assert!(svg.contains("<svg"), "SVG should contain opening tag");
       assert!(svg.contains("</svg>"), "SVG should contain closing tag");
   }

   #[test]
   fn test_render_with_options() {
       let mut tk = Toolkit::new().unwrap();
       let options = Options::new()
           .scale(50)
           .page_width(1000)
           .adjust_page_height(true);

       tk.set_options(&options).unwrap();

       let musicxml = include_str!("../../../test-fixtures/musicxml/simple.musicxml");
       tk.load_data(musicxml).unwrap();

       let svg = tk.render_to_svg(1).unwrap();
       assert!(svg.contains("<svg"));
   }

   #[test]
   fn test_render_piano_grand_staff() {
       let mut tk = Toolkit::new().unwrap();
       let musicxml = include_str!("../../../test-fixtures/musicxml/piano.musicxml");
       tk.load_data(musicxml).unwrap();

       let svg = tk.render_to_svg(1).unwrap();
       assert!(svg.contains("<svg"));
       // Grand staff should have multiple staff elements
       assert!(svg.matches("<g class=\"staff\"").count() >= 2
               || svg.matches("staff").count() >= 2,
               "Grand staff should have at least 2 staves");
   }

   #[test]
   fn test_render_mei() {
       let mut tk = Toolkit::new().unwrap();
       let mei = include_str!("../../../test-fixtures/mei/simple.mei");
       tk.load_data(mei).unwrap();

       let svg = tk.render_to_svg(1).unwrap();
       assert!(svg.contains("<svg"));
   }

   #[test]
   fn test_render_abc() {
       let mut tk = Toolkit::new().unwrap();
       let abc = r#"
   X:1
   T:Simple Tune
   M:4/4
   L:1/4
   K:C
   CDEF|GABc|cBAG|FEDC|
   "#;
       tk.load_data(abc).unwrap();

       let svg = tk.render_to_svg(1).unwrap();
       assert!(svg.contains("<svg"));
   }

   #[test]
   fn test_multi_page_rendering() {
       let mut tk = Toolkit::new().unwrap();
       let options = Options::new()
           .page_height(500)  // Force multiple pages with small height
           .adjust_page_height(false);
       tk.set_options(&options).unwrap();

       let musicxml = include_str!("../../../test-fixtures/musicxml/multi-page.musicxml");
       tk.load_data(musicxml).unwrap();

       let page_count = tk.page_count();
       assert!(page_count >= 1, "Should have at least 1 page");

       for page in 1..=page_count {
           let svg = tk.render_to_svg(page).unwrap();
           assert!(svg.contains("<svg"), "Page {} should be valid SVG", page);
       }
   }

   #[test]
   fn test_get_mei_export() {
       let mut tk = Toolkit::new().unwrap();
       let musicxml = include_str!("../../../test-fixtures/musicxml/simple.musicxml");
       tk.load_data(musicxml).unwrap();

       let mei = tk.get_mei().unwrap();
       assert!(mei.contains("<mei"), "Export should contain MEI root element");
   }

   #[test]
   fn test_invalid_input_returns_error() {
       let mut tk = Toolkit::new().unwrap();
       let result = tk.load_data("this is not valid musicxml or mei or abc");
       // Verovio may or may not error on invalid input, but should not panic
       // If it does error, we should get a LoadError
       if result.is_err() {
           match result.unwrap_err() {
               verovioxide::Error::LoadError(_) => (), // Expected
               e => panic!("Expected LoadError, got {:?}", e),
           }
       }
   }
   ```

3. **Add unit tests for options: append to `crates/verovioxide/src/options.rs`**

   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_options_serialization() {
           let options = Options::new()
               .scale(50)
               .page_width(2100)
               .font("Bravura");

           let json = serde_json::to_string(&options).unwrap();
           assert!(json.contains("\"scale\":50"));
           assert!(json.contains("\"pageWidth\":2100"));
           assert!(json.contains("\"font\":\"Bravura\""));
       }

       #[test]
       fn test_options_skip_none() {
           let options = Options::new().scale(50);
           let json = serde_json::to_string(&options).unwrap();

           assert!(json.contains("scale"));
           assert!(!json.contains("pageWidth"));
           assert!(!json.contains("pageHeight"));
       }

       #[test]
       fn test_options_camel_case() {
           let options = Options::new()
               .page_width(100)
               .page_height(200)
               .adjust_page_height(true);

           let json = serde_json::to_string(&options).unwrap();

           // Should use camelCase, not snake_case
           assert!(json.contains("pageWidth"));
           assert!(json.contains("pageHeight"));
           assert!(json.contains("adjustPageHeight"));
           assert!(!json.contains("page_width"));
           assert!(!json.contains("page_height"));
       }

       #[test]
       fn test_default_options() {
           let options = Options::default();
           let json = serde_json::to_string(&options).unwrap();
           assert_eq!(json, "{}");
       }
   }
   ```

---

### Phase 6: Examples

1. **Create `crates/verovioxide/examples/render_musicxml.rs`**

   ```rust
   //! Render a MusicXML file to SVG
   //!
   //! Usage: cargo run --example render_musicxml <input.musicxml> [output.svg]

   use std::fs;
   use verovioxide::{Toolkit, Options};

   fn main() -> verovioxide::Result<()> {
       let args: Vec<String> = std::env::args().collect();

       if args.len() < 2 {
           eprintln!("Usage: {} <input.musicxml> [output.svg]", args[0]);
           std::process::exit(1);
       }

       let input_file = &args[1];
       let output_file = args.get(2).map(String::as_str).unwrap_or("output.svg");

       println!("Initializing Verovio toolkit...");
       let mut toolkit = Toolkit::new()?;
       println!("Verovio version: {}", toolkit.version());

       let options = Options::new()
           .scale(40)
           .adjust_page_height(true);
       toolkit.set_options(&options)?;

       println!("Loading {}...", input_file);
       let musicxml = fs::read_to_string(input_file)?;
       toolkit.load_data(&musicxml)?;

       let page_count = toolkit.page_count();
       println!("Loaded {} page(s)", page_count);

       println!("Rendering page 1...");
       let svg = toolkit.render_to_svg(1)?;
       fs::write(output_file, &svg)?;

       println!("Written to {}", output_file);
       Ok(())
   }
   ```

2. **Create `crates/verovioxide/examples/render_all_pages.rs`**

   ```rust
   //! Render all pages of a music file to separate SVGs
   //!
   //! Usage: cargo run --example render_all_pages <input> <output_prefix>

   use std::fs;
   use verovioxide::{Toolkit, Options};

   fn main() -> verovioxide::Result<()> {
       let args: Vec<String> = std::env::args().collect();

       if args.len() < 2 {
           eprintln!("Usage: {} <input> [output_prefix]", args[0]);
           std::process::exit(1);
       }

       let input_file = &args[1];
       let output_prefix = args.get(2).map(String::as_str).unwrap_or("page");

       let mut toolkit = Toolkit::new()?;
       println!("Verovio version: {}", toolkit.version());

       let options = Options::new()
           .scale(40)
           .page_width(2100)
           .page_height(2970);  // A4-ish proportions
       toolkit.set_options(&options)?;

       let data = fs::read_to_string(input_file)?;
       toolkit.load_data(&data)?;

       let page_count = toolkit.page_count();
       println!("Rendering {} pages...", page_count);

       for page in 1..=page_count {
           let svg = toolkit.render_to_svg(page)?;
           let filename = format!("{}-{:03}.svg", output_prefix, page);
           fs::write(&filename, &svg)?;
           println!("  Written: {}", filename);
       }

       println!("Done!");
       Ok(())
   }
   ```

3. **Create `crates/verovioxide/examples/render_abc.rs`**

   ```rust
   //! Render ABC notation to SVG
   //!
   //! Usage: cargo run --example render_abc

   use std::fs;
   use verovioxide::{Toolkit, Options};

   fn main() -> verovioxide::Result<()> {
       let mut toolkit = Toolkit::new()?;
       println!("Verovio version: {}", toolkit.version());

       let options = Options::new()
           .scale(40)
           .adjust_page_height(true);
       toolkit.set_options(&options)?;

       // Simple ABC tune
       let abc = r#"
   X:1
   T:Twinkle Twinkle Little Star
   C:Traditional
   M:4/4
   L:1/4
   K:C
   CC GG|AA G2|FF EE|DD C2|
   GG FF|EE D2|GG FF|EE D2|
   CC GG|AA G2|FF EE|DD C2|
   "#;

       println!("Loading ABC notation...");
       toolkit.load_data(abc)?;

       let svg = toolkit.render_to_svg(1)?;
       fs::write("twinkle.svg", &svg)?;

       println!("Written to twinkle.svg");
       Ok(())
   }
   ```

---

### Phase 7: CI Setup

**Create `.github/workflows/ci.yml`**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  test:
    name: Test (${{ matrix.os }}, ${{ matrix.rust }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest]
        rust: [stable, beta]

    steps:
      - name: Checkout
        uses: actions/checkout@v4
        with:
          submodules: recursive

      - name: Install Rust
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Install dependencies (Ubuntu)
        if: matrix.os == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y cmake build-essential pkg-config

      - name: Install dependencies (macOS)
        if: matrix.os == 'macos-latest'
        run: |
          brew install cmake

      - name: Build
        run: cargo build --workspace --all-features

      - name: Run tests
        run: cargo test --workspace --all-features

      - name: Run examples
        run: |
          cargo run --example render_musicxml -- test-fixtures/musicxml/simple.musicxml /tmp/test-output.svg
          test -f /tmp/test-output.svg
          echo "SVG file created successfully"

  fmt:
    name: Formatting
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y cmake build-essential pkg-config
      - run: cargo clippy --workspace --all-features -- -D warnings

  docs:
    name: Documentation
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - uses: dtolnay/rust-toolchain@stable
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y cmake build-essential pkg-config
      - run: cargo doc --workspace --all-features --no-deps
        env:
          RUSTDOCFLAGS: -D warnings
```

---

### Phase 8: Documentation

1. **Create `README.md`**

   ```markdown
   # verovioxide

   [![CI](https://github.com/oxur/verovioxide/actions/workflows/ci.yml/badge.svg)](https://github.com/oxur/verovioxide/actions/workflows/ci.yml)
   [![Crates.io](https://img.shields.io/crates/v/verovioxide.svg)](https://crates.io/crates/verovioxide)
   [![Documentation](https://docs.rs/verovioxide/badge.svg)](https://docs.rs/verovioxide)

   Safe Rust bindings to [Verovio](https://www.verovio.org/), the music notation
   engraving library.

   ## Features

   - 🎼 Render MusicXML, MEI, ABC, and Humdrum notation to SVG
   - 🎨 Bundled SMuFL fonts (Leipzig, Bravura, Gootville, Leland, Petaluma)
   - 🔒 Type-safe options API with serde serialization
   - 📦 No runtime dependencies (statically linked Verovio)
   - 🦀 Safe Rust wrapper over C FFI

   ## Installation

   Add to your `Cargo.toml`:

   ```toml
   [dependencies]
   verovioxide = "0.1"
   ```

   ## Quick Start

   ```rust
   use verovioxide::{Toolkit, Options};

   fn main() -> verovioxide::Result<()> {
       // Create toolkit with bundled fonts
       let mut toolkit = Toolkit::new()?;

       // Configure rendering options
       let options = Options::new()
           .scale(40)
           .adjust_page_height(true);
       toolkit.set_options(&options)?;

       // Load MusicXML (also supports MEI, ABC, Humdrum)
       let musicxml = std::fs::read_to_string("score.musicxml")?;
       toolkit.load_data(&musicxml)?;

       // Render each page to SVG
       for page in 1..=toolkit.page_count() {
           let svg = toolkit.render_to_svg(page)?;
           std::fs::write(format!("page-{}.svg", page), &svg)?;
       }

       Ok(())
   }
   ```

   ## Supported Input Formats

   | Format | Extensions | Notes |
   |--------|------------|-------|
   | MusicXML | `.musicxml`, `.xml`, `.mxl` | Most common interchange format |
   | MEI | `.mei` | Music Encoding Initiative |
   | ABC | `.abc` | Simple text-based notation |
   | Humdrum | `.krn` | Requires `humdrum` feature |

   ## Feature Flags

   | Feature | Default | Description |
   |---------|---------|-------------|
   | `bundled-data` | ✓ | Include Leipzig font |
   | `font-bravura` | | Include Bravura font |
   | `font-leland` | | Include Leland font |
   | `font-petaluma` | | Include Petaluma font |
   | `font-gootville` | | Include Gootville font |
   | `all-fonts` | | Include all available fonts |

   ## Building from Source

   ```bash
   # Clone with submodules
   git clone --recursive https://github.com/oxur/verovioxide.git
   cd verovioxide

   # Build
   cargo build --release

   # Run tests
   cargo test

   # Run example
   cargo run --example render_musicxml -- path/to/score.musicxml output.svg
   ```

   ## License

   This project is dual-licensed under MIT OR Apache-2.0, at your option.

   **Note:** This project links against [Verovio](https://github.com/rism-digital/verovio)
   (LGPL-2.1-or-later) and includes SMuFL fonts (SIL Open Font License 1.1).
   See [NOTICE](NOTICE) for full attribution.

   ## Acknowledgments

   - [Verovio](https://www.verovio.org/) by RISM Digital
   - [SMuFL](https://www.smufl.org/) Standard Music Font Layout

   ```

2. **Create `NOTICE`**

   ```
   verovioxide
   Copyright (c) 2024 oxur contributors

   This project is licensed under the MIT License or Apache License 2.0,
   at your option.

   ---

   This project includes or links to the following third-party components:

   Verovio - Music notation engraving library
   Copyright (c) RISM Digital Center and contributors
   Licensed under LGPL-2.1-or-later
   https://github.com/rism-digital/verovio

   Leipzig Font
   Copyright (c) 2018 RISM Digital Center
   Licensed under SIL Open Font License 1.1

   Bravura Font
   Copyright (c) Steinberg Media Technologies GmbH
   Licensed under SIL Open Font License 1.1

   Gootville Font
   Copyright (c) MuseScore BVBA
   Licensed under SIL Open Font License 1.1

   Leland Font
   Copyright (c) MuseScore BVBA
   Licensed under SIL Open Font License 1.1

   Petaluma Font
   Copyright (c) Steinberg Media Technologies GmbH
   Licensed under SIL Open Font License 1.1
   ```

3. **Create `LICENSE-MIT`**

   ```
   MIT License

   Copyright (c) 2024 oxur contributors

   Permission is hereby granted, free of charge, to any person obtaining a copy
   of this software and associated documentation files (the "Software"), to deal
   in the Software without restriction, including without limitation the rights
   to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
   copies of the Software, and to permit persons to whom the Software is
   furnished to do so, subject to the following conditions:

   The above copyright notice and this permission notice shall be included in all
   copies or substantial portions of the Software.

   THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
   IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
   FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
   AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
   LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
   OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
   SOFTWARE.
   ```

4. **Create `LICENSE-APACHE`**

   ```
                                    Apache License
                              Version 2.0, January 2004
                           http://www.apache.org/licenses/

      [Full Apache 2.0 license text - use standard template]
   ```

---

## Verification Checklist

After implementation, verify:

- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --workspace` passes all tests
- [ ] `cargo clippy --workspace` has no warnings
- [ ] `cargo fmt --check` passes
- [ ] `cargo doc --workspace --no-deps` generates documentation
- [ ] Example `render_musicxml` produces valid SVG output
- [ ] SVG output renders correctly in a browser
- [ ] CI pipeline passes on GitHub Actions
- [ ] All fonts render correctly when enabled

---

## Test Fixtures Needed

Please provide the following test files to be placed in `test-fixtures/`:

1. **`musicxml/simple.musicxml`** — Single staff, 4-8 measures, basic notes
2. **`musicxml/multi-page.musicxml`** — Longer piece (20+ measures) for pagination tests
3. **`musicxml/piano.musicxml`** — Grand staff (treble + bass clef)
4. **`mei/simple.mei`** — Basic MEI file with a few measures

Alternatively, these can be sourced from:

- Verovio's test suite: `verovio/regression/input/`
- MusicXML test suite: <https://github.com/w3c/musicxml/tree/main/schema>

---

## Notes

- **Rust Edition**: Using 2024; change to `2021` if preferred for stability
- **Humdrum Support**: Feature-flagged since it increases binary size ~30%
- **Build Time**: First build compiles Verovio C++ (~2-5 minutes); subsequent builds use cache
- **Binary Size**: ~5-10 MB depending on fonts included
