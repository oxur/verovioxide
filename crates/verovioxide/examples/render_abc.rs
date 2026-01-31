//! Render ABC notation to SVG.
//!
//! This example demonstrates using inline ABC notation to render music.
//! ABC notation is a text-based music notation format that is easy to
//! write and read.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example render_abc
//! ```
//!
//! # Output
//!
//! Produces `twinkle.svg` containing "Twinkle Twinkle Little Star".
//!
//! # About ABC Notation
//!
//! ABC notation is a shorthand for music notation. Key elements:
//! - `X:` - Reference number
//! - `T:` - Title
//! - `M:` - Meter (time signature)
//! - `L:` - Default note length
//! - `K:` - Key signature
//! - Notes: C D E F G A B c d e f g a b
//! - Octave: Capital = lower octave, lowercase = higher octave
//! - Duration: `/2` = half, `2` = double, etc.
//!
//! For more information about ABC notation, see: <https://abcnotation.com/>

use std::fs;

use verovioxide::{Options, Result, Toolkit};

fn main() -> Result<()> {
    println!("Creating Verovio toolkit with bundled resources...");

    // Create a toolkit with bundled resources
    let mut toolkit = Toolkit::new()?;

    // Print version information
    println!("Verovio version: {}", toolkit.version());

    // Configure rendering options for a nice output
    let options = Options::builder()
        .scale(60)
        .adjust_page_height(true)
        .page_width(2100)
        .build();

    println!("Setting rendering options...");
    toolkit.set_options(&options)?;

    // ABC notation for "Twinkle Twinkle Little Star"
    // This is the classic children's melody in C major
    let abc = r#"X:1
T:Twinkle Twinkle Little Star
C:Traditional
M:4/4
L:1/4
Q:1/4=100
K:C
%%MIDI program 0
|: C C G G | A A G2 | F F E E | D D C2 :|
|: G G F F | E E D2 | G G F F | E E D2 :|
|: C C G G | A A G2 | F F E E | D D C2 :|
"#;

    println!("Loading ABC notation for 'Twinkle Twinkle Little Star'...");
    println!();
    println!("ABC notation:");
    println!("---");
    for line in abc.lines() {
        println!("  {}", line);
    }
    println!("---");
    println!();

    // Load the ABC notation
    toolkit.load_data(abc)?;

    // Get page count
    let page_count = toolkit.page_count();
    println!("Document loaded successfully. Page count: {}", page_count);

    // Render to SVG
    println!("Rendering to SVG...");
    let svg = toolkit.render_to_svg(1)?;

    // Write output file
    let output_file = "twinkle.svg";
    println!("Writing SVG to: {}", output_file);
    fs::write(output_file, &svg)?;

    println!("Done! Rendered {} bytes of SVG.", svg.len());
    println!();
    println!("Open {} in a web browser or SVG viewer to see the result.", output_file);

    Ok(())
}
