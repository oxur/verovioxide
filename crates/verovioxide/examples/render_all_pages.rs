//! Render all pages of a music file to separate SVG files.
//!
//! This example demonstrates loading any supported music format (auto-detected)
//! and rendering all pages to separate SVG files with customizable page dimensions.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example render_all_pages -- input.mei [output-prefix]
//! ```
//!
//! # Arguments
//!
//! - `input`: Path to the input file (MEI, MusicXML, Humdrum, ABC, or PAE)
//! - `output-prefix`: Prefix for output files (optional, defaults to "page")
//!
//! # Output
//!
//! Files are named with the pattern: `{prefix}-001.svg`, `{prefix}-002.svg`, etc.
//!
//! # Example
//!
//! ```bash
//! cargo run --example render_all_pages -- score.mei score
//! # Produces: score-001.svg, score-002.svg, ...
//! ```

use std::env;
use std::fs;
use std::path::Path;

use verovioxide::{Options, Result, Toolkit};

fn main() -> Result<()> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <input> [output-prefix]", args[0]);
        eprintln!();
        eprintln!("Renders all pages of a music file to separate SVG files.");
        eprintln!();
        eprintln!("Arguments:");
        eprintln!("  input          Path to the input file (format auto-detected)");
        eprintln!("  output-prefix  Prefix for output files (default: page)");
        eprintln!();
        eprintln!("Supported formats: MEI, MusicXML, Humdrum, ABC, PAE");
        eprintln!();
        eprintln!("Output files will be named: <prefix>-001.svg, <prefix>-002.svg, etc.");
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    let output_prefix = if args.len() > 2 {
        args[2].clone()
    } else {
        "page".to_string()
    };

    println!("Creating Verovio toolkit with bundled resources...");

    // Create a toolkit with bundled resources
    let mut toolkit = Toolkit::new()?;

    // Print version information
    println!("Verovio version: {}", toolkit.version());

    // Configure rendering options with A4-like page dimensions
    // A4 in MEI units (roughly 210mm x 297mm at 10 units/mm)
    let options = Options::builder()
        .page_width(2100)
        .page_height(2970)
        .build();

    println!("Setting page dimensions: width=2100, height=2970 (A4-like)");
    toolkit.set_options(&options)?;

    // Load the input file (format is auto-detected)
    println!("Loading file: {} (format auto-detected)", input_path.display());
    toolkit.load_file(input_path)?;

    // Get page count
    let page_count = toolkit.page_count();
    println!("Document loaded successfully. Total pages: {}", page_count);

    if page_count == 0 {
        println!("Warning: Document has no pages to render.");
        return Ok(());
    }

    // Render all pages to separate SVG files
    println!("Rendering {} pages...", page_count);

    for page_num in 1..=page_count {
        // Generate output filename with zero-padded page number
        let output_file = format!("{}-{:03}.svg", output_prefix, page_num);

        // Render this page
        let svg = toolkit.render_to_svg(page_num)?;

        // Write to file
        fs::write(&output_file, &svg)?;

        println!(
            "  Page {}/{}: {} ({} bytes)",
            page_num,
            page_count,
            output_file,
            svg.len()
        );
    }

    println!("Done! Rendered {} pages.", page_count);

    Ok(())
}
