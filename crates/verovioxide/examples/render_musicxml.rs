//! Render a MusicXML file to SVG.
//!
//! This example demonstrates loading a MusicXML file and rendering it to SVG format
//! using the Verovioxide library with bundled resources.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example render_musicxml -- input.musicxml [output.svg]
//! ```
//!
//! # Arguments
//!
//! - `input.musicxml`: Path to the input MusicXML file (required)
//! - `output.svg`: Path for the output SVG file (optional, defaults to "output.svg")
//!
//! # Example
//!
//! ```bash
//! cargo run --example render_musicxml -- examples/data/score.musicxml rendered.svg
//! ```

use std::env;
use std::fs;
use std::path::Path;

use verovioxide::{Options, Result, Toolkit};

fn main() -> Result<()> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <input.musicxml> [output.svg]", args[0]);
        eprintln!();
        eprintln!("Renders a MusicXML file to SVG format.");
        eprintln!();
        eprintln!("Arguments:");
        eprintln!("  input.musicxml  Path to the input MusicXML file");
        eprintln!("  output.svg      Path for the output SVG file (default: output.svg)");
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    let output_path = if args.len() > 2 {
        args[2].clone()
    } else {
        "output.svg".to_string()
    };

    println!("Creating Verovio toolkit with bundled resources...");

    // Create a toolkit with bundled resources (fonts, etc.)
    let mut toolkit = Toolkit::new()?;

    // Print version information
    println!("Verovio version: {}", toolkit.version());

    // Configure rendering options
    // - scale: 40% of default size for a compact rendering
    // - adjust_page_height: true to fit content to page height
    let options = Options::builder()
        .scale(40)
        .adjust_page_height(true)
        .build();

    println!("Setting rendering options: scale=40, adjust_page_height=true");
    toolkit.set_options(&options)?;

    // Load the MusicXML file
    println!("Loading MusicXML file: {}", input_path.display());
    toolkit.load_file(input_path)?;

    // Get and print page count
    let page_count = toolkit.page_count();
    println!("Document loaded successfully. Page count: {}", page_count);

    // Render page 1 to SVG
    println!("Rendering page 1 to SVG...");
    let svg = toolkit.render_to_svg(1)?;

    // Write the output
    println!("Writing SVG to: {}", output_path);
    fs::write(&output_path, &svg)?;

    println!("Done! Rendered {} bytes of SVG.", svg.len());

    Ok(())
}
