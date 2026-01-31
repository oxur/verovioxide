//! Safe wrapper around the Verovio toolkit.
//!
//! This module provides a safe, idiomatic Rust wrapper around the Verovio C API.
//! The [`Toolkit`] struct manages the lifecycle of a Verovio toolkit instance
//! and provides methods for loading music data and rendering to various formats.
//!
//! # Example
//!
//! ```no_run
//! use verovioxide::{Toolkit, Options};
//!
//! // Create a toolkit with bundled resources
//! let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
//!
//! // Load MEI data
//! let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
//! <mei xmlns="http://www.music-encoding.org/ns/mei">
//!   <music><body><mdiv><score>
//!     <scoreDef><staffGrp>
//!       <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
//!     </staffGrp></scoreDef>
//!     <section><measure><staff n="1"><layer n="1">
//!       <note pname="c" oct="4" dur="4"/>
//!     </layer></staff></measure></section>
//!   </score></mdiv></body></music>
//! </mei>"#;
//!
//! toolkit.load_data(mei).expect("Failed to load MEI");
//!
//! // Configure rendering options
//! let options = Options::builder()
//!     .scale(100)
//!     .adjust_page_height(true)
//!     .build();
//! toolkit.set_options(&options).expect("Failed to set options");
//!
//! // Render to SVG
//! let svg = toolkit.render_to_svg(1).expect("Failed to render");
//! println!("{}", svg);
//! ```

use std::ffi::{CStr, CString, c_void};
use std::path::Path;

#[cfg(feature = "bundled-data")]
use tempfile::TempDir;

use crate::error::{Error, Result};
use crate::options::Options;

/// A safe wrapper around the Verovio toolkit.
///
/// This struct provides a safe, idiomatic interface to the Verovio music engraving library.
/// It manages the lifecycle of the underlying C++ toolkit and ensures proper cleanup.
///
/// # Thread Safety
///
/// `Toolkit` implements `Send` but not `Sync`. This means you can move a toolkit between
/// threads, but you cannot share references to it across threads. Each toolkit instance
/// has internal mutable state that is not thread-safe to access concurrently.
///
/// # Resource Management
///
/// When created with bundled resources (via [`Toolkit::new()`]), the toolkit extracts
/// resources to a temporary directory that is automatically cleaned up when the toolkit
/// is dropped.
pub struct Toolkit {
    /// Raw pointer to the Verovio toolkit instance.
    ptr: *mut c_void,

    /// Temporary directory holding extracted resources.
    /// Kept alive for the lifetime of the toolkit.
    #[cfg(feature = "bundled-data")]
    _temp_dir: Option<TempDir>,
}

// SAFETY: Toolkit can be sent between threads because:
// - The underlying Verovio toolkit pointer is owned exclusively
// - No references are shared across threads
// - The TempDir is also Send
unsafe impl Send for Toolkit {}

// NOTE: We intentionally do NOT implement Sync because:
// - The Verovio toolkit has internal mutable state
// - Concurrent access to the same toolkit is not safe
// - Users who need concurrent rendering should create separate toolkits

impl Drop for Toolkit {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            // SAFETY: ptr is valid and was created by a constructor function
            unsafe {
                verovioxide_sys::vrvToolkit_destructor(self.ptr);
            }
        }
    }
}

impl Toolkit {
    /// Creates a new toolkit with bundled resources.
    ///
    /// This extracts the embedded Verovio resources (fonts, etc.) to a temporary
    /// directory and initializes the toolkit to use them. The temporary directory
    /// is automatically cleaned up when the toolkit is dropped.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Resource extraction fails
    /// - Toolkit initialization fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// println!("Verovio version: {}", toolkit.version());
    /// ```
    #[cfg(feature = "bundled-data")]
    pub fn new() -> Result<Self> {
        let temp_dir = verovioxide_data::extract_resources()?;
        let resource_path = temp_dir.path();

        let path_str = resource_path.to_str().ok_or_else(|| {
            Error::InitializationError("resource path contains invalid UTF-8".into())
        })?;

        let c_path = CString::new(path_str)?;

        // SAFETY: c_path is a valid null-terminated string
        let ptr = unsafe { verovioxide_sys::vrvToolkit_constructorResourcePath(c_path.as_ptr()) };

        if ptr.is_null() {
            return Err(Error::InitializationError(
                "failed to create toolkit with resource path".into(),
            ));
        }

        Ok(Self {
            ptr,
            _temp_dir: Some(temp_dir),
        })
    }

    /// Creates a new toolkit with an explicit resource path.
    ///
    /// Use this when you have your own Verovio resources directory and don't want
    /// to use the bundled resources.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Verovio resources directory
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path contains invalid UTF-8
    /// - Toolkit initialization fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    /// use std::path::Path;
    ///
    /// let toolkit = Toolkit::with_resource_path(Path::new("/path/to/verovio/data"))
    ///     .expect("Failed to create toolkit");
    /// ```
    pub fn with_resource_path(path: &Path) -> Result<Self> {
        let path_str = path.to_str().ok_or_else(|| {
            Error::InitializationError("resource path contains invalid UTF-8".into())
        })?;

        let c_path = CString::new(path_str)?;

        // SAFETY: c_path is a valid null-terminated string
        let ptr = unsafe { verovioxide_sys::vrvToolkit_constructorResourcePath(c_path.as_ptr()) };

        if ptr.is_null() {
            return Err(Error::InitializationError(
                "failed to create toolkit with resource path".into(),
            ));
        }

        Ok(Self {
            ptr,
            #[cfg(feature = "bundled-data")]
            _temp_dir: None,
        })
    }

    /// Creates a new toolkit without loading any resources.
    ///
    /// This is useful for operations that don't require font resources, such as
    /// converting between formats or extracting metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if toolkit initialization fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
    /// println!("Verovio version: {}", toolkit.version());
    /// ```
    pub fn without_resources() -> Result<Self> {
        // SAFETY: This function has no preconditions
        let ptr = unsafe { verovioxide_sys::vrvToolkit_constructorNoResource() };

        if ptr.is_null() {
            return Err(Error::InitializationError(
                "failed to create toolkit without resources".into(),
            ));
        }

        Ok(Self {
            ptr,
            #[cfg(feature = "bundled-data")]
            _temp_dir: None,
        })
    }

    /// Loads music data from a string.
    ///
    /// The data format is auto-detected. Supported formats include:
    /// - MEI (Music Encoding Initiative)
    /// - MusicXML
    /// - Humdrum
    /// - Plaine & Easie Code (PAE)
    /// - ABC notation
    ///
    /// # Arguments
    ///
    /// * `data` - The music data as a string
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The data is malformed
    /// - The format is not recognized
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    ///
    /// let mei = r#"<mei xmlns="http://www.music-encoding.org/ns/mei">...</mei>"#;
    /// toolkit.load_data(mei).expect("Failed to load data");
    /// ```
    pub fn load_data(&mut self, data: &str) -> Result<()> {
        let c_data = CString::new(data)?;

        // SAFETY: ptr is valid, c_data is a valid null-terminated string
        let success = unsafe { verovioxide_sys::vrvToolkit_loadData(self.ptr, c_data.as_ptr()) };

        if success {
            Ok(())
        } else {
            Err(Error::LoadError(
                "failed to load data (check format and content)".into(),
            ))
        }
    }

    /// Loads music data from a file.
    ///
    /// The file format is auto-detected based on content.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the music file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file does not exist
    /// - The file cannot be read
    /// - The data is malformed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    /// use std::path::Path;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// toolkit.load_file(Path::new("score.mei")).expect("Failed to load file");
    /// ```
    pub fn load_file(&mut self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(Error::FileNotFound(path.to_path_buf()));
        }

        let path_str = path
            .to_str()
            .ok_or_else(|| Error::LoadError("file path contains invalid UTF-8".into()))?;

        let c_path = CString::new(path_str)?;

        // SAFETY: ptr is valid, c_path is a valid null-terminated string
        let success = unsafe { verovioxide_sys::vrvToolkit_loadFile(self.ptr, c_path.as_ptr()) };

        if success {
            Ok(())
        } else {
            Err(Error::LoadError(format!(
                "failed to load file: {}",
                path.display()
            )))
        }
    }

    /// Renders a page to SVG.
    ///
    /// Page numbers are 1-based. Use [`page_count()`](Self::page_count) to get the
    /// total number of pages.
    ///
    /// # Arguments
    ///
    /// * `page` - The page number to render (1-based)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No data has been loaded
    /// - The page number is out of range
    /// - Rendering fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load data ...
    ///
    /// let svg = toolkit.render_to_svg(1).expect("Failed to render");
    /// println!("{}", svg);
    /// ```
    pub fn render_to_svg(&self, page: u32) -> Result<String> {
        let page_count = self.page_count();
        if page == 0 || page > page_count {
            return Err(Error::RenderError(format!(
                "page {} out of range (document has {} pages)",
                page, page_count
            )));
        }

        // SAFETY: ptr is valid, page number is in range
        let svg_ptr =
            unsafe { verovioxide_sys::vrvToolkit_renderToSVG(self.ptr, page as i32, false) };

        self.ptr_to_string(svg_ptr)
            .ok_or_else(|| Error::RenderError("failed to render SVG".into()))
    }

    /// Renders a page to SVG with XML declaration.
    ///
    /// Same as [`render_to_svg`](Self::render_to_svg) but includes the XML declaration
    /// at the start of the SVG output.
    ///
    /// # Arguments
    ///
    /// * `page` - The page number to render (1-based)
    ///
    /// # Errors
    ///
    /// Returns an error if rendering fails.
    pub fn render_to_svg_with_declaration(&self, page: u32) -> Result<String> {
        let page_count = self.page_count();
        if page == 0 || page > page_count {
            return Err(Error::RenderError(format!(
                "page {} out of range (document has {} pages)",
                page, page_count
            )));
        }

        // SAFETY: ptr is valid, page number is in range
        let svg_ptr =
            unsafe { verovioxide_sys::vrvToolkit_renderToSVG(self.ptr, page as i32, true) };

        self.ptr_to_string(svg_ptr)
            .ok_or_else(|| Error::RenderError("failed to render SVG".into()))
    }

    /// Renders all pages to SVG.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No data has been loaded
    /// - Rendering any page fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load data ...
    ///
    /// let pages = toolkit.render_all_pages().expect("Failed to render");
    /// for (i, svg) in pages.iter().enumerate() {
    ///     println!("Page {}: {} bytes", i + 1, svg.len());
    /// }
    /// ```
    pub fn render_all_pages(&self) -> Result<Vec<String>> {
        let count = self.page_count();
        let mut pages = Vec::with_capacity(count as usize);

        for page in 1..=count {
            pages.push(self.render_to_svg(page)?);
        }

        Ok(pages)
    }

    /// Returns the number of pages in the loaded document.
    ///
    /// Returns 0 if no document is loaded.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load data ...
    ///
    /// println!("Document has {} pages", toolkit.page_count());
    /// ```
    #[must_use]
    pub fn page_count(&self) -> u32 {
        // SAFETY: ptr is valid
        let count = unsafe { verovioxide_sys::vrvToolkit_getPageCount(self.ptr) };
        count.max(0) as u32
    }

    /// Sets rendering options.
    ///
    /// Options are merged with existing options. To reset to defaults, use
    /// [`reset_options()`](Self::reset_options) first.
    ///
    /// # Arguments
    ///
    /// * `options` - The rendering options to set
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - JSON serialization fails
    /// - Option values are invalid
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::{Toolkit, Options};
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    ///
    /// let options = Options::builder()
    ///     .scale(80)
    ///     .adjust_page_height(true)
    ///     .build();
    ///
    /// toolkit.set_options(&options).expect("Failed to set options");
    /// ```
    pub fn set_options(&mut self, options: &Options) -> Result<()> {
        let json = options
            .to_json()
            .map_err(|e| Error::OptionsError(e.to_string()))?;

        let c_json = CString::new(json)?;

        // SAFETY: ptr is valid, c_json is a valid null-terminated string
        let success = unsafe { verovioxide_sys::vrvToolkit_setOptions(self.ptr, c_json.as_ptr()) };

        if success {
            Ok(())
        } else {
            Err(Error::OptionsError("failed to set options".into()))
        }
    }

    /// Gets the current options as a JSON string.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// let options_json = toolkit.get_options();
    /// println!("Current options: {}", options_json);
    /// ```
    #[must_use]
    pub fn get_options(&self) -> String {
        // SAFETY: ptr is valid
        let options_ptr = unsafe { verovioxide_sys::vrvToolkit_getOptions(self.ptr) };
        self.ptr_to_string(options_ptr).unwrap_or_default()
    }

    /// Gets the default options as a JSON string.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// let defaults = toolkit.get_default_options();
    /// println!("Default options: {}", defaults);
    /// ```
    #[must_use]
    pub fn get_default_options(&self) -> String {
        // SAFETY: ptr is valid
        let options_ptr = unsafe { verovioxide_sys::vrvToolkit_getDefaultOptions(self.ptr) };
        self.ptr_to_string(options_ptr).unwrap_or_default()
    }

    /// Gets all available options and their descriptions as a JSON string.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// let available = toolkit.get_available_options();
    /// println!("Available options: {}", available);
    /// ```
    #[must_use]
    pub fn get_available_options(&self) -> String {
        // SAFETY: ptr is valid
        let options_ptr = unsafe { verovioxide_sys::vrvToolkit_getAvailableOptions(self.ptr) };
        self.ptr_to_string(options_ptr).unwrap_or_default()
    }

    /// Resets all options to their default values.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// toolkit.reset_options();
    /// ```
    pub fn reset_options(&mut self) {
        // SAFETY: ptr is valid
        unsafe { verovioxide_sys::vrvToolkit_resetOptions(self.ptr) };
    }

    /// Returns the Verovio version string.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// println!("Verovio version: {}", toolkit.version());
    /// ```
    #[must_use]
    pub fn version(&self) -> String {
        // SAFETY: ptr is valid
        let version_ptr = unsafe { verovioxide_sys::vrvToolkit_getVersion(self.ptr) };
        self.ptr_to_string(version_ptr)
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Returns the log output from Verovio.
    ///
    /// Log output is only available if logging to buffer was enabled before
    /// loading data. Use [`enable_log_to_buffer()`](Self::enable_log_to_buffer)
    /// to enable it.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// Toolkit::enable_log_to_buffer(true);
    /// // ... load data ...
    /// let log = toolkit.get_log();
    /// println!("Verovio log: {}", log);
    /// ```
    #[must_use]
    pub fn get_log(&self) -> String {
        // SAFETY: ptr is valid
        let log_ptr = unsafe { verovioxide_sys::vrvToolkit_getLog(self.ptr) };
        self.ptr_to_string(log_ptr).unwrap_or_default()
    }

    /// Exports the loaded document as MEI.
    ///
    /// # Errors
    ///
    /// Returns an error if no document is loaded or export fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use verovioxide::Toolkit;
    ///
    /// let mut toolkit = Toolkit::new().expect("Failed to create toolkit");
    /// // ... load MusicXML or other format ...
    ///
    /// let mei = toolkit.get_mei().expect("Failed to export MEI");
    /// println!("{}", mei);
    /// ```
    pub fn get_mei(&self) -> Result<String> {
        self.get_mei_with_options("{}")
    }

    /// Exports the loaded document as MEI with options.
    ///
    /// # Arguments
    ///
    /// * `options` - JSON string with MEI export options
    ///
    /// # Errors
    ///
    /// Returns an error if no document is loaded or export fails.
    pub fn get_mei_with_options(&self, options: &str) -> Result<String> {
        let c_options = CString::new(options)?;

        // SAFETY: ptr is valid, c_options is a valid null-terminated string
        let mei_ptr = unsafe { verovioxide_sys::vrvToolkit_getMEI(self.ptr, c_options.as_ptr()) };

        self.ptr_to_string(mei_ptr)
            .ok_or_else(|| Error::RenderError("failed to export MEI".into()))
    }

    /// Exports the loaded document as Humdrum.
    ///
    /// # Errors
    ///
    /// Returns an error if no document is loaded or export fails.
    pub fn get_humdrum(&self) -> Result<String> {
        // SAFETY: ptr is valid
        let humdrum_ptr = unsafe { verovioxide_sys::vrvToolkit_getHumdrum(self.ptr) };

        self.ptr_to_string(humdrum_ptr)
            .ok_or_else(|| Error::RenderError("failed to export Humdrum".into()))
    }

    /// Renders the loaded document to MIDI as base64-encoded data.
    ///
    /// # Errors
    ///
    /// Returns an error if no document is loaded or rendering fails.
    pub fn render_to_midi(&self) -> Result<String> {
        if self.page_count() == 0 {
            return Err(Error::RenderError("no data loaded".into()));
        }

        // SAFETY: ptr is valid, data is loaded
        let midi_ptr = unsafe { verovioxide_sys::vrvToolkit_renderToMIDI(self.ptr) };

        self.ptr_to_string(midi_ptr)
            .ok_or_else(|| Error::RenderError("failed to render MIDI".into()))
    }

    /// Renders the loaded document to Plaine & Easie code.
    ///
    /// # Errors
    ///
    /// Returns an error if no document is loaded or rendering fails.
    pub fn render_to_pae(&self) -> Result<String> {
        if self.page_count() == 0 {
            return Err(Error::RenderError("no data loaded".into()));
        }

        // SAFETY: ptr is valid, data is loaded
        let pae_ptr = unsafe { verovioxide_sys::vrvToolkit_renderToPAE(self.ptr) };

        self.ptr_to_string(pae_ptr)
            .ok_or_else(|| Error::RenderError("failed to render PAE".into()))
    }

    /// Gets the timemap as JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if no document is loaded or export fails.
    pub fn render_to_timemap(&self) -> Result<String> {
        self.render_to_timemap_with_options("{}")
    }

    /// Gets the timemap as JSON with options.
    ///
    /// # Arguments
    ///
    /// * `options` - JSON string with timemap options
    ///
    /// # Errors
    ///
    /// Returns an error if no document is loaded or export fails.
    pub fn render_to_timemap_with_options(&self, options: &str) -> Result<String> {
        let c_options = CString::new(options)?;

        // SAFETY: ptr is valid, c_options is a valid null-terminated string
        let timemap_ptr =
            unsafe { verovioxide_sys::vrvToolkit_renderToTimemap(self.ptr, c_options.as_ptr()) };

        self.ptr_to_string(timemap_ptr)
            .ok_or_else(|| Error::RenderError("failed to render timemap".into()))
    }

    /// Gets the expansion map as JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if no document is loaded or export fails.
    pub fn render_to_expansion_map(&self) -> Result<String> {
        // SAFETY: ptr is valid
        let map_ptr = unsafe { verovioxide_sys::vrvToolkit_renderToExpansionMap(self.ptr) };

        self.ptr_to_string(map_ptr)
            .ok_or_else(|| Error::RenderError("failed to render expansion map".into()))
    }

    /// Gets the current rendering scale as a percentage.
    #[must_use]
    pub fn get_scale(&self) -> i32 {
        // SAFETY: ptr is valid
        unsafe { verovioxide_sys::vrvToolkit_getScale(self.ptr) }
    }

    /// Sets the rendering scale as a percentage.
    ///
    /// # Arguments
    ///
    /// * `scale` - Scale percentage (e.g., 100 for 100%)
    ///
    /// # Errors
    ///
    /// Returns an error if the scale value is invalid.
    pub fn set_scale(&mut self, scale: i32) -> Result<()> {
        // SAFETY: ptr is valid
        let success = unsafe { verovioxide_sys::vrvToolkit_setScale(self.ptr, scale) };

        if success {
            Ok(())
        } else {
            Err(Error::OptionsError(format!("invalid scale: {}", scale)))
        }
    }

    /// Gets the toolkit instance ID.
    #[must_use]
    pub fn get_id(&self) -> String {
        // SAFETY: ptr is valid
        let id_ptr = unsafe { verovioxide_sys::vrvToolkit_getID(self.ptr) };
        self.ptr_to_string(id_ptr).unwrap_or_default()
    }

    /// Gets the current resource path.
    #[must_use]
    pub fn get_resource_path(&self) -> String {
        // SAFETY: ptr is valid
        let path_ptr = unsafe { verovioxide_sys::vrvToolkit_getResourcePath(self.ptr) };
        self.ptr_to_string(path_ptr).unwrap_or_default()
    }

    /// Sets the resource path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Verovio resources directory
    ///
    /// # Errors
    ///
    /// Returns an error if the path is invalid.
    pub fn set_resource_path(&mut self, path: &Path) -> Result<()> {
        let path_str = path
            .to_str()
            .ok_or_else(|| Error::OptionsError("resource path contains invalid UTF-8".into()))?;

        let c_path = CString::new(path_str)?;

        // SAFETY: ptr is valid, c_path is a valid null-terminated string
        let success =
            unsafe { verovioxide_sys::vrvToolkit_setResourcePath(self.ptr, c_path.as_ptr()) };

        if success {
            Ok(())
        } else {
            Err(Error::OptionsError("failed to set resource path".into()))
        }
    }

    /// Gets the page number containing a specific element.
    ///
    /// # Arguments
    ///
    /// * `xml_id` - The xml:id of the element
    ///
    /// # Returns
    ///
    /// The page number (1-based), or 0 if the element is not found.
    pub fn get_page_with_element(&self, xml_id: &str) -> Result<u32> {
        let c_id = CString::new(xml_id)?;

        // SAFETY: ptr is valid, c_id is a valid null-terminated string
        let page =
            unsafe { verovioxide_sys::vrvToolkit_getPageWithElement(self.ptr, c_id.as_ptr()) };

        Ok(page.max(0) as u32)
    }

    /// Gets element attributes by xml:id.
    ///
    /// # Arguments
    ///
    /// * `xml_id` - The xml:id of the element
    ///
    /// # Returns
    ///
    /// A JSON string with the element's attributes.
    pub fn get_element_attr(&self, xml_id: &str) -> Result<String> {
        let c_id = CString::new(xml_id)?;

        // SAFETY: ptr is valid, c_id is a valid null-terminated string
        let attr_ptr =
            unsafe { verovioxide_sys::vrvToolkit_getElementAttr(self.ptr, c_id.as_ptr()) };

        self.ptr_to_string(attr_ptr).ok_or_else(|| {
            Error::RenderError(format!("failed to get attributes for element: {}", xml_id))
        })
    }

    /// Gets elements at a specific time in milliseconds.
    ///
    /// # Arguments
    ///
    /// * `millisec` - Time in milliseconds
    ///
    /// # Returns
    ///
    /// A JSON string with the element IDs at the specified time.
    pub fn get_elements_at_time(&self, millisec: i32) -> Result<String> {
        // SAFETY: ptr is valid
        let elements_ptr =
            unsafe { verovioxide_sys::vrvToolkit_getElementsAtTime(self.ptr, millisec) };

        self.ptr_to_string(elements_ptr).ok_or_else(|| {
            Error::RenderError(format!("failed to get elements at time: {}", millisec))
        })
    }

    /// Gets the time (in milliseconds) for an element.
    ///
    /// # Arguments
    ///
    /// * `xml_id` - The xml:id of the element
    ///
    /// # Returns
    ///
    /// The time in milliseconds.
    pub fn get_time_for_element(&self, xml_id: &str) -> Result<f64> {
        let c_id = CString::new(xml_id)?;

        // SAFETY: ptr is valid, c_id is a valid null-terminated string
        let time =
            unsafe { verovioxide_sys::vrvToolkit_getTimeForElement(self.ptr, c_id.as_ptr()) };

        Ok(time)
    }

    /// Redoes the layout with optional new options.
    ///
    /// # Arguments
    ///
    /// * `options` - Optional JSON string with layout options
    pub fn redo_layout(&mut self, options: Option<&str>) -> Result<()> {
        let c_options = CString::new(options.unwrap_or("{}"))?;

        // SAFETY: ptr is valid, c_options is a valid null-terminated string
        unsafe { verovioxide_sys::vrvToolkit_redoLayout(self.ptr, c_options.as_ptr()) };

        Ok(())
    }

    /// Performs an editor action on the loaded document.
    ///
    /// # Arguments
    ///
    /// * `action` - JSON string describing the editor action
    ///
    /// # Errors
    ///
    /// Returns an error if the action fails.
    pub fn edit(&mut self, action: &str) -> Result<()> {
        let c_action = CString::new(action)?;

        // SAFETY: ptr is valid, c_action is a valid null-terminated string
        let success = unsafe { verovioxide_sys::vrvToolkit_edit(self.ptr, c_action.as_ptr()) };

        if success {
            Ok(())
        } else {
            Err(Error::RenderError("editor action failed".into()))
        }
    }

    /// Gets information about the last edit operation.
    #[must_use]
    pub fn edit_info(&self) -> String {
        // SAFETY: ptr is valid
        let info_ptr = unsafe { verovioxide_sys::vrvToolkit_editInfo(self.ptr) };
        self.ptr_to_string(info_ptr).unwrap_or_default()
    }

    /// Enables or disables logging to stderr.
    ///
    /// # Arguments
    ///
    /// * `enable` - `true` to enable logging, `false` to disable
    pub fn enable_log(enable: bool) {
        // SAFETY: This function has no preconditions
        unsafe { verovioxide_sys::enableLog(enable) };
    }

    /// Enables or disables logging to an internal buffer.
    ///
    /// When enabled, log messages can be retrieved with [`get_log()`](Self::get_log).
    ///
    /// # Arguments
    ///
    /// * `enable` - `true` to enable buffer logging, `false` to disable
    pub fn enable_log_to_buffer(enable: bool) {
        // SAFETY: This function has no preconditions
        unsafe { verovioxide_sys::enableLogToBuffer(enable) };
    }

    /// Converts a C string pointer to an owned Rust string.
    ///
    /// Returns `None` if the pointer is null or contains invalid UTF-8.
    fn ptr_to_string(&self, ptr: *const i8) -> Option<String> {
        if ptr.is_null() {
            return None;
        }

        // SAFETY: ptr is non-null and points to a valid C string owned by the toolkit
        let c_str = unsafe { CStr::from_ptr(ptr) };

        c_str.to_str().ok().map(String::from)
    }
}

impl std::fmt::Debug for Toolkit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Toolkit")
            .field("version", &self.version())
            .field("page_count", &self.page_count())
            .field("resource_path", &self.get_resource_path())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolkit_without_resources() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        assert!(!toolkit.version().is_empty());
    }

    #[test]
    fn test_toolkit_version() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let version = toolkit.version();
        // Version should look like a version number
        assert!(!version.is_empty());
    }

    #[test]
    fn test_toolkit_page_count_empty() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        assert_eq!(toolkit.page_count(), 0);
    }

    #[test]
    fn test_toolkit_get_options() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let options = toolkit.get_options();
        let trimmed = options.trim();
        assert!(trimmed.starts_with('{'));
        assert!(trimmed.ends_with('}'));
    }

    #[test]
    fn test_toolkit_get_default_options() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let options = toolkit.get_default_options();
        assert!(options.starts_with('{'));
    }

    #[test]
    fn test_toolkit_get_available_options() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let options = toolkit.get_available_options();
        assert!(options.starts_with('{'));
    }

    #[test]
    fn test_toolkit_reset_options() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        toolkit.reset_options();
        // Should not panic
    }

    #[test]
    fn test_toolkit_get_scale() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let scale = toolkit.get_scale();
        assert!(scale > 0);
    }

    #[test]
    fn test_toolkit_set_scale() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        toolkit.set_scale(80).expect("Failed to set scale");
        assert_eq!(toolkit.get_scale(), 80);
    }

    #[test]
    fn test_toolkit_get_id() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let id = toolkit.get_id();
        assert!(!id.is_empty());
    }

    #[test]
    fn test_toolkit_debug() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let debug = format!("{:?}", toolkit);
        assert!(debug.contains("Toolkit"));
        assert!(debug.contains("version"));
    }

    #[test]
    fn test_toolkit_render_to_svg_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_to_svg(1);
        assert!(result.is_err());
    }

    #[test]
    fn test_toolkit_render_to_svg_page_zero() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_to_svg(0);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("out of range"));
    }

    #[test]
    fn test_toolkit_set_options() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let options = Options::builder().scale(80).build();
        toolkit
            .set_options(&options)
            .expect("Failed to set options");
    }

    #[test]
    fn test_toolkit_load_data_empty() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.load_data("");
        assert!(result.is_err());
    }

    #[test]
    fn test_toolkit_load_file_not_found() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.load_file(Path::new("/nonexistent/path/to/file.mei"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn test_toolkit_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Toolkit>();
    }

    #[test]
    fn test_toolkit_enable_log() {
        Toolkit::enable_log(true);
        Toolkit::enable_log(false);
        // Should not panic
    }

    #[test]
    fn test_toolkit_enable_log_to_buffer() {
        Toolkit::enable_log_to_buffer(true);
        Toolkit::enable_log_to_buffer(false);
        // Should not panic
    }

    #[test]
    fn test_toolkit_get_log() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let _log = toolkit.get_log();
        // Log may be empty, that's fine
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_new_with_bundled_data() {
        let toolkit = Toolkit::new().expect("Failed to create toolkit");
        assert!(!toolkit.version().is_empty());
        assert!(!toolkit.get_resource_path().is_empty());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_load_simple_mei() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");
        assert!(toolkit.page_count() > 0);
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_render_simple_mei() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let svg = toolkit.render_to_svg(1).expect("Failed to render SVG");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_render_all_pages() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let pages = toolkit.render_all_pages().expect("Failed to render pages");
        assert!(!pages.is_empty());
        for page in &pages {
            assert!(page.contains("<svg"));
        }
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_get_mei() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let exported_mei = toolkit.get_mei().expect("Failed to export MEI");
        assert!(exported_mei.contains("mei"));
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_with_options() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let options = Options::builder()
            .scale(50)
            .page_width(1000)
            .page_height(1000)
            .adjust_page_height(true)
            .build();

        toolkit
            .set_options(&options)
            .expect("Failed to set options");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let svg = toolkit.render_to_svg(1).expect("Failed to render SVG");
        assert!(svg.contains("<svg"));
    }

    // =========================================================================
    // Additional tests for improved coverage
    // =========================================================================

    #[test]
    fn test_toolkit_render_to_svg_with_declaration_page_zero() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_to_svg_with_declaration(0);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("out of range"));
    }

    #[test]
    fn test_toolkit_render_to_svg_with_declaration_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_to_svg_with_declaration(1);
        assert!(result.is_err());
    }

    #[test]
    fn test_toolkit_get_mei_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_mei();
        // With no data loaded, this may return an empty MEI or error depending on Verovio
        // Just ensure it doesn't panic and returns a result
        let _ = result;
    }

    #[test]
    fn test_toolkit_get_mei_with_options_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_mei_with_options(r#"{"removeIds": true}"#);
        // May succeed with empty result or error - just ensure no panic
        let _ = result;
    }

    #[test]
    fn test_toolkit_get_humdrum_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_humdrum();
        // May return empty or error - just ensure no panic
        let _ = result;
    }

    #[test]
    fn test_toolkit_render_to_midi_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_to_midi();
        assert!(result.is_err(), "render_to_midi should fail without data");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("no data loaded"),
            "Error should mention no data loaded"
        );
    }

    #[test]
    fn test_toolkit_render_to_pae_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_to_pae();
        assert!(result.is_err(), "render_to_pae should fail without data");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("no data loaded"),
            "Error should mention no data loaded"
        );
    }

    #[test]
    fn test_toolkit_render_to_timemap_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_to_timemap();
        // Returns JSON array (possibly empty) or error
        let _ = result;
    }

    #[test]
    fn test_toolkit_render_to_timemap_with_options_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_to_timemap_with_options(r#"{"includeMeasures": true}"#);
        let _ = result;
    }

    #[test]
    fn test_toolkit_render_to_expansion_map_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_to_expansion_map();
        let _ = result;
    }

    #[test]
    fn test_toolkit_set_scale_invalid() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        // Scale of 0 should be invalid
        let result = toolkit.set_scale(0);
        // Verovio may accept or reject this - test that we handle the result
        let _ = result;
    }

    #[test]
    fn test_toolkit_set_scale_negative() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.set_scale(-1);
        // Verovio may accept or reject this - test that we handle the result
        let _ = result;
    }

    #[test]
    fn test_toolkit_set_scale_large() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        // Very large scale
        let result = toolkit.set_scale(10000);
        let _ = result;
    }

    #[test]
    fn test_toolkit_get_resource_path() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let path = toolkit.get_resource_path();
        // Without resources, path may be empty
        let _ = path;
    }

    #[test]
    fn test_toolkit_set_resource_path_nonexistent() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.set_resource_path(Path::new("/nonexistent/path/to/resources"));
        // This may succeed (just sets the path) or fail depending on implementation
        let _ = result;
    }

    #[test]
    fn test_toolkit_get_page_with_element_not_found() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_page_with_element("nonexistent-id");
        // Should return 0 when element not found
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_toolkit_get_element_attr_not_found() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_element_attr("nonexistent-id");
        // May return empty JSON or error
        let _ = result;
    }

    #[test]
    fn test_toolkit_get_elements_at_time_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_elements_at_time(0);
        // May return empty array or error
        let _ = result;
    }

    #[test]
    fn test_toolkit_get_elements_at_time_negative() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_elements_at_time(-1000);
        let _ = result;
    }

    #[test]
    fn test_toolkit_get_time_for_element_not_found() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_time_for_element("nonexistent-id");
        assert!(result.is_ok());
        // Time may be 0 or negative when not found
    }

    #[test]
    fn test_toolkit_redo_layout_no_data() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.redo_layout(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_toolkit_redo_layout_with_options() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.redo_layout(Some(r#"{"pageWidth": 2100}"#));
        assert!(result.is_ok());
    }

    #[test]
    fn test_toolkit_edit_no_data() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.edit(r#"{"action": "commit"}"#);
        // Edit without data may fail
        let _ = result;
    }

    #[test]
    fn test_toolkit_edit_invalid_json() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.edit("not valid json");
        // Should handle invalid JSON gracefully
        let _ = result;
    }

    #[test]
    fn test_toolkit_edit_info() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let info = toolkit.edit_info();
        // Should return JSON (possibly empty object)
        let _ = info;
    }

    #[test]
    fn test_toolkit_load_data_with_null_byte() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.load_data("test\0data");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_load_data_malformed_mei() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.load_data("<mei><invalid></mei>");
        assert!(result.is_err());
    }

    #[test]
    fn test_toolkit_load_data_random_text() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.load_data("This is just random text, not valid music notation");
        assert!(result.is_err());
    }

    #[test]
    fn test_toolkit_get_mei_with_options_null_byte() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_mei_with_options("test\0options");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_render_to_timemap_with_options_null_byte() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_to_timemap_with_options("test\0options");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_get_page_with_element_null_byte() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_page_with_element("test\0id");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_get_element_attr_null_byte() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_element_attr("test\0id");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_get_time_for_element_null_byte() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.get_time_for_element("test\0id");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_redo_layout_null_byte() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.redo_layout(Some("test\0options"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_edit_null_byte() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.edit("test\0action");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("null byte"));
    }

    #[test]
    fn test_toolkit_set_options_null_byte_in_font() {
        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        // Create options with a null byte in a string field
        let mut options = Options::builder().build();
        options.font = Some("test\0font".to_string());
        let result = toolkit.set_options(&options);
        // The JSON serialization should succeed, but CString creation may fail
        // or the toolkit may reject it
        let _ = result;
    }

    #[test]
    fn test_toolkit_render_all_pages_no_data() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_all_pages();
        // With no data, page_count is 0, so we get empty vec
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_render_to_svg_with_declaration() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let svg = toolkit
            .render_to_svg_with_declaration(1)
            .expect("Failed to render SVG");
        assert!(svg.contains("<?xml"));
        assert!(svg.contains("<svg"));
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_render_to_midi() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let result = toolkit.render_to_midi();
        // MIDI rendering should succeed with valid MEI
        assert!(result.is_ok());
        let midi = result.unwrap();
        // MIDI is base64-encoded
        assert!(!midi.is_empty());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_get_mei_with_options() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let result = toolkit.get_mei_with_options(r#"{"removeIds": false}"#);
        assert!(result.is_ok());
        let exported = result.unwrap();
        assert!(exported.contains("mei"));
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_render_to_timemap() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let result = toolkit.render_to_timemap();
        assert!(result.is_ok());
        let timemap = result.unwrap();
        // Timemap is JSON
        assert!(timemap.starts_with('[') || timemap.starts_with('{'));
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_render_to_expansion_map() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let result = toolkit.render_to_expansion_map();
        assert!(result.is_ok());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_redo_layout_after_load() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        // Redo layout with different options
        let result = toolkit.redo_layout(Some(r#"{"pageWidth": 1500}"#));
        assert!(result.is_ok());

        // Should still be able to render
        let svg = toolkit
            .render_to_svg(1)
            .expect("Failed to render after redo");
        assert!(svg.contains("<svg"));
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_set_and_get_resource_path() {
        let toolkit = Toolkit::new().expect("Failed to create toolkit");
        let path = toolkit.get_resource_path();
        // With bundled data, resource path should not be empty
        assert!(!path.is_empty());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_get_elements_at_time_with_data() {
        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let result = toolkit.get_elements_at_time(0);
        assert!(result.is_ok());
        let elements = result.unwrap();
        // Should be valid JSON
        assert!(elements.starts_with('{') || elements.starts_with('['));
    }

    #[test]
    fn test_toolkit_with_resource_path_nonexistent() {
        let result = Toolkit::with_resource_path(Path::new("/nonexistent/resources"));
        // May fail or succeed depending on whether Verovio validates the path
        let _ = result;
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_load_file_with_tempfile() {
        use std::io::Write;

        let mut toolkit = Toolkit::without_resources().expect("Failed to create toolkit");

        // Create a temp file with invalid content
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.mei");
        let mut file = std::fs::File::create(&file_path).expect("Failed to create file");
        file.write_all(b"<invalid>not valid mei</invalid>")
            .expect("Failed to write");

        let result = toolkit.load_file(&file_path);
        // Should fail to load invalid MEI
        assert!(result.is_err());
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_load_file_with_valid_mei() {
        use std::io::Write;

        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.mei");
        let mut file = std::fs::File::create(&file_path).expect("Failed to create file");
        file.write_all(mei.as_bytes()).expect("Failed to write");

        let result = toolkit.load_file(&file_path);
        assert!(result.is_ok());
        assert!(toolkit.page_count() > 0);
    }

    #[test]
    fn test_toolkit_not_sync() {
        // This is a compile-time check - Toolkit should NOT implement Sync
        // We can't easily test this at runtime, but we document it here
        // fn assert_sync<T: Sync>() {}
        // assert_sync::<Toolkit>(); // This would fail to compile
    }

    #[test]
    fn test_toolkit_debug_format_detailed() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let debug = format!("{:?}", toolkit);
        assert!(debug.contains("Toolkit"));
        assert!(debug.contains("version"));
        assert!(debug.contains("page_count"));
        assert!(debug.contains("resource_path"));
    }

    #[test]
    fn test_toolkit_enable_log_toggle() {
        // Test toggling log on and off multiple times
        Toolkit::enable_log(true);
        Toolkit::enable_log(true);
        Toolkit::enable_log(false);
        Toolkit::enable_log(false);
        Toolkit::enable_log(true);
        Toolkit::enable_log(false);
    }

    #[test]
    fn test_toolkit_enable_log_to_buffer_toggle() {
        // Test toggling buffer log on and off multiple times
        Toolkit::enable_log_to_buffer(true);
        Toolkit::enable_log_to_buffer(true);
        Toolkit::enable_log_to_buffer(false);
        Toolkit::enable_log_to_buffer(false);
        Toolkit::enable_log_to_buffer(true);
        Toolkit::enable_log_to_buffer(false);
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_toolkit_logging_with_data() {
        Toolkit::enable_log_to_buffer(true);

        let mut toolkit = Toolkit::new().expect("Failed to create toolkit");

        // Load some data to generate log messages
        let mei = r#"<?xml version="1.0" encoding="UTF-8"?>
<mei xmlns="http://www.music-encoding.org/ns/mei">
  <music>
    <body>
      <mdiv>
        <score>
          <scoreDef>
            <staffGrp>
              <staffDef n="1" lines="5" clef.shape="G" clef.line="2"/>
            </staffGrp>
          </scoreDef>
          <section>
            <measure>
              <staff n="1">
                <layer n="1">
                  <note pname="c" oct="4" dur="4"/>
                </layer>
              </staff>
            </measure>
          </section>
        </score>
      </mdiv>
    </body>
  </music>
</mei>"#;

        toolkit.load_data(mei).expect("Failed to load MEI");

        let log = toolkit.get_log();
        // Log may or may not have content depending on Verovio's behavior
        let _ = log;

        Toolkit::enable_log_to_buffer(false);
    }

    #[test]
    fn test_toolkit_render_to_svg_page_exceeds_count() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        // With no data, page_count is 0, so page 1 should be out of range
        let result = toolkit.render_to_svg(1);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("out of range"));
        assert!(err.to_string().contains("0 pages"));
    }

    #[test]
    fn test_toolkit_render_to_svg_with_declaration_page_exceeds_count() {
        let toolkit = Toolkit::without_resources().expect("Failed to create toolkit");
        let result = toolkit.render_to_svg_with_declaration(100);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("out of range"));
    }
}
