//! Unified query API with trait-based `get()` method.
//!
//! This module provides a consistent, type-safe API for querying element
//! information from loaded music documents. Each query type specifies its
//! return type, enabling compile-time type checking.
//!
//! # Element Queries
//!
//! Query information about specific elements by their xml:id:
//!
//! ```no_run
//! use verovioxide::{Toolkit, Page, Attrs, Time, Times};
//!
//! let mut voxide = Toolkit::new().unwrap();
//! voxide.load("score.mei").unwrap();
//!
//! // Get page containing an element
//! let page: u32 = voxide.get(Page::of("note-001")).unwrap();
//!
//! // Get element attributes as JSON
//! let attrs: String = voxide.get(Attrs::of("note-001")).unwrap();
//!
//! // Get timing information
//! let time: f64 = voxide.get(Time::of("note-001")).unwrap();
//! let times: String = voxide.get(Times::of("note-001")).unwrap();
//! ```
//!
//! # Time-Based Queries
//!
//! Query elements at a specific time:
//!
//! ```no_run
//! use verovioxide::{Toolkit, Elements};
//!
//! let mut voxide = Toolkit::new().unwrap();
//! voxide.load("score.mei").unwrap();
//!
//! // Get elements sounding at 5000ms
//! let elements: String = voxide.get(Elements::at(5000)).unwrap();
//! ```
//!
//! # Descriptive Features
//!
//! Get descriptive features of the document:
//!
//! ```no_run
//! use verovioxide::{Toolkit, Features};
//!
//! let mut voxide = Toolkit::new().unwrap();
//! voxide.load("score.mei").unwrap();
//!
//! let features: String = voxide.get(Features).unwrap();
//! ```

use crate::{Result, Toolkit};

// =============================================================================
// Trait
// =============================================================================

/// Trait for queries with type-safe output.
///
/// Each query type implements this trait, specifying its output type
/// (e.g., `u32` for page numbers, `f64` for time, `String` for JSON).
pub trait QueryOutput {
    /// The type returned by this query.
    type Output;

    /// Execute the query using the given toolkit.
    fn query(self, toolkit: &Toolkit) -> Result<Self::Output>;
}

// =============================================================================
// Element-Based Query Types
// =============================================================================

/// Query for the page containing an element.
///
/// Returns the 1-based page number.
///
/// # Example
///
/// ```no_run
/// use verovioxide::{Toolkit, Page};
///
/// let mut voxide = Toolkit::new().unwrap();
/// voxide.load("score.mei").unwrap();
///
/// let page: u32 = voxide.get(Page::of("note-001")).unwrap();
/// println!("Element is on page {}", page);
/// ```
#[derive(Debug, Clone)]
pub struct Page<'a> {
    xml_id: &'a str,
}

impl<'a> Page<'a> {
    /// Create a page query for the given element ID.
    pub fn of(xml_id: &'a str) -> Self {
        Self { xml_id }
    }
}

impl<'a> QueryOutput for Page<'a> {
    type Output = u32;

    fn query(self, toolkit: &Toolkit) -> Result<u32> {
        toolkit.get_page_with_element(self.xml_id)
    }
}

/// Query for element attributes as JSON.
///
/// # Example
///
/// ```no_run
/// use verovioxide::{Toolkit, Attrs};
///
/// let mut voxide = Toolkit::new().unwrap();
/// voxide.load("score.mei").unwrap();
///
/// let attrs: String = voxide.get(Attrs::of("note-001")).unwrap();
/// println!("Attributes: {}", attrs);
/// ```
#[derive(Debug, Clone)]
pub struct Attrs<'a> {
    xml_id: &'a str,
}

impl<'a> Attrs<'a> {
    /// Create an attributes query for the given element ID.
    pub fn of(xml_id: &'a str) -> Self {
        Self { xml_id }
    }
}

impl<'a> QueryOutput for Attrs<'a> {
    type Output = String;

    fn query(self, toolkit: &Toolkit) -> Result<String> {
        toolkit.get_element_attr(self.xml_id)
    }
}

/// Query for element time in milliseconds.
///
/// Returns the onset time of the element.
///
/// # Example
///
/// ```no_run
/// use verovioxide::{Toolkit, Time};
///
/// let mut voxide = Toolkit::new().unwrap();
/// voxide.load("score.mei").unwrap();
///
/// let time: f64 = voxide.get(Time::of("note-001")).unwrap();
/// println!("Element starts at {} ms", time);
/// ```
#[derive(Debug, Clone)]
pub struct Time<'a> {
    xml_id: &'a str,
}

impl<'a> Time<'a> {
    /// Create a time query for the given element ID.
    pub fn of(xml_id: &'a str) -> Self {
        Self { xml_id }
    }
}

impl<'a> QueryOutput for Time<'a> {
    type Output = f64;

    fn query(self, toolkit: &Toolkit) -> Result<f64> {
        toolkit.get_time_for_element(self.xml_id)
    }
}

/// Query for element times as JSON array.
///
/// Returns all times associated with the element (for elements with duration).
///
/// # Example
///
/// ```no_run
/// use verovioxide::{Toolkit, Times};
///
/// let mut voxide = Toolkit::new().unwrap();
/// voxide.load("score.mei").unwrap();
///
/// let times: String = voxide.get(Times::of("note-001")).unwrap();
/// println!("Times: {}", times);
/// ```
#[derive(Debug, Clone)]
pub struct Times<'a> {
    xml_id: &'a str,
}

impl<'a> Times<'a> {
    /// Create a times query for the given element ID.
    pub fn of(xml_id: &'a str) -> Self {
        Self { xml_id }
    }
}

impl<'a> QueryOutput for Times<'a> {
    type Output = String;

    fn query(self, toolkit: &Toolkit) -> Result<String> {
        toolkit.get_times_for_element(self.xml_id)
    }
}

/// Query for expansion IDs associated with an element.
///
/// Used with documents containing repeats or other expansion elements.
///
/// # Example
///
/// ```no_run
/// use verovioxide::{Toolkit, ExpansionIds};
///
/// let mut voxide = Toolkit::new().unwrap();
/// voxide.load("score.mei").unwrap();
///
/// let ids: String = voxide.get(ExpansionIds::of("note-001")).unwrap();
/// println!("Expansion IDs: {}", ids);
/// ```
#[derive(Debug, Clone)]
pub struct ExpansionIds<'a> {
    xml_id: &'a str,
}

impl<'a> ExpansionIds<'a> {
    /// Create an expansion IDs query for the given element ID.
    pub fn of(xml_id: &'a str) -> Self {
        Self { xml_id }
    }
}

impl<'a> QueryOutput for ExpansionIds<'a> {
    type Output = String;

    fn query(self, toolkit: &Toolkit) -> Result<String> {
        toolkit.get_expansion_ids_for_element(self.xml_id)
    }
}

/// Query for MIDI values associated with an element.
///
/// Returns pitch, velocity, and other MIDI information.
///
/// # Example
///
/// ```no_run
/// use verovioxide::{Toolkit, MidiValues};
///
/// let mut voxide = Toolkit::new().unwrap();
/// voxide.load("score.mei").unwrap();
///
/// let midi: String = voxide.get(MidiValues::of("note-001")).unwrap();
/// println!("MIDI values: {}", midi);
/// ```
#[derive(Debug, Clone)]
pub struct MidiValues<'a> {
    xml_id: &'a str,
}

impl<'a> MidiValues<'a> {
    /// Create a MIDI values query for the given element ID.
    pub fn of(xml_id: &'a str) -> Self {
        Self { xml_id }
    }
}

impl<'a> QueryOutput for MidiValues<'a> {
    type Output = String;

    fn query(self, toolkit: &Toolkit) -> Result<String> {
        toolkit.get_midi_values_for_element(self.xml_id)
    }
}

/// Query for the notated ID of an element.
///
/// Returns the original notated element ID (before expansion).
///
/// # Example
///
/// ```no_run
/// use verovioxide::{Toolkit, NotatedId};
///
/// let mut voxide = Toolkit::new().unwrap();
/// voxide.load("score.mei").unwrap();
///
/// let notated: String = voxide.get(NotatedId::of("note-001")).unwrap();
/// println!("Notated ID: {}", notated);
/// ```
#[derive(Debug, Clone)]
pub struct NotatedId<'a> {
    xml_id: &'a str,
}

impl<'a> NotatedId<'a> {
    /// Create a notated ID query for the given element ID.
    pub fn of(xml_id: &'a str) -> Self {
        Self { xml_id }
    }
}

impl<'a> QueryOutput for NotatedId<'a> {
    type Output = String;

    fn query(self, toolkit: &Toolkit) -> Result<String> {
        toolkit.get_notated_id_for_element(self.xml_id)
    }
}

// =============================================================================
// Time-Based Query Types
// =============================================================================

/// Query for elements at a specific time.
///
/// Returns JSON with element IDs sounding at the given time.
///
/// # Example
///
/// ```no_run
/// use verovioxide::{Toolkit, Elements};
///
/// let mut voxide = Toolkit::new().unwrap();
/// voxide.load("score.mei").unwrap();
///
/// // Get elements at 5 seconds
/// let elements: String = voxide.get(Elements::at(5000)).unwrap();
/// println!("Elements at 5s: {}", elements);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Elements {
    millisec: i32,
}

impl Elements {
    /// Create a query for elements at the given time in milliseconds.
    pub fn at(millisec: i32) -> Self {
        Self { millisec }
    }
}

impl QueryOutput for Elements {
    type Output = String;

    fn query(self, toolkit: &Toolkit) -> Result<String> {
        toolkit.get_elements_at_time(self.millisec)
    }
}

// =============================================================================
// Descriptive Features
// =============================================================================

/// Query for descriptive features of the document.
///
/// Returns JSON with various document features.
///
/// # Example
///
/// ```no_run
/// use verovioxide::{Toolkit, Features};
///
/// let mut voxide = Toolkit::new().unwrap();
/// voxide.load("score.mei").unwrap();
///
/// let features: String = voxide.get(Features).unwrap();
/// println!("Features: {}", features);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Features;

impl Features {
    /// Create a features query with custom options.
    pub fn with_options() -> FeaturesOptionsBuilder {
        FeaturesOptionsBuilder::default()
    }
}

impl QueryOutput for Features {
    type Output = String;

    fn query(self, toolkit: &Toolkit) -> Result<String> {
        toolkit.get_descriptive_features(None)
    }
}

/// Builder for descriptive features options.
#[derive(Debug, Clone, Default)]
pub struct FeaturesOptionsBuilder {
    options: Vec<(String, String)>,
}

impl FeaturesOptionsBuilder {
    /// Add a custom option.
    pub fn option(mut self, key: &str, value: &str) -> Self {
        self.options.push((key.to_string(), value.to_string()));
        self
    }

    /// Build the options JSON string.
    fn to_json(&self) -> String {
        if self.options.is_empty() {
            "{}".to_string()
        } else {
            let parts: Vec<String> = self
                .options
                .iter()
                .map(|(k, v)| format!("\"{}\":\"{}\"", k, v))
                .collect();
            format!("{{{}}}", parts.join(","))
        }
    }
}

impl QueryOutput for FeaturesOptionsBuilder {
    type Output = String;

    fn query(self, toolkit: &Toolkit) -> Result<String> {
        toolkit.get_descriptive_features(Some(&self.to_json()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_of() {
        let query = Page::of("note-001");
        assert_eq!(query.xml_id, "note-001");
    }

    #[test]
    fn test_attrs_of() {
        let query = Attrs::of("note-001");
        assert_eq!(query.xml_id, "note-001");
    }

    #[test]
    fn test_time_of() {
        let query = Time::of("note-001");
        assert_eq!(query.xml_id, "note-001");
    }

    #[test]
    fn test_times_of() {
        let query = Times::of("note-001");
        assert_eq!(query.xml_id, "note-001");
    }

    #[test]
    fn test_expansion_ids_of() {
        let query = ExpansionIds::of("note-001");
        assert_eq!(query.xml_id, "note-001");
    }

    #[test]
    fn test_midi_values_of() {
        let query = MidiValues::of("note-001");
        assert_eq!(query.xml_id, "note-001");
    }

    #[test]
    fn test_notated_id_of() {
        let query = NotatedId::of("note-001");
        assert_eq!(query.xml_id, "note-001");
    }

    #[test]
    fn test_elements_at() {
        let query = Elements::at(5000);
        assert_eq!(query.millisec, 5000);
    }

    #[test]
    fn test_features_options_empty() {
        let builder = FeaturesOptionsBuilder::default();
        assert_eq!(builder.to_json(), "{}");
    }

    #[test]
    fn test_features_options_with_values() {
        let builder = Features::with_options()
            .option("key1", "value1")
            .option("key2", "value2");
        let json = builder.to_json();
        assert!(json.contains("\"key1\":\"value1\""));
        assert!(json.contains("\"key2\":\"value2\""));
    }

    #[test]
    fn test_query_types_are_send() {
        fn assert_send<T: Send>() {}

        assert_send::<Page<'_>>();
        assert_send::<Attrs<'_>>();
        assert_send::<Time<'_>>();
        assert_send::<Times<'_>>();
        assert_send::<ExpansionIds<'_>>();
        assert_send::<MidiValues<'_>>();
        assert_send::<NotatedId<'_>>();
        assert_send::<Elements>();
        assert_send::<Features>();
        assert_send::<FeaturesOptionsBuilder>();
    }
}
