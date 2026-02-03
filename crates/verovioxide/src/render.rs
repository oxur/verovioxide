//! Unified render API with builder pattern for all output formats.
//!
//! This module provides a consistent, type-safe API for rendering music notation
//! to various output formats. Each format has its own builder type that configures
//! format-specific options.
//!
//! # In-Memory Rendering
//!
//! Use [`Toolkit::render()`](crate::Toolkit::render) with format builders:
//!
//! ```no_run
//! use verovioxide::{Toolkit, Svg, Midi, Timemap, Mei};
//!
//! let mut voxide = Toolkit::new().unwrap();
//! voxide.load("score.mei").unwrap();
//!
//! // SVG rendering
//! let svg = voxide.render(Svg::page(1)).unwrap();
//! let pages = voxide.render(Svg::all_pages()).unwrap();
//!
//! // Other formats
//! let midi = voxide.render(Midi).unwrap();
//! let timemap = voxide.render(Timemap).unwrap();
//! let mei = voxide.render(Mei).unwrap();
//! ```
//!
//! # File Rendering
//!
//! Use [`Toolkit::render_to()`](crate::Toolkit::render_to) for simple cases
//! (format inferred from extension) or
//! [`Toolkit::render_to_as()`](crate::Toolkit::render_to_as) for explicit control:
//!
//! ```no_run
//! use verovioxide::{Toolkit, Svg, Timemap};
//!
//! let mut voxide = Toolkit::new().unwrap();
//! voxide.load("score.mei").unwrap();
//!
//! // Infer format from extension
//! voxide.render_to("output.svg").unwrap();
//! voxide.render_to("output.mid").unwrap();
//!
//! // Explicit format specification
//! voxide.render_to_as("output.svg", Svg::page(3)).unwrap();
//! voxide.render_to_as("output.svg", Svg::all_pages()).unwrap();
//! voxide.render_to_as("output.json", Timemap).unwrap();
//! ```

use crate::{Error, Result, Toolkit};
use std::fs;
use std::path::Path;

// =============================================================================
// Traits
// =============================================================================

/// Trait for in-memory rendering with format-specific output type.
///
/// Each render format implements this trait, specifying its output type
/// (e.g., `String` for single-page SVG, `Vec<String>` for all pages).
pub trait RenderOutput {
    /// The type returned by this render operation.
    type Output;

    /// Perform the render operation using the given toolkit.
    fn render(self, toolkit: &Toolkit) -> Result<Self::Output>;
}

/// Trait for file-based rendering with format-specific behavior.
///
/// Implementations handle writing output to files, including multi-page
/// directory creation for SVG page ranges.
pub trait RenderSpec {
    /// Render to a file at the given path.
    fn render_to_file(self, toolkit: &Toolkit, path: &Path) -> Result<()>;
}

// =============================================================================
// SVG Format Types
// =============================================================================

/// SVG format builder.
///
/// Use the static methods to create page-specific render specifications:
///
/// ```no_run
/// use verovioxide::{Toolkit, Svg};
///
/// let mut voxide = Toolkit::new().unwrap();
/// voxide.load("score.mei").unwrap();
///
/// let svg = voxide.render(Svg::page(1)).unwrap();
/// let pages = voxide.render(Svg::all_pages()).unwrap();
/// ```
pub struct Svg;

impl Svg {
    /// Render a single page.
    ///
    /// Page numbers are 1-indexed.
    pub fn page(n: u32) -> SvgPage {
        SvgPage {
            page: n,
            declaration: false,
        }
    }

    /// Render a range of pages.
    ///
    /// Page numbers are 1-indexed and inclusive.
    pub fn pages(start: u32, end: u32) -> SvgPages {
        SvgPages {
            start,
            end,
            declaration: false,
        }
    }

    /// Render all pages.
    pub fn all_pages() -> SvgAllPages {
        SvgAllPages { declaration: false }
    }
}

/// Single SVG page render specification.
#[derive(Debug, Clone)]
pub struct SvgPage {
    page: u32,
    declaration: bool,
}

impl SvgPage {
    /// Include XML declaration (`<?xml ...?>`) in the output.
    pub fn with_declaration(mut self) -> Self {
        self.declaration = true;
        self
    }

    /// Get the page number.
    pub fn page(&self) -> u32 {
        self.page
    }
}

impl RenderOutput for SvgPage {
    type Output = String;

    fn render(self, toolkit: &Toolkit) -> Result<Self::Output> {
        if self.declaration {
            toolkit.render_to_svg_with_declaration(self.page)
        } else {
            toolkit.render_to_svg(self.page)
        }
    }
}

impl RenderSpec for SvgPage {
    fn render_to_file(self, toolkit: &Toolkit, path: &Path) -> Result<()> {
        let svg = self.render(toolkit)?;
        fs::write(path, &svg).map_err(Error::IoError)?;
        Ok(())
    }
}

/// SVG page range render specification.
#[derive(Debug, Clone)]
pub struct SvgPages {
    start: u32,
    end: u32,
    declaration: bool,
}

impl SvgPages {
    /// Include XML declaration in each page output.
    pub fn with_declaration(mut self) -> Self {
        self.declaration = true;
        self
    }
}

impl RenderOutput for SvgPages {
    type Output = Vec<String>;

    fn render(self, toolkit: &Toolkit) -> Result<Self::Output> {
        let mut pages = Vec::with_capacity((self.end - self.start + 1) as usize);
        for page in self.start..=self.end {
            let svg = if self.declaration {
                toolkit.render_to_svg_with_declaration(page)?
            } else {
                toolkit.render_to_svg(page)?
            };
            pages.push(svg);
        }
        Ok(pages)
    }
}

impl RenderSpec for SvgPages {
    fn render_to_file(self, toolkit: &Toolkit, path: &Path) -> Result<()> {
        // Create directory named after the file (without extension)
        let dir = path.with_extension("");
        fs::create_dir_all(&dir).map_err(Error::IoError)?;

        for page in self.start..=self.end {
            let svg = if self.declaration {
                toolkit.render_to_svg_with_declaration(page)?
            } else {
                toolkit.render_to_svg(page)?
            };
            let page_path = dir.join(format!("page-{:03}.svg", page));
            fs::write(&page_path, &svg).map_err(Error::IoError)?;
        }
        Ok(())
    }
}

/// Render all SVG pages specification.
#[derive(Debug, Clone)]
pub struct SvgAllPages {
    declaration: bool,
}

impl SvgAllPages {
    /// Include XML declaration in each page output.
    pub fn with_declaration(mut self) -> Self {
        self.declaration = true;
        self
    }
}

impl RenderOutput for SvgAllPages {
    type Output = Vec<String>;

    fn render(self, toolkit: &Toolkit) -> Result<Self::Output> {
        let count = toolkit.page_count();
        if count == 0 {
            return Err(Error::RenderError("no data loaded".into()));
        }

        let mut pages = Vec::with_capacity(count as usize);
        for page in 1..=count {
            let svg = if self.declaration {
                toolkit.render_to_svg_with_declaration(page)?
            } else {
                toolkit.render_to_svg(page)?
            };
            pages.push(svg);
        }
        Ok(pages)
    }
}

impl RenderSpec for SvgAllPages {
    fn render_to_file(self, toolkit: &Toolkit, path: &Path) -> Result<()> {
        let count = toolkit.page_count();
        if count == 0 {
            return Err(Error::RenderError("no data loaded".into()));
        }

        // Create directory named after the file (without extension)
        let dir = path.with_extension("");
        fs::create_dir_all(&dir).map_err(Error::IoError)?;

        for page in 1..=count {
            let svg = if self.declaration {
                toolkit.render_to_svg_with_declaration(page)?
            } else {
                toolkit.render_to_svg(page)?
            };
            let page_path = dir.join(format!("page-{:03}.svg", page));
            fs::write(&page_path, &svg).map_err(Error::IoError)?;
        }
        Ok(())
    }
}

// =============================================================================
// Simple Format Types (Unit Structs)
// =============================================================================

/// MIDI format (base64-encoded).
#[derive(Debug, Clone, Copy)]
pub struct Midi;

impl RenderOutput for Midi {
    type Output = String;

    fn render(self, toolkit: &Toolkit) -> Result<Self::Output> {
        toolkit.render_to_midi()
    }
}

impl RenderSpec for Midi {
    fn render_to_file(self, toolkit: &Toolkit, path: &Path) -> Result<()> {
        toolkit.render_to_midi_file(path)
    }
}

/// Plaine & Easie (PAE) format.
#[derive(Debug, Clone, Copy)]
pub struct Pae;

impl RenderOutput for Pae {
    type Output = String;

    fn render(self, toolkit: &Toolkit) -> Result<Self::Output> {
        toolkit.render_to_pae()
    }
}

impl RenderSpec for Pae {
    fn render_to_file(self, toolkit: &Toolkit, path: &Path) -> Result<()> {
        toolkit.render_to_pae_file(path)
    }
}

/// Expansion map (JSON format).
#[derive(Debug, Clone, Copy)]
pub struct ExpansionMap;

impl RenderOutput for ExpansionMap {
    type Output = String;

    fn render(self, toolkit: &Toolkit) -> Result<Self::Output> {
        toolkit.render_to_expansion_map()
    }
}

impl RenderSpec for ExpansionMap {
    fn render_to_file(self, toolkit: &Toolkit, path: &Path) -> Result<()> {
        toolkit.render_to_expansion_map_file(path)
    }
}

/// Humdrum format.
#[derive(Debug, Clone, Copy)]
pub struct Humdrum;

impl RenderOutput for Humdrum {
    type Output = String;

    fn render(self, toolkit: &Toolkit) -> Result<Self::Output> {
        toolkit.get_humdrum()
    }
}

impl RenderSpec for Humdrum {
    fn render_to_file(self, toolkit: &Toolkit, path: &Path) -> Result<()> {
        toolkit.save_humdrum_to_file(path)
    }
}

// =============================================================================
// Timemap Format with Options
// =============================================================================

/// Timemap format (JSON).
///
/// Use `Timemap` for defaults or `Timemap::with_options()` for custom settings:
///
/// ```no_run
/// use verovioxide::{Toolkit, Timemap};
///
/// let mut voxide = Toolkit::new().unwrap();
/// voxide.load("score.mei").unwrap();
///
/// // Default options
/// let timemap = voxide.render(Timemap).unwrap();
///
/// // Custom options
/// let timemap = voxide.render(
///     Timemap::with_options()
///         .include_measures(true)
///         .include_rests(false)
/// ).unwrap();
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Timemap;

impl Timemap {
    /// Create a timemap options builder.
    pub fn with_options() -> TimemapOptionsBuilder {
        TimemapOptionsBuilder::default()
    }
}

impl RenderOutput for Timemap {
    type Output = String;

    fn render(self, toolkit: &Toolkit) -> Result<Self::Output> {
        toolkit.render_to_timemap()
    }
}

impl RenderSpec for Timemap {
    fn render_to_file(self, toolkit: &Toolkit, path: &Path) -> Result<()> {
        toolkit.render_to_timemap_file(path, None)
    }
}

/// Builder for timemap options.
#[derive(Debug, Clone, Default)]
pub struct TimemapOptionsBuilder {
    include_measures: Option<bool>,
    include_rests: Option<bool>,
}

impl TimemapOptionsBuilder {
    /// Include measure information in the timemap.
    pub fn include_measures(mut self, v: bool) -> Self {
        self.include_measures = Some(v);
        self
    }

    /// Include rest events in the timemap.
    pub fn include_rests(mut self, v: bool) -> Self {
        self.include_rests = Some(v);
        self
    }

    /// Build the timemap options JSON string.
    fn to_json(&self) -> String {
        let mut parts = Vec::new();
        if let Some(v) = self.include_measures {
            parts.push(format!("\"includeMeasures\":{}", v));
        }
        if let Some(v) = self.include_rests {
            parts.push(format!("\"includeRests\":{}", v));
        }
        format!("{{{}}}", parts.join(","))
    }
}

impl RenderOutput for TimemapOptionsBuilder {
    type Output = String;

    fn render(self, toolkit: &Toolkit) -> Result<Self::Output> {
        toolkit.render_to_timemap_with_options(&self.to_json())
    }
}

impl RenderSpec for TimemapOptionsBuilder {
    fn render_to_file(self, toolkit: &Toolkit, path: &Path) -> Result<()> {
        toolkit.render_to_timemap_file(path, Some(&self.to_json()))
    }
}

// =============================================================================
// MEI Format with Options
// =============================================================================

/// MEI export format.
///
/// Use `Mei` for defaults or `Mei::with_options()` for custom settings:
///
/// ```no_run
/// use verovioxide::{Toolkit, Mei};
///
/// let mut voxide = Toolkit::new().unwrap();
/// voxide.load("score.musicxml").unwrap();
///
/// // Default export
/// let mei = voxide.render(Mei).unwrap();
///
/// // Custom options
/// let mei = voxide.render(
///     Mei::with_options()
///         .remove_ids(true)
/// ).unwrap();
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Mei;

impl Mei {
    /// Create an MEI options builder.
    pub fn with_options() -> MeiOptionsBuilder {
        MeiOptionsBuilder::default()
    }
}

impl RenderOutput for Mei {
    type Output = String;

    fn render(self, toolkit: &Toolkit) -> Result<Self::Output> {
        toolkit.get_mei()
    }
}

impl RenderSpec for Mei {
    fn render_to_file(self, toolkit: &Toolkit, path: &Path) -> Result<()> {
        let mei = toolkit.get_mei()?;
        fs::write(path, &mei).map_err(Error::IoError)?;
        Ok(())
    }
}

/// Builder for MEI export options.
#[derive(Debug, Clone, Default)]
pub struct MeiOptionsBuilder {
    remove_ids: Option<bool>,
    page_based: Option<bool>,
    scorebased_mei: Option<bool>,
}

impl MeiOptionsBuilder {
    /// Remove auto-generated IDs from the output.
    pub fn remove_ids(mut self, v: bool) -> Self {
        self.remove_ids = Some(v);
        self
    }

    /// Generate page-based MEI output.
    pub fn page_based(mut self, v: bool) -> Self {
        self.page_based = Some(v);
        self
    }

    /// Generate score-based MEI output.
    pub fn scorebased_mei(mut self, v: bool) -> Self {
        self.scorebased_mei = Some(v);
        self
    }

    /// Build the MEI options JSON string.
    fn to_json(&self) -> String {
        let mut parts = Vec::new();
        if let Some(v) = self.remove_ids {
            parts.push(format!("\"removeIds\":{}", v));
        }
        if let Some(v) = self.page_based {
            parts.push(format!("\"pageBasedMei\":{}", v));
        }
        if let Some(v) = self.scorebased_mei {
            parts.push(format!("\"scoreBasedMei\":{}", v));
        }
        format!("{{{}}}", parts.join(","))
    }
}

impl RenderOutput for MeiOptionsBuilder {
    type Output = String;

    fn render(self, toolkit: &Toolkit) -> Result<Self::Output> {
        toolkit.get_mei_with_options(&self.to_json())
    }
}

impl RenderSpec for MeiOptionsBuilder {
    fn render_to_file(self, toolkit: &Toolkit, path: &Path) -> Result<()> {
        let mei = toolkit.get_mei_with_options(&self.to_json())?;
        fs::write(path, &mei).map_err(Error::IoError)?;
        Ok(())
    }
}

// =============================================================================
// PNG Format Types (feature-gated)
// =============================================================================

/// PNG format builder.
///
/// Use the static methods to create page-specific render specifications.
/// Returns raw PNG bytes (`Vec<u8>`) suitable for use with image processing
/// libraries like `image` or terminal display with `viuer`.
///
/// *Added in 0.3.0.*
///
/// # Example
///
/// ```no_run
/// use verovioxide::{Toolkit, Png};
///
/// let mut voxide = Toolkit::new().unwrap();
/// voxide.load("score.mei").unwrap();
///
/// // Render single page to PNG bytes
/// let png_bytes: Vec<u8> = voxide.render(Png::page(1)).unwrap();
///
/// // Render with options
/// let png_bytes: Vec<u8> = voxide.render(
///     Png::page(1).width(800).white_background()
/// ).unwrap();
///
/// // Render all pages
/// let all_pngs: Vec<Vec<u8>> = voxide.render(Png::all_pages()).unwrap();
/// ```
#[cfg(feature = "png")]
#[cfg_attr(docsrs, doc(cfg(feature = "png")))]
pub struct Png;

#[cfg(feature = "png")]
impl Png {
    /// Render a single page to PNG.
    ///
    /// Page numbers are 1-indexed.
    ///
    /// *Added in 0.3.0.*
    pub fn page(n: u32) -> PngPage {
        PngPage {
            page: n,
            options: PngOptions::default(),
        }
    }

    /// Render a range of pages to PNG.
    ///
    /// Page numbers are 1-indexed and inclusive.
    ///
    /// *Added in 0.3.0.*
    pub fn pages(start: u32, end: u32) -> PngPages {
        PngPages {
            start,
            end,
            options: PngOptions::default(),
        }
    }

    /// Render all pages to PNG.
    ///
    /// *Added in 0.3.0.*
    pub fn all_pages() -> PngAllPages {
        PngAllPages {
            options: PngOptions::default(),
        }
    }
}

/// PNG rendering options.
///
/// Controls the output dimensions and background color for PNG rendering.
///
/// *Added in 0.3.0.*
#[cfg(feature = "png")]
#[cfg_attr(docsrs, doc(cfg(feature = "png")))]
#[derive(Debug, Clone, Default)]
pub struct PngOptions {
    /// Target width in pixels. Scales proportionally if height is not set.
    pub(crate) width: Option<u32>,
    /// Target height in pixels. Scales proportionally if width is not set.
    pub(crate) height: Option<u32>,
    /// Zoom factor (1.0 = original size, 2.0 = double size).
    pub(crate) scale: Option<f32>,
    /// Background color. None = transparent (default).
    pub(crate) background: Option<tiny_skia::Color>,
}

/// Single PNG page render specification.
///
/// *Added in 0.3.0.*
#[cfg(feature = "png")]
#[cfg_attr(docsrs, doc(cfg(feature = "png")))]
#[derive(Debug, Clone)]
pub struct PngPage {
    page: u32,
    options: PngOptions,
}

#[cfg(feature = "png")]
impl PngPage {
    /// Set target width in pixels (scales proportionally).
    ///
    /// *Added in 0.3.0.*
    pub fn width(mut self, w: u32) -> Self {
        self.options.width = Some(w);
        self
    }

    /// Set target height in pixels (scales proportionally).
    ///
    /// *Added in 0.3.0.*
    pub fn height(mut self, h: u32) -> Self {
        self.options.height = Some(h);
        self
    }

    /// Set zoom factor (1.0 = original size).
    ///
    /// *Added in 0.3.0.*
    pub fn scale(mut self, s: f32) -> Self {
        self.options.scale = Some(s);
        self
    }

    /// Set background color with RGBA values.
    ///
    /// *Added in 0.3.0.*
    pub fn background(mut self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.options.background = Some(tiny_skia::Color::from_rgba8(r, g, b, a));
        self
    }

    /// Set white opaque background.
    ///
    /// *Added in 0.3.0.*
    pub fn white_background(mut self) -> Self {
        self.options.background = Some(tiny_skia::Color::WHITE);
        self
    }

    /// Get the page number.
    pub fn page(&self) -> u32 {
        self.page
    }
}

#[cfg(feature = "png")]
impl RenderOutput for PngPage {
    type Output = Vec<u8>;

    fn render(self, toolkit: &Toolkit) -> Result<Self::Output> {
        let svg = toolkit.render_to_svg(self.page)?;
        svg_to_png(&svg, &self.options)
    }
}

#[cfg(feature = "png")]
impl RenderSpec for PngPage {
    fn render_to_file(self, toolkit: &Toolkit, path: &Path) -> Result<()> {
        let png_bytes = RenderOutput::render(self, toolkit)?;
        fs::write(path, &png_bytes).map_err(Error::IoError)?;
        Ok(())
    }
}

/// PNG page range render specification.
///
/// *Added in 0.3.0.*
#[cfg(feature = "png")]
#[cfg_attr(docsrs, doc(cfg(feature = "png")))]
#[derive(Debug, Clone)]
pub struct PngPages {
    start: u32,
    end: u32,
    options: PngOptions,
}

#[cfg(feature = "png")]
impl PngPages {
    /// Set target width in pixels for all pages.
    ///
    /// *Added in 0.3.0.*
    pub fn width(mut self, w: u32) -> Self {
        self.options.width = Some(w);
        self
    }

    /// Set target height in pixels for all pages.
    ///
    /// *Added in 0.3.0.*
    pub fn height(mut self, h: u32) -> Self {
        self.options.height = Some(h);
        self
    }

    /// Set zoom factor for all pages.
    ///
    /// *Added in 0.3.0.*
    pub fn scale(mut self, s: f32) -> Self {
        self.options.scale = Some(s);
        self
    }

    /// Set background color for all pages.
    ///
    /// *Added in 0.3.0.*
    pub fn background(mut self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.options.background = Some(tiny_skia::Color::from_rgba8(r, g, b, a));
        self
    }

    /// Set white opaque background for all pages.
    ///
    /// *Added in 0.3.0.*
    pub fn white_background(mut self) -> Self {
        self.options.background = Some(tiny_skia::Color::WHITE);
        self
    }
}

#[cfg(feature = "png")]
impl RenderOutput for PngPages {
    type Output = Vec<Vec<u8>>;

    fn render(self, toolkit: &Toolkit) -> Result<Self::Output> {
        let mut pages = Vec::with_capacity((self.end - self.start + 1) as usize);
        for page in self.start..=self.end {
            let svg = toolkit.render_to_svg(page)?;
            let png = svg_to_png(&svg, &self.options)?;
            pages.push(png);
        }
        Ok(pages)
    }
}

#[cfg(feature = "png")]
impl RenderSpec for PngPages {
    fn render_to_file(self, toolkit: &Toolkit, path: &Path) -> Result<()> {
        // Create directory named after the file (without extension)
        let dir = path.with_extension("");
        fs::create_dir_all(&dir).map_err(Error::IoError)?;

        for page in self.start..=self.end {
            let svg = toolkit.render_to_svg(page)?;
            let png = svg_to_png(&svg, &self.options)?;
            let page_path = dir.join(format!("page-{:03}.png", page));
            fs::write(&page_path, &png).map_err(Error::IoError)?;
        }
        Ok(())
    }
}

/// Render all PNG pages specification.
///
/// *Added in 0.3.0.*
#[cfg(feature = "png")]
#[cfg_attr(docsrs, doc(cfg(feature = "png")))]
#[derive(Debug, Clone)]
pub struct PngAllPages {
    options: PngOptions,
}

#[cfg(feature = "png")]
impl PngAllPages {
    /// Set target width in pixels for all pages.
    ///
    /// *Added in 0.3.0.*
    pub fn width(mut self, w: u32) -> Self {
        self.options.width = Some(w);
        self
    }

    /// Set target height in pixels for all pages.
    ///
    /// *Added in 0.3.0.*
    pub fn height(mut self, h: u32) -> Self {
        self.options.height = Some(h);
        self
    }

    /// Set zoom factor for all pages.
    ///
    /// *Added in 0.3.0.*
    pub fn scale(mut self, s: f32) -> Self {
        self.options.scale = Some(s);
        self
    }

    /// Set background color for all pages.
    ///
    /// *Added in 0.3.0.*
    pub fn background(mut self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.options.background = Some(tiny_skia::Color::from_rgba8(r, g, b, a));
        self
    }

    /// Set white opaque background for all pages.
    ///
    /// *Added in 0.3.0.*
    pub fn white_background(mut self) -> Self {
        self.options.background = Some(tiny_skia::Color::WHITE);
        self
    }
}

#[cfg(feature = "png")]
impl RenderOutput for PngAllPages {
    type Output = Vec<Vec<u8>>;

    fn render(self, toolkit: &Toolkit) -> Result<Self::Output> {
        let count = toolkit.page_count();
        if count == 0 {
            return Err(Error::RenderError("no data loaded".into()));
        }

        let mut pages = Vec::with_capacity(count as usize);
        for page in 1..=count {
            let svg = toolkit.render_to_svg(page)?;
            let png = svg_to_png(&svg, &self.options)?;
            pages.push(png);
        }
        Ok(pages)
    }
}

#[cfg(feature = "png")]
impl RenderSpec for PngAllPages {
    fn render_to_file(self, toolkit: &Toolkit, path: &Path) -> Result<()> {
        let count = toolkit.page_count();
        if count == 0 {
            return Err(Error::RenderError("no data loaded".into()));
        }

        // Create directory named after the file (without extension)
        let dir = path.with_extension("");
        fs::create_dir_all(&dir).map_err(Error::IoError)?;

        for page in 1..=count {
            let svg = toolkit.render_to_svg(page)?;
            let png = svg_to_png(&svg, &self.options)?;
            let page_path = dir.join(format!("page-{:03}.png", page));
            fs::write(&page_path, &png).map_err(Error::IoError)?;
        }
        Ok(())
    }
}

// =============================================================================
// PNG Helper Functions
// =============================================================================

/// Convert SVG string to PNG bytes with the given options.
#[cfg(feature = "png")]
fn svg_to_png(svg: &str, options: &PngOptions) -> Result<Vec<u8>> {
    // Parse SVG using usvg
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default())
        .map_err(|e| Error::RenderError(format!("failed to parse SVG for PNG conversion: {}", e)))?;

    // Get original SVG dimensions
    let svg_size = tree.size();
    let original_width = svg_size.width();
    let original_height = svg_size.height();

    // Calculate target dimensions based on options
    let (target_width, target_height, scale_x, scale_y) =
        calculate_png_dimensions(original_width, original_height, options);

    // Create pixmap with target dimensions
    let mut pixmap = tiny_skia::Pixmap::new(target_width, target_height).ok_or_else(|| {
        Error::RenderError(format!(
            "failed to create pixmap with dimensions {}x{}",
            target_width, target_height
        ))
    })?;

    // Apply background color if specified (otherwise transparent)
    if let Some(bg) = options.background {
        pixmap.fill(bg);
    }

    // Create transform for scaling
    let transform = tiny_skia::Transform::from_scale(scale_x, scale_y);

    // Render SVG to pixmap
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Encode to PNG
    pixmap
        .encode_png()
        .map_err(|e| Error::RenderError(format!("failed to encode PNG: {}", e)))
}

/// Calculate target PNG dimensions based on options.
///
/// Returns (width, height, scale_x, scale_y).
#[cfg(feature = "png")]
fn calculate_png_dimensions(
    original_width: f32,
    original_height: f32,
    options: &PngOptions,
) -> (u32, u32, f32, f32) {
    // Priority: explicit scale > width/height constraints > original size

    if let Some(scale) = options.scale {
        // Scale factor takes priority
        let w = (original_width * scale).ceil() as u32;
        let h = (original_height * scale).ceil() as u32;
        return (w.max(1), h.max(1), scale, scale);
    }

    match (options.width, options.height) {
        (Some(w), Some(h)) => {
            // Both dimensions specified - scale to fit, maintaining aspect ratio
            let scale_x = w as f32 / original_width;
            let scale_y = h as f32 / original_height;
            let scale = scale_x.min(scale_y); // Fit within bounds
            let final_w = (original_width * scale).ceil() as u32;
            let final_h = (original_height * scale).ceil() as u32;
            (final_w.max(1), final_h.max(1), scale, scale)
        }
        (Some(w), None) => {
            // Width only - scale proportionally
            let scale = w as f32 / original_width;
            let h = (original_height * scale).ceil() as u32;
            (w.max(1), h.max(1), scale, scale)
        }
        (None, Some(h)) => {
            // Height only - scale proportionally
            let scale = h as f32 / original_height;
            let w = (original_width * scale).ceil() as u32;
            (w.max(1), h.max(1), scale, scale)
        }
        (None, None) => {
            // No constraints - use original size
            let w = original_width.ceil() as u32;
            let h = original_height.ceil() as u32;
            (w.max(1), h.max(1), 1.0, 1.0)
        }
    }
}

// =============================================================================
// Format Inference
// =============================================================================

/// Infer the render format from a file extension.
///
/// Returns an error for ambiguous extensions (like `.json`) that require
/// explicit format specification.
pub(crate) fn infer_format_and_render(toolkit: &Toolkit, path: &Path) -> Result<()> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());

    match ext.as_deref() {
        Some("svg") => SvgPage {
            page: 1,
            declaration: false,
        }
        .render_to_file(toolkit, path),
        #[cfg(feature = "png")]
        Some("png") => PngPage {
            page: 1,
            options: PngOptions::default(),
        }
        .render_to_file(toolkit, path),
        Some("mid") | Some("midi") => Midi.render_to_file(toolkit, path),
        Some("pae") => Pae.render_to_file(toolkit, path),
        Some("mei") => Mei.render_to_file(toolkit, path),
        Some("krn") | Some("hmd") => Humdrum.render_to_file(toolkit, path),
        Some("json") => Err(Error::RenderError(
            "ambiguous .json extension: use render_to_as() with Timemap or ExpansionMap".into(),
        )),
        Some(ext) => Err(Error::RenderError(format!(
            "unsupported file extension: .{}",
            ext
        ))),
        None => Err(Error::RenderError("file path has no extension".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_page_builder() {
        let spec = Svg::page(1);
        assert_eq!(spec.page(), 1);
        assert!(!spec.declaration);

        let spec = Svg::page(3).with_declaration();
        assert_eq!(spec.page(), 3);
        assert!(spec.declaration);
    }

    #[test]
    fn test_svg_pages_builder() {
        let spec = Svg::pages(2, 5);
        assert_eq!(spec.start, 2);
        assert_eq!(spec.end, 5);
        assert!(!spec.declaration);

        let spec = Svg::pages(1, 10).with_declaration();
        assert!(spec.declaration);
    }

    #[test]
    fn test_svg_all_pages_builder() {
        let spec = Svg::all_pages();
        assert!(!spec.declaration);

        let spec = Svg::all_pages().with_declaration();
        assert!(spec.declaration);
    }

    #[test]
    fn test_timemap_options_to_json() {
        let opts = TimemapOptionsBuilder::default();
        assert_eq!(opts.to_json(), "{}");

        let opts = Timemap::with_options().include_measures(true);
        assert_eq!(opts.to_json(), "{\"includeMeasures\":true}");

        let opts = Timemap::with_options()
            .include_measures(true)
            .include_rests(false);
        assert!(opts.to_json().contains("\"includeMeasures\":true"));
        assert!(opts.to_json().contains("\"includeRests\":false"));
    }

    #[test]
    fn test_mei_options_to_json() {
        let opts = MeiOptionsBuilder::default();
        assert_eq!(opts.to_json(), "{}");

        let opts = Mei::with_options().remove_ids(true);
        assert_eq!(opts.to_json(), "{\"removeIds\":true}");

        let opts = Mei::with_options().remove_ids(true).page_based(false);
        assert!(opts.to_json().contains("\"removeIds\":true"));
        assert!(opts.to_json().contains("\"pageBasedMei\":false"));
    }

    #[test]
    fn test_format_types_are_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<Svg>();
        assert_sync::<Svg>();
        assert_send::<SvgPage>();
        assert_send::<SvgPages>();
        assert_send::<SvgAllPages>();
        assert_send::<Midi>();
        assert_sync::<Midi>();
        assert_send::<Pae>();
        assert_send::<ExpansionMap>();
        assert_send::<Humdrum>();
        assert_send::<Timemap>();
        assert_send::<Mei>();
    }

    #[test]
    fn test_mei_options_scorebased() {
        let opts = Mei::with_options().scorebased_mei(true);
        assert!(opts.to_json().contains("\"scoreBasedMei\":true"));

        let opts = Mei::with_options()
            .remove_ids(true)
            .page_based(false)
            .scorebased_mei(true);
        assert!(opts.to_json().contains("\"removeIds\":true"));
        assert!(opts.to_json().contains("\"pageBasedMei\":false"));
        assert!(opts.to_json().contains("\"scoreBasedMei\":true"));
    }

    #[test]
    fn test_svg_page_accessors() {
        let spec = Svg::page(5);
        assert_eq!(spec.page(), 5);
    }

    #[test]
    fn test_svg_pages_range() {
        let spec = Svg::pages(3, 7);
        assert_eq!(spec.start, 3);
        assert_eq!(spec.end, 7);
    }

    #[test]
    fn test_timemap_options_default_empty() {
        let opts = TimemapOptionsBuilder::default();
        assert_eq!(opts.to_json(), "{}");
    }

    #[test]
    fn test_timemap_options_rests_only() {
        let opts = Timemap::with_options().include_rests(true);
        assert_eq!(opts.to_json(), "{\"includeRests\":true}");
    }

    #[test]
    fn test_mei_options_page_based_only() {
        let opts = Mei::with_options().page_based(true);
        assert_eq!(opts.to_json(), "{\"pageBasedMei\":true}");
    }

    #[test]
    fn test_format_debug_impls() {
        // Test Debug implementations for coverage
        let _ = format!("{:?}", Svg::page(1));
        let _ = format!("{:?}", Svg::pages(1, 2));
        let _ = format!("{:?}", Svg::all_pages());
        let _ = format!("{:?}", Midi);
        let _ = format!("{:?}", Pae);
        let _ = format!("{:?}", ExpansionMap);
        let _ = format!("{:?}", Humdrum);
        let _ = format!("{:?}", Timemap);
        let _ = format!("{:?}", Mei);
        let _ = format!("{:?}", Timemap::with_options());
        let _ = format!("{:?}", Mei::with_options());
    }

    #[test]
    fn test_format_clone_impls() {
        // Test Clone implementations for coverage
        let page = Svg::page(1);
        let _cloned = page.clone();

        let pages = Svg::pages(1, 2);
        let _cloned = pages.clone();

        let all = Svg::all_pages();
        let _cloned = all.clone();

        let midi = Midi;
        let _cloned = midi;

        let pae = Pae;
        let _cloned = pae;

        let opts = Timemap::with_options().include_measures(true);
        let _cloned = opts.clone();

        let opts = Mei::with_options().remove_ids(true);
        let _cloned = opts.clone();
    }
}

#[cfg(all(test, feature = "png"))]
mod png_tests {
    use super::*;

    #[test]
    fn test_png_page_builder() {
        let spec = Png::page(1);
        assert_eq!(spec.page(), 1);
        assert!(spec.options.width.is_none());
        assert!(spec.options.height.is_none());
        assert!(spec.options.scale.is_none());
        assert!(spec.options.background.is_none());
    }

    #[test]
    fn test_png_page_with_width() {
        let spec = Png::page(2).width(800);
        assert_eq!(spec.page(), 2);
        assert_eq!(spec.options.width, Some(800));
    }

    #[test]
    fn test_png_page_with_height() {
        let spec = Png::page(1).height(600);
        assert_eq!(spec.options.height, Some(600));
    }

    #[test]
    fn test_png_page_with_scale() {
        let spec = Png::page(1).scale(2.0);
        assert_eq!(spec.options.scale, Some(2.0));
    }

    #[test]
    fn test_png_page_with_background() {
        let spec = Png::page(1).background(255, 255, 255, 255);
        assert!(spec.options.background.is_some());
    }

    #[test]
    fn test_png_page_with_white_background() {
        let spec = Png::page(1).white_background();
        assert!(spec.options.background.is_some());
    }

    #[test]
    fn test_png_page_chained_options() {
        let spec = Png::page(3).width(800).height(600).scale(2.0).white_background();
        assert_eq!(spec.page(), 3);
        assert_eq!(spec.options.width, Some(800));
        assert_eq!(spec.options.height, Some(600));
        assert_eq!(spec.options.scale, Some(2.0));
        assert!(spec.options.background.is_some());
    }

    #[test]
    fn test_png_pages_builder() {
        let spec = Png::pages(2, 5);
        assert_eq!(spec.start, 2);
        assert_eq!(spec.end, 5);
    }

    #[test]
    fn test_png_pages_with_options() {
        let spec = Png::pages(1, 10).width(1024).white_background();
        assert_eq!(spec.options.width, Some(1024));
        assert!(spec.options.background.is_some());
    }

    #[test]
    fn test_png_all_pages_builder() {
        let spec = Png::all_pages();
        assert!(spec.options.width.is_none());
    }

    #[test]
    fn test_png_all_pages_with_options() {
        let spec = Png::all_pages().scale(1.5).white_background();
        assert_eq!(spec.options.scale, Some(1.5));
        assert!(spec.options.background.is_some());
    }

    #[test]
    fn test_calculate_png_dimensions_original() {
        let options = PngOptions::default();
        let (w, h, sx, sy) = calculate_png_dimensions(100.0, 200.0, &options);
        assert_eq!(w, 100);
        assert_eq!(h, 200);
        assert_eq!(sx, 1.0);
        assert_eq!(sy, 1.0);
    }

    #[test]
    fn test_calculate_png_dimensions_scale() {
        let options = PngOptions {
            scale: Some(2.0),
            ..Default::default()
        };
        let (w, h, sx, sy) = calculate_png_dimensions(100.0, 200.0, &options);
        assert_eq!(w, 200);
        assert_eq!(h, 400);
        assert_eq!(sx, 2.0);
        assert_eq!(sy, 2.0);
    }

    #[test]
    fn test_calculate_png_dimensions_width_only() {
        let options = PngOptions {
            width: Some(200),
            ..Default::default()
        };
        let (w, h, sx, sy) = calculate_png_dimensions(100.0, 200.0, &options);
        assert_eq!(w, 200);
        assert_eq!(h, 400);
        assert_eq!(sx, 2.0);
        assert_eq!(sy, 2.0);
    }

    #[test]
    fn test_calculate_png_dimensions_height_only() {
        let options = PngOptions {
            height: Some(100),
            ..Default::default()
        };
        let (w, h, _sx, _sy) = calculate_png_dimensions(100.0, 200.0, &options);
        assert_eq!(w, 50);
        assert_eq!(h, 100);
    }

    #[test]
    fn test_calculate_png_dimensions_both() {
        let options = PngOptions {
            width: Some(200),
            height: Some(200),
            ..Default::default()
        };
        // Original is 100x200, so fitting into 200x200 means scale=1.0 (constrained by height)
        let (w, h, _sx, _sy) = calculate_png_dimensions(100.0, 200.0, &options);
        // Scale by height: 200/200 = 1.0, so width = 100, height = 200
        assert_eq!(w, 100);
        assert_eq!(h, 200);
    }

    #[test]
    fn test_png_format_types_are_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<Png>();
        assert_sync::<Png>();
        assert_send::<PngPage>();
        assert_send::<PngPages>();
        assert_send::<PngAllPages>();
        assert_send::<PngOptions>();
    }

    #[test]
    fn test_png_format_debug_impls() {
        let _ = format!("{:?}", Png::page(1));
        let _ = format!("{:?}", Png::pages(1, 2));
        let _ = format!("{:?}", Png::all_pages());
        let _ = format!("{:?}", PngOptions::default());
    }

    #[test]
    fn test_png_format_clone_impls() {
        let page = Png::page(1).width(800);
        let _cloned = page.clone();

        let pages = Png::pages(1, 2).scale(2.0);
        let _cloned = pages.clone();

        let all = Png::all_pages().white_background();
        let _cloned = all.clone();

        let opts = PngOptions::default();
        let _cloned = opts.clone();
    }
}
