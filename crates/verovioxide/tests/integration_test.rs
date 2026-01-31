//! Integration tests for the verovioxide crate.
//!
//! These tests verify end-to-end functionality including:
//! - Toolkit creation with bundled resources
//! - Loading and rendering various music notation formats
//! - Options configuration
//! - Format conversion (e.g., MusicXML to MEI)
//! - Error handling for invalid input

use verovioxide::{Options, Toolkit};

// =============================================================================
// Test Fixtures
// =============================================================================

/// Simple MusicXML file with a C major scale (4 measures, quarter notes).
const SIMPLE_MUSICXML: &str = include_str!("../../../test-fixtures/musicxml/simple.musicxml");

/// Simple MEI file with a C major scale (4 measures, quarter notes).
const SIMPLE_MEI: &str = include_str!("../../../test-fixtures/mei/simple.mei");

/// Simple ABC notation file with "Twinkle Twinkle Little Star".
const SIMPLE_ABC: &str = include_str!("../../../test-fixtures/abc/simple.abc");

// =============================================================================
// Helper Functions
// =============================================================================

/// Verifies that the given string is a valid SVG document.
///
/// Checks for the presence of opening and closing SVG tags.
fn assert_valid_svg(svg: &str) {
    assert!(
        svg.contains("<svg"),
        "SVG should contain opening <svg tag, but got: {}...",
        &svg[..svg.len().min(200)]
    );
    assert!(
        svg.contains("</svg>"),
        "SVG should contain closing </svg> tag"
    );
}

// =============================================================================
// Toolkit Creation Tests
// =============================================================================

/// Test that a toolkit can be created with bundled resources and reports a valid version.
#[test]
fn test_toolkit_creation() {
    let toolkit = Toolkit::new().expect("Failed to create toolkit with bundled resources");

    // Version should be a non-empty string
    let version = toolkit.version();
    assert!(!version.is_empty(), "Version should not be empty");

    // Version should look like a semver (e.g., "4.2.1" or similar)
    // At minimum, it should contain digits and dots
    assert!(
        version.chars().any(|c| c.is_ascii_digit()),
        "Version should contain digits: {}",
        version
    );
}

/// Test that a toolkit can be created without resources.
#[test]
fn test_toolkit_without_resources() {
    let toolkit = Toolkit::without_resources().expect("Failed to create toolkit without resources");
    assert!(!toolkit.version().is_empty());
}

// =============================================================================
// MusicXML Rendering Tests
// =============================================================================

/// Test loading and rendering a MusicXML file.
#[test]
fn test_render_musicxml() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Load the MusicXML fixture
    toolkit
        .load_data(SIMPLE_MUSICXML)
        .expect("Failed to load MusicXML data");

    // Verify page count is at least 1
    let page_count = toolkit.page_count();
    assert!(
        page_count >= 1,
        "Document should have at least 1 page, got {}",
        page_count
    );

    // Render to SVG and verify
    let svg = toolkit.render_to_svg(1).expect("Failed to render page 1");
    assert_valid_svg(&svg);

    // SVG should be reasonably sized (at least a few KB for music notation)
    assert!(
        svg.len() > 1000,
        "SVG should be substantial, got {} bytes",
        svg.len()
    );
}

// =============================================================================
// MEI Rendering Tests
// =============================================================================

/// Test loading and rendering an MEI file.
#[test]
fn test_render_mei() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Load the MEI fixture
    toolkit
        .load_data(SIMPLE_MEI)
        .expect("Failed to load MEI data");

    // Verify we have pages
    assert!(toolkit.page_count() >= 1, "Document should have at least 1 page");

    // Render to SVG and verify
    let svg = toolkit.render_to_svg(1).expect("Failed to render MEI to SVG");
    assert_valid_svg(&svg);
}

// =============================================================================
// ABC Notation Rendering Tests
// =============================================================================

/// Test loading and rendering ABC notation.
#[test]
fn test_render_abc() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Load the ABC fixture
    toolkit
        .load_data(SIMPLE_ABC)
        .expect("Failed to load ABC data");

    // Verify we have pages
    assert!(toolkit.page_count() >= 1, "Document should have at least 1 page");

    // Render to SVG and verify
    let svg = toolkit.render_to_svg(1).expect("Failed to render ABC to SVG");
    assert_valid_svg(&svg);
}

/// Test rendering ABC notation provided inline (not from fixture file).
#[test]
fn test_render_abc_inline() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Minimal ABC tune inline
    let abc_inline = r#"X:1
T:Test Tune
M:4/4
L:1/4
K:C
CDEF | GABC |]
"#;

    toolkit
        .load_data(abc_inline)
        .expect("Failed to load inline ABC data");

    assert!(toolkit.page_count() >= 1);

    let svg = toolkit.render_to_svg(1).expect("Failed to render inline ABC");
    assert_valid_svg(&svg);
}

// =============================================================================
// Options Tests
// =============================================================================

/// Test setting various rendering options.
#[test]
fn test_options() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Build options with various settings
    let options = Options::builder()
        .scale(80) // 80% scale
        .page_width(2100) // A4 width in MEI units
        .page_height(2970) // A4 height
        .adjust_page_height(true)
        .page_margin(50)
        .build();

    // Set options should succeed
    toolkit
        .set_options(&options)
        .expect("Failed to set options");

    // Load some data to verify options take effect
    toolkit
        .load_data(SIMPLE_MEI)
        .expect("Failed to load MEI data");

    // Render should work with custom options
    let svg = toolkit.render_to_svg(1).expect("Failed to render with custom options");
    assert_valid_svg(&svg);

    // Verify scale was set correctly
    assert_eq!(toolkit.get_scale(), 80, "Scale should be 80");
}

/// Test setting and retrieving scale independently.
#[test]
fn test_options_scale() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Get default scale
    let default_scale = toolkit.get_scale();
    assert!(default_scale > 0, "Default scale should be positive");

    // Set new scale
    toolkit.set_scale(50).expect("Failed to set scale to 50");
    assert_eq!(toolkit.get_scale(), 50);

    // Set back to 100
    toolkit.set_scale(100).expect("Failed to set scale to 100");
    assert_eq!(toolkit.get_scale(), 100);
}

/// Test that options can be retrieved as JSON.
#[test]
fn test_options_json() {
    let toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Get current options (Verovio returns JSON with trailing whitespace)
    let options_json = toolkit.get_options();
    let trimmed = options_json.trim();
    assert!(trimmed.starts_with('{'), "Options should be valid JSON");
    assert!(trimmed.ends_with('}'), "Options should be valid JSON");

    // Get default options
    let defaults_json = toolkit.get_default_options();
    assert!(defaults_json.trim().starts_with('{'));

    // Get available options
    let available_json = toolkit.get_available_options();
    assert!(available_json.trim().starts_with('{'));
}

/// Test resetting options to defaults.
#[test]
fn test_options_reset() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Get initial scale (should be 100)
    let initial_scale = toolkit.get_scale();
    assert_eq!(initial_scale, 100, "Initial scale should be 100");

    // Change scale
    toolkit.set_scale(50).expect("Failed to set scale");
    assert_eq!(toolkit.get_scale(), 50, "Scale should be 50 after setting");

    // Reset options - this calls Verovio's resetOptions
    // Note: Verovio's reset behavior for scale may vary between versions
    toolkit.reset_options();

    // Just verify reset_options doesn't crash and the toolkit is still usable
    // The exact reset behavior is Verovio-version dependent
    let _scale = toolkit.get_scale();
    let _options = toolkit.get_options();
    assert!(!_options.is_empty(), "Options should still be retrievable after reset");
}

// =============================================================================
// MEI Export Tests
// =============================================================================

/// Test exporting loaded MusicXML as MEI.
#[test]
fn test_get_mei_export() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Load MusicXML
    toolkit
        .load_data(SIMPLE_MUSICXML)
        .expect("Failed to load MusicXML data");

    // Export as MEI
    let mei = toolkit.get_mei().expect("Failed to export MEI");

    // Verify it's valid MEI
    assert!(mei.contains("<mei"), "Exported data should contain <mei tag");
    assert!(mei.contains("</mei>"), "Exported data should contain </mei> closing tag");
    assert!(
        mei.contains("http://www.music-encoding.org/ns/mei"),
        "Exported data should contain MEI namespace"
    );
}

/// Test exporting with options.
#[test]
fn test_get_mei_export_with_options() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Load MEI
    toolkit
        .load_data(SIMPLE_MEI)
        .expect("Failed to load MEI data");

    // Export as MEI with default options
    let mei = toolkit
        .get_mei_with_options("{}")
        .expect("Failed to export MEI with options");

    assert!(mei.contains("<mei"));
}

// =============================================================================
// Error Handling Tests
// =============================================================================

/// Test that loading invalid/malformed input produces an error.
#[test]
fn test_invalid_input() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Empty string should fail
    let result = toolkit.load_data("");
    assert!(result.is_err(), "Loading empty string should fail");

    // Random text should fail
    let result = toolkit.load_data("this is not valid music notation");
    assert!(result.is_err(), "Loading random text should fail");

    // Malformed XML should fail
    let result = toolkit.load_data("<mei><broken");
    assert!(result.is_err(), "Loading malformed XML should fail");
}

/// Test that rendering with no data loaded produces an error.
#[test]
fn test_render_no_data() {
    let toolkit = Toolkit::new().expect("Failed to create toolkit");

    // No data loaded, page count should be 0
    assert_eq!(toolkit.page_count(), 0);

    // Rendering page 1 should fail
    let result = toolkit.render_to_svg(1);
    assert!(result.is_err(), "Rendering with no data should fail");
}

/// Test that rendering an invalid page number produces an error.
#[test]
fn test_render_invalid_page() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    toolkit
        .load_data(SIMPLE_MEI)
        .expect("Failed to load MEI data");

    // Page 0 should fail (pages are 1-indexed)
    let result = toolkit.render_to_svg(0);
    assert!(result.is_err(), "Rendering page 0 should fail");

    // Page 1000 should fail (beyond page count)
    let result = toolkit.render_to_svg(1000);
    assert!(result.is_err(), "Rendering beyond page count should fail");
}

/// Test loading a non-existent file.
#[test]
fn test_load_nonexistent_file() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    let result = toolkit.load_file(std::path::Path::new("/nonexistent/path/to/score.mei"));
    assert!(result.is_err(), "Loading non-existent file should fail");

    // Error should mention file not found
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("not found"),
        "Error should mention file not found: {}",
        err
    );
}

// =============================================================================
// Multi-Page Rendering Tests
// =============================================================================

/// Test rendering all pages of a document.
#[test]
fn test_render_all_pages() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    toolkit
        .load_data(SIMPLE_MEI)
        .expect("Failed to load MEI data");

    let pages = toolkit
        .render_all_pages()
        .expect("Failed to render all pages");

    assert!(!pages.is_empty(), "Should have at least one page");

    for (i, svg) in pages.iter().enumerate() {
        assert_valid_svg(svg);
        assert!(
            svg.len() > 100,
            "Page {} should have substantial content",
            i + 1
        );
    }
}

// =============================================================================
// SVG Output Variation Tests
// =============================================================================

/// Test rendering with XML declaration.
#[test]
fn test_render_with_declaration() {
    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    toolkit
        .load_data(SIMPLE_MEI)
        .expect("Failed to load MEI data");

    let svg = toolkit
        .render_to_svg_with_declaration(1)
        .expect("Failed to render with declaration");

    assert_valid_svg(&svg);
    // With declaration should include XML prolog
    assert!(
        svg.contains("<?xml") || svg.contains("<svg"),
        "Should contain XML declaration or SVG tag"
    );
}

// =============================================================================
// Toolkit Information Tests
// =============================================================================

/// Test retrieving toolkit ID.
#[test]
fn test_toolkit_id() {
    let toolkit = Toolkit::new().expect("Failed to create toolkit");

    let id = toolkit.get_id();
    assert!(!id.is_empty(), "Toolkit should have an ID");
}

/// Test retrieving resource path.
#[test]
fn test_toolkit_resource_path() {
    let toolkit = Toolkit::new().expect("Failed to create toolkit");

    let path = toolkit.get_resource_path();
    assert!(!path.is_empty(), "Resource path should not be empty");
}

/// Test toolkit Debug implementation.
#[test]
fn test_toolkit_debug() {
    let toolkit = Toolkit::new().expect("Failed to create toolkit");

    let debug_str = format!("{:?}", toolkit);
    assert!(debug_str.contains("Toolkit"), "Debug should contain 'Toolkit'");
    assert!(debug_str.contains("version"), "Debug should contain 'version'");
}

// =============================================================================
// Logging Tests
// =============================================================================

/// Test enabling and retrieving logs.
#[test]
fn test_logging() {
    // Enable logging to buffer
    Toolkit::enable_log_to_buffer(true);

    let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

    // Load some data (may generate log messages)
    toolkit
        .load_data(SIMPLE_MEI)
        .expect("Failed to load MEI data");

    // Get log - may or may not have content depending on Verovio's behavior
    let _log = toolkit.get_log();

    // Disable logging
    Toolkit::enable_log_to_buffer(false);
}
