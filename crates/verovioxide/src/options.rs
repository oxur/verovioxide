//! Rendering options for the Verovio toolkit.
//!
//! This module provides a builder-style API for configuring Verovio rendering options.
//! Options are serialized to JSON and passed to the underlying Verovio toolkit.
//!
//! # Example
//!
//! ```
//! use verovioxide::Options;
//!
//! let options = Options::builder()
//!     .scale(80)
//!     .page_width(2100)
//!     .page_height(2970)
//!     .adjust_page_height(true)
//!     .build();
//! ```

use serde::{Deserialize, Serialize};

/// Break mode for page and system breaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BreakMode {
    /// Automatic break placement.
    #[default]
    Auto,
    /// No automatic breaks.
    None,
    /// Use encoded breaks from the input file.
    Encoded,
    /// Break at each line.
    Line,
    /// Smart break placement.
    Smart,
}

/// Condense mode for dense layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CondenseMode {
    /// No condensing.
    #[default]
    None,
    /// Automatic condensing.
    Auto,
    /// Use encoded condensing.
    Encoded,
}

/// Footer display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FooterMode {
    /// No footer.
    #[default]
    None,
    /// Automatic footer.
    Auto,
    /// Use encoded footer.
    Encoded,
    /// Always show footer.
    Always,
}

/// Header display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HeaderMode {
    /// No header.
    #[default]
    None,
    /// Automatic header.
    Auto,
    /// Use encoded header.
    Encoded,
}

/// SMuFL text font to use for text rendering.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextFont {
    /// Times font (default).
    #[default]
    Times,
    /// Custom font name.
    Custom(String),
}

impl TextFont {
    /// Returns the font name as a string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Times => "Times",
            Self::Custom(name) => name,
        }
    }
}

/// Rendering options for the Verovio toolkit.
///
/// This struct provides a type-safe way to configure Verovio rendering options.
/// Use the builder pattern via [`Options::builder()`] to construct options.
///
/// All fields use `Option` to allow partial configuration. When serialized to JSON,
/// only set fields are included, letting Verovio use its defaults for unset values.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    // =========================================================================
    // General Options
    // =========================================================================
    /// Rendering scale as a percentage (e.g., 100 for 100%).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<u32>,

    /// Page width in MEI units.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_width: Option<u32>,

    /// Page height in MEI units.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_height: Option<u32>,

    /// Whether to adjust the page height to the content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjust_page_height: Option<bool>,

    /// Page margin for all sides (in MEI units).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_margin: Option<u32>,

    /// Top page margin (in MEI units).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_margin_top: Option<u32>,

    /// Bottom page margin (in MEI units).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_margin_bottom: Option<u32>,

    /// Left page margin (in MEI units).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_margin_left: Option<u32>,

    /// Right page margin (in MEI units).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_margin_right: Option<u32>,

    // =========================================================================
    // Font Options
    // =========================================================================
    /// The SMuFL music font to use (e.g., "Leipzig", "Bravura").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,

    /// Lyric size as a percentage of the staff size.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lyric_size: Option<f64>,

    // =========================================================================
    // Layout Options
    // =========================================================================
    /// Break mode for page and system breaks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breaks: Option<BreakMode>,

    /// Condense mode for dense layouts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condense: Option<CondenseMode>,

    /// Whether to condense the first page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condense_first_page: Option<bool>,

    /// Minimum width for condensed scores.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condense_tempo_pages: Option<bool>,

    /// Whether to even note spacing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub even_note_spacing: Option<bool>,

    /// The minimum measure width.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_measure_width: Option<u32>,

    /// Header display mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<HeaderMode>,

    /// Footer display mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer: Option<FooterMode>,

    // =========================================================================
    // SVG Output Options
    // =========================================================================
    /// Whether to include the XML declaration in SVG output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub svg_xml_declaration: Option<bool>,

    /// Whether to include bounding boxes in SVG output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub svg_bounding_boxes: Option<bool>,

    /// Whether to use viewBox attribute in SVG output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub svg_view_box: Option<bool>,

    /// Whether to remove xlink namespace from SVG output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub svg_remove_xlink: Option<bool>,

    /// CSS stylesheet to embed in SVG output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub svg_css: Option<String>,

    /// Whether to format SVG output with indentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub svg_format_raw: Option<bool>,

    /// Whether to include font fallback in SVG output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub svg_font_face_include: Option<bool>,

    // =========================================================================
    // MIDI Options
    // =========================================================================
    /// Default MIDI tempo.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub midi_tempo: Option<f64>,

    /// MIDI velocity for notes without dynamics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub midi_velocity: Option<u8>,

    // =========================================================================
    // Input Options
    // =========================================================================
    /// Input format (auto, mei, musicxml, musicxml-compressed, humdrum, pae, abc).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_from: Option<String>,

    // =========================================================================
    // Selection Options
    // =========================================================================
    /// Starting measure for selection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mdiv_x_path_query: Option<String>,

    /// Expansion to use from the MEI document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expansion: Option<String>,

    // =========================================================================
    // Transposition Options
    // =========================================================================
    /// Transposition interval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transpose: Option<String>,

    /// Whether to transpose the written/sounding selection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transpose_selected_only: Option<bool>,

    /// Whether to transpose to written or sounding pitch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transpose_to_sounding_pitch: Option<bool>,

    // =========================================================================
    // Spacing Options
    // =========================================================================
    /// Spacing between staff lines.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing_staff: Option<u32>,

    /// Spacing between systems.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing_system: Option<u32>,

    /// Linear spacing factor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing_linear: Option<f64>,

    /// Non-linear spacing factor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing_non_linear: Option<f64>,
}

impl Options {
    /// Creates a new builder for constructing options.
    #[must_use]
    pub fn builder() -> OptionsBuilder {
        OptionsBuilder::default()
    }

    /// Serializes the options to a JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserializes options from a JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Builder for constructing [`Options`].
///
/// Use [`Options::builder()`] to create a new builder.
#[derive(Debug, Default, Clone)]
pub struct OptionsBuilder {
    options: Options,
}

impl OptionsBuilder {
    /// Sets the rendering scale as a percentage.
    ///
    /// # Arguments
    ///
    /// * `scale` - Scale percentage (e.g., 100 for 100%)
    #[must_use]
    pub fn scale(mut self, scale: u32) -> Self {
        self.options.scale = Some(scale);
        self
    }

    /// Sets the page width in MEI units.
    #[must_use]
    pub fn page_width(mut self, width: u32) -> Self {
        self.options.page_width = Some(width);
        self
    }

    /// Sets the page height in MEI units.
    #[must_use]
    pub fn page_height(mut self, height: u32) -> Self {
        self.options.page_height = Some(height);
        self
    }

    /// Sets whether to adjust the page height to the content.
    #[must_use]
    pub fn adjust_page_height(mut self, adjust: bool) -> Self {
        self.options.adjust_page_height = Some(adjust);
        self
    }

    /// Sets the page margin for all sides in MEI units.
    #[must_use]
    pub fn page_margin(mut self, margin: u32) -> Self {
        self.options.page_margin = Some(margin);
        self
    }

    /// Sets the top page margin in MEI units.
    #[must_use]
    pub fn page_margin_top(mut self, margin: u32) -> Self {
        self.options.page_margin_top = Some(margin);
        self
    }

    /// Sets the bottom page margin in MEI units.
    #[must_use]
    pub fn page_margin_bottom(mut self, margin: u32) -> Self {
        self.options.page_margin_bottom = Some(margin);
        self
    }

    /// Sets the left page margin in MEI units.
    #[must_use]
    pub fn page_margin_left(mut self, margin: u32) -> Self {
        self.options.page_margin_left = Some(margin);
        self
    }

    /// Sets the right page margin in MEI units.
    #[must_use]
    pub fn page_margin_right(mut self, margin: u32) -> Self {
        self.options.page_margin_right = Some(margin);
        self
    }

    /// Sets the SMuFL music font to use.
    #[must_use]
    pub fn font(mut self, font: impl Into<String>) -> Self {
        self.options.font = Some(font.into());
        self
    }

    /// Sets the lyric size as a percentage of staff size.
    #[must_use]
    pub fn lyric_size(mut self, size: f64) -> Self {
        self.options.lyric_size = Some(size);
        self
    }

    /// Sets the break mode for page and system breaks.
    #[must_use]
    pub fn breaks(mut self, mode: BreakMode) -> Self {
        self.options.breaks = Some(mode);
        self
    }

    /// Sets the condense mode for dense layouts.
    #[must_use]
    pub fn condense(mut self, mode: CondenseMode) -> Self {
        self.options.condense = Some(mode);
        self
    }

    /// Sets whether to condense the first page.
    #[must_use]
    pub fn condense_first_page(mut self, condense: bool) -> Self {
        self.options.condense_first_page = Some(condense);
        self
    }

    /// Sets whether to condense tempo pages.
    #[must_use]
    pub fn condense_tempo_pages(mut self, condense: bool) -> Self {
        self.options.condense_tempo_pages = Some(condense);
        self
    }

    /// Sets whether to use even note spacing.
    #[must_use]
    pub fn even_note_spacing(mut self, even: bool) -> Self {
        self.options.even_note_spacing = Some(even);
        self
    }

    /// Sets the minimum measure width.
    #[must_use]
    pub fn min_measure_width(mut self, width: u32) -> Self {
        self.options.min_measure_width = Some(width);
        self
    }

    /// Sets the header display mode.
    #[must_use]
    pub fn header(mut self, mode: HeaderMode) -> Self {
        self.options.header = Some(mode);
        self
    }

    /// Sets the footer display mode.
    #[must_use]
    pub fn footer(mut self, mode: FooterMode) -> Self {
        self.options.footer = Some(mode);
        self
    }

    /// Sets whether to include the XML declaration in SVG output.
    #[must_use]
    pub fn svg_xml_declaration(mut self, include: bool) -> Self {
        self.options.svg_xml_declaration = Some(include);
        self
    }

    /// Sets whether to include bounding boxes in SVG output.
    #[must_use]
    pub fn svg_bounding_boxes(mut self, include: bool) -> Self {
        self.options.svg_bounding_boxes = Some(include);
        self
    }

    /// Sets whether to use viewBox attribute in SVG output.
    #[must_use]
    pub fn svg_view_box(mut self, use_view_box: bool) -> Self {
        self.options.svg_view_box = Some(use_view_box);
        self
    }

    /// Sets whether to remove xlink namespace from SVG output.
    #[must_use]
    pub fn svg_remove_xlink(mut self, remove: bool) -> Self {
        self.options.svg_remove_xlink = Some(remove);
        self
    }

    /// Sets the CSS stylesheet to embed in SVG output.
    #[must_use]
    pub fn svg_css(mut self, css: impl Into<String>) -> Self {
        self.options.svg_css = Some(css.into());
        self
    }

    /// Sets whether to format SVG output with indentation.
    #[must_use]
    pub fn svg_format_raw(mut self, raw: bool) -> Self {
        self.options.svg_format_raw = Some(raw);
        self
    }

    /// Sets whether to include font fallback in SVG output.
    #[must_use]
    pub fn svg_font_face_include(mut self, include: bool) -> Self {
        self.options.svg_font_face_include = Some(include);
        self
    }

    /// Sets the default MIDI tempo.
    #[must_use]
    pub fn midi_tempo(mut self, tempo: f64) -> Self {
        self.options.midi_tempo = Some(tempo);
        self
    }

    /// Sets the MIDI velocity for notes without dynamics.
    #[must_use]
    pub fn midi_velocity(mut self, velocity: u8) -> Self {
        self.options.midi_velocity = Some(velocity);
        self
    }

    /// Sets the input format.
    #[must_use]
    pub fn input_from(mut self, format: impl Into<String>) -> Self {
        self.options.input_from = Some(format.into());
        self
    }

    /// Sets the mdiv XPath query.
    #[must_use]
    pub fn mdiv_x_path_query(mut self, query: impl Into<String>) -> Self {
        self.options.mdiv_x_path_query = Some(query.into());
        self
    }

    /// Sets the expansion to use from the MEI document.
    #[must_use]
    pub fn expansion(mut self, expansion: impl Into<String>) -> Self {
        self.options.expansion = Some(expansion.into());
        self
    }

    /// Sets the transposition interval.
    #[must_use]
    pub fn transpose(mut self, interval: impl Into<String>) -> Self {
        self.options.transpose = Some(interval.into());
        self
    }

    /// Sets whether to transpose only the selection.
    #[must_use]
    pub fn transpose_selected_only(mut self, selected: bool) -> Self {
        self.options.transpose_selected_only = Some(selected);
        self
    }

    /// Sets whether to transpose to sounding pitch.
    #[must_use]
    pub fn transpose_to_sounding_pitch(mut self, sounding: bool) -> Self {
        self.options.transpose_to_sounding_pitch = Some(sounding);
        self
    }

    /// Sets the spacing between staff lines.
    #[must_use]
    pub fn spacing_staff(mut self, spacing: u32) -> Self {
        self.options.spacing_staff = Some(spacing);
        self
    }

    /// Sets the spacing between systems.
    #[must_use]
    pub fn spacing_system(mut self, spacing: u32) -> Self {
        self.options.spacing_system = Some(spacing);
        self
    }

    /// Sets the linear spacing factor.
    #[must_use]
    pub fn spacing_linear(mut self, factor: f64) -> Self {
        self.options.spacing_linear = Some(factor);
        self
    }

    /// Sets the non-linear spacing factor.
    #[must_use]
    pub fn spacing_non_linear(mut self, factor: f64) -> Self {
        self.options.spacing_non_linear = Some(factor);
        self
    }

    /// Builds the options.
    #[must_use]
    pub fn build(self) -> Options {
        self.options
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_builder_scale() {
        let options = Options::builder().scale(80).build();
        assert_eq!(options.scale, Some(80));
    }

    #[test]
    fn test_options_builder_page_dimensions() {
        let options = Options::builder()
            .page_width(2100)
            .page_height(2970)
            .build();
        assert_eq!(options.page_width, Some(2100));
        assert_eq!(options.page_height, Some(2970));
    }

    #[test]
    fn test_options_builder_adjust_page_height() {
        let options = Options::builder().adjust_page_height(true).build();
        assert_eq!(options.adjust_page_height, Some(true));
    }

    #[test]
    fn test_options_builder_font() {
        let options = Options::builder().font("Bravura").build();
        assert_eq!(options.font, Some("Bravura".to_string()));
    }

    #[test]
    fn test_options_builder_breaks() {
        let options = Options::builder().breaks(BreakMode::Encoded).build();
        assert_eq!(options.breaks, Some(BreakMode::Encoded));
    }

    #[test]
    fn test_options_builder_header_footer() {
        let options = Options::builder()
            .header(HeaderMode::Auto)
            .footer(FooterMode::Always)
            .build();
        assert_eq!(options.header, Some(HeaderMode::Auto));
        assert_eq!(options.footer, Some(FooterMode::Always));
    }

    #[test]
    fn test_options_builder_chaining() {
        let options = Options::builder()
            .scale(80)
            .page_width(2100)
            .page_height(2970)
            .adjust_page_height(true)
            .font("Leipzig")
            .breaks(BreakMode::Auto)
            .build();

        assert_eq!(options.scale, Some(80));
        assert_eq!(options.page_width, Some(2100));
        assert_eq!(options.page_height, Some(2970));
        assert_eq!(options.adjust_page_height, Some(true));
        assert_eq!(options.font, Some("Leipzig".to_string()));
        assert_eq!(options.breaks, Some(BreakMode::Auto));
    }

    #[test]
    fn test_options_to_json() {
        let options = Options::builder().scale(80).page_width(2100).build();
        let json = options.to_json().unwrap();
        assert!(json.contains("\"scale\":80"));
        assert!(json.contains("\"pageWidth\":2100"));
    }

    #[test]
    fn test_options_from_json() {
        let json = r#"{"scale":80,"pageWidth":2100}"#;
        let options = Options::from_json(json).unwrap();
        assert_eq!(options.scale, Some(80));
        assert_eq!(options.page_width, Some(2100));
    }

    #[test]
    fn test_options_skip_none_in_json() {
        let options = Options::builder().scale(80).build();
        let json = options.to_json().unwrap();
        assert!(json.contains("scale"));
        assert!(!json.contains("pageWidth"));
    }

    #[test]
    fn test_default_options_empty() {
        let options = Options::default();
        assert!(options.scale.is_none());
        assert!(options.page_width.is_none());
        assert!(options.page_height.is_none());
    }

    #[test]
    fn test_break_mode_serialize() {
        assert_eq!(
            serde_json::to_string(&BreakMode::Auto).unwrap(),
            r#""auto""#
        );
        assert_eq!(
            serde_json::to_string(&BreakMode::Encoded).unwrap(),
            r#""encoded""#
        );
    }

    #[test]
    fn test_break_mode_deserialize() {
        let mode: BreakMode = serde_json::from_str(r#""smart""#).unwrap();
        assert_eq!(mode, BreakMode::Smart);
    }

    #[test]
    fn test_header_mode_serialize() {
        assert_eq!(
            serde_json::to_string(&HeaderMode::None).unwrap(),
            r#""none""#
        );
        assert_eq!(
            serde_json::to_string(&HeaderMode::Auto).unwrap(),
            r#""auto""#
        );
    }

    #[test]
    fn test_footer_mode_serialize() {
        assert_eq!(
            serde_json::to_string(&FooterMode::Always).unwrap(),
            r#""always""#
        );
    }

    #[test]
    fn test_condense_mode_serialize() {
        assert_eq!(
            serde_json::to_string(&CondenseMode::Auto).unwrap(),
            r#""auto""#
        );
    }

    #[test]
    fn test_text_font_default() {
        let font = TextFont::default();
        assert_eq!(font.as_str(), "Times");
    }

    #[test]
    fn test_text_font_custom() {
        let font = TextFont::Custom("Arial".to_string());
        assert_eq!(font.as_str(), "Arial");
    }

    #[test]
    fn test_options_builder_margins() {
        let options = Options::builder()
            .page_margin(50)
            .page_margin_top(100)
            .page_margin_bottom(100)
            .page_margin_left(75)
            .page_margin_right(75)
            .build();

        assert_eq!(options.page_margin, Some(50));
        assert_eq!(options.page_margin_top, Some(100));
        assert_eq!(options.page_margin_bottom, Some(100));
        assert_eq!(options.page_margin_left, Some(75));
        assert_eq!(options.page_margin_right, Some(75));
    }

    #[test]
    fn test_options_builder_svg_options() {
        let options = Options::builder()
            .svg_xml_declaration(false)
            .svg_bounding_boxes(true)
            .svg_view_box(true)
            .svg_remove_xlink(true)
            .svg_css("svg { background: white; }")
            .svg_format_raw(false)
            .svg_font_face_include(true)
            .build();

        assert_eq!(options.svg_xml_declaration, Some(false));
        assert_eq!(options.svg_bounding_boxes, Some(true));
        assert_eq!(options.svg_view_box, Some(true));
        assert_eq!(options.svg_remove_xlink, Some(true));
        assert_eq!(
            options.svg_css,
            Some("svg { background: white; }".to_string())
        );
        assert_eq!(options.svg_format_raw, Some(false));
        assert_eq!(options.svg_font_face_include, Some(true));
    }

    #[test]
    fn test_options_builder_midi_options() {
        let options = Options::builder().midi_tempo(120.0).midi_velocity(80).build();

        assert_eq!(options.midi_tempo, Some(120.0));
        assert_eq!(options.midi_velocity, Some(80));
    }

    #[test]
    fn test_options_builder_spacing_options() {
        let options = Options::builder()
            .spacing_staff(12)
            .spacing_system(6)
            .spacing_linear(0.25)
            .spacing_non_linear(0.6)
            .build();

        assert_eq!(options.spacing_staff, Some(12));
        assert_eq!(options.spacing_system, Some(6));
        assert_eq!(options.spacing_linear, Some(0.25));
        assert_eq!(options.spacing_non_linear, Some(0.6));
    }

    #[test]
    fn test_options_builder_transposition() {
        let options = Options::builder()
            .transpose("M2")
            .transpose_selected_only(true)
            .transpose_to_sounding_pitch(false)
            .build();

        assert_eq!(options.transpose, Some("M2".to_string()));
        assert_eq!(options.transpose_selected_only, Some(true));
        assert_eq!(options.transpose_to_sounding_pitch, Some(false));
    }

    #[test]
    fn test_options_round_trip_json() {
        let original = Options::builder()
            .scale(80)
            .page_width(2100)
            .page_height(2970)
            .font("Leipzig")
            .breaks(BreakMode::Smart)
            .build();

        let json = original.to_json().unwrap();
        let parsed = Options::from_json(&json).unwrap();

        assert_eq!(original.scale, parsed.scale);
        assert_eq!(original.page_width, parsed.page_width);
        assert_eq!(original.page_height, parsed.page_height);
        assert_eq!(original.font, parsed.font);
        assert_eq!(original.breaks, parsed.breaks);
    }
}
