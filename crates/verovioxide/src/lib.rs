//! Safe Rust bindings to the Verovio music notation engraving library.
//!
//! Verovioxide provides a safe, idiomatic Rust API for [Verovio](https://www.verovio.org/),
//! a lightweight open-source library for engraving Music Encoding Initiative (MEI)
//! music scores as SVG.
//!
//! # Features
//!
//! - **Safe API**: All FFI calls are wrapped in safe Rust functions with proper error handling
//! - **Builder Pattern**: Fluent API for configuring rendering options
//! - **Multiple Formats**: Load MEI, MusicXML, Humdrum, ABC, and Plaine & Easie
//! - **SVG Output**: Render music notation as scalable vector graphics
//! - **Bundled Resources**: Optional embedded SMuFL fonts (enabled by default)
//!
//! # Quick Start
//!
//! ```no_run
//! use verovioxide::{Toolkit, Options, Result};
//!
//! fn main() -> Result<()> {
//!     // Create a toolkit with bundled resources
//!     let mut toolkit = Toolkit::new()?;
//!
//!     // Load MEI data
//!     let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
//!     <mei xmlns="http://www.music-encoding.org/ns/mei">
//!       <music><body><mdiv><score>
//!         <scoreDef><staffGrp>
//!           <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
//!         </staffGrp></scoreDef>
//!         <section><measure><staff n="1"><layer n="1">
//!           <note pname="c" oct="4" dur="4"/>
//!         </layer></staff></measure></section>
//!       </score></mdiv></body></music>
//!     </mei>"#;
//!
//!     toolkit.load_data(mei)?;
//!
//!     // Configure rendering options
//!     let options = Options::builder()
//!         .scale(100)
//!         .adjust_page_height(true)
//!         .build();
//!     toolkit.set_options(&options)?;
//!
//!     // Render to SVG
//!     let svg = toolkit.render_to_svg(1)?;
//!     println!("Rendered {} bytes of SVG", svg.len());
//!
//!     Ok(())
//! }
//! ```
//!
//! # Feature Flags
//!
//! - `bundled-data` (default): Include bundled SMuFL fonts and resources. Disable this
//!   feature if you want to provide your own resource path.
//!
//! # Loading Music Data
//!
//! Verovio auto-detects the input format. Supported formats include:
//!
//! - **MEI**: Music Encoding Initiative XML
//! - **MusicXML**: Standard interchange format
//! - **Humdrum**: Text-based music representation
//! - **ABC**: Simple text-based notation
//! - **PAE**: Plaine & Easie Code (RISM)
//!
//! ```no_run
//! use verovioxide::Toolkit;
//!
//! let mut toolkit = Toolkit::new().unwrap();
//!
//! // Load from string
//! toolkit.load_data("<mei>...</mei>").unwrap();
//!
//! // Or load from file
//! use std::path::Path;
//! toolkit.load_file(Path::new("score.musicxml")).unwrap();
//! ```
//!
//! # Rendering Options
//!
//! Use the [`Options`] builder to configure rendering:
//!
//! ```
//! use verovioxide::{Options, BreakMode, HeaderMode};
//!
//! let options = Options::builder()
//!     .scale(80)                          // 80% scale
//!     .page_width(2100)                   // A4 width in MEI units
//!     .page_height(2970)                  // A4 height
//!     .adjust_page_height(true)           // Fit content
//!     .font("Bravura")                    // Use Bravura font
//!     .breaks(BreakMode::Auto)            // Automatic page breaks
//!     .header(HeaderMode::None)           // No header
//!     .build();
//! ```
//!
//! # Thread Safety
//!
//! [`Toolkit`] implements `Send` but not `Sync`. You can move a toolkit between
//! threads, but you cannot share references across threads. For concurrent rendering,
//! create separate toolkit instances.
//!
//! # Error Handling
//!
//! All fallible operations return [`Result<T, Error>`](Result). The [`Error`] type
//! provides detailed information about what went wrong.
//!
//! ```no_run
//! use verovioxide::{Toolkit, Error};
//!
//! let toolkit = Toolkit::new().unwrap();
//!
//! match toolkit.render_to_svg(1) {
//!     Ok(svg) => println!("Success!"),
//!     Err(Error::RenderError(msg)) => eprintln!("Render failed: {}", msg),
//!     Err(e) => eprintln!("Other error: {}", e),
//! }
//! ```
//!
//! # Low-Level Access
//!
//! For advanced use cases, the raw FFI bindings are available in the
//! [`verovioxide-sys`](https://docs.rs/verovioxide-sys) crate.

mod error;
mod options;
mod toolkit;

pub use error::{Error, Result};
pub use options::{BreakMode, CondenseMode, FooterMode, HeaderMode, Options, OptionsBuilder, TextFont};
pub use toolkit::Toolkit;

// Re-export data crate types when bundled-data feature is enabled
#[cfg(feature = "bundled-data")]
pub use verovioxide_data::{
    available_fonts, default_font, extract_resources, has_bravura, has_gootville, has_leland,
    has_leipzig, has_petaluma, resource_dir, DataError,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_api_types_available() {
        // Ensure all public types are accessible
        let _ = std::any::type_name::<Error>();
        let _ = std::any::type_name::<Options>();
        let _ = std::any::type_name::<OptionsBuilder>();
        let _ = std::any::type_name::<Toolkit>();
        let _ = std::any::type_name::<BreakMode>();
        let _ = std::any::type_name::<CondenseMode>();
        let _ = std::any::type_name::<FooterMode>();
        let _ = std::any::type_name::<HeaderMode>();
        let _ = std::any::type_name::<TextFont>();
    }

    #[test]
    fn test_result_type_alias() {
        fn example_function() -> Result<String> {
            Ok("test".to_string())
        }
        assert!(example_function().is_ok());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_bundled_data_exports() {
        // Ensure bundled-data re-exports are available
        let _ = default_font();
        let _ = available_fonts();
        let _ = has_leipzig();
        let _ = has_bravura();
    }
}
