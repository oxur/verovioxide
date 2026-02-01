//! Error types for verovioxide.
//!
//! This module defines the error types used throughout the verovioxide crate.
//! All errors provide context about what went wrong and can be converted to
//! `std::error::Error` for use with the `?` operator.

use std::path::PathBuf;

/// The result type used throughout verovioxide.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when using the verovioxide library.
///
/// # See also
///
/// - [`Result`] - The result type alias using this error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to initialize the Verovio toolkit.
    ///
    /// This typically occurs when:
    /// - Memory allocation fails
    /// - Resource initialization fails
    ///
    /// # Returned by
    ///
    /// - [`Toolkit::new`](crate::Toolkit::new)
    /// - [`Toolkit::with_resource_path`](crate::Toolkit::with_resource_path)
    /// - [`Toolkit::without_resources`](crate::Toolkit::without_resources)
    #[error("failed to initialize toolkit: {0}")]
    InitializationError(String),

    /// Failed to load music data into the toolkit.
    ///
    /// This can occur when:
    /// - The input data is malformed
    /// - The input format is not recognized
    /// - The file cannot be read
    ///
    /// # Returned by
    ///
    /// - [`Toolkit::load_data`](crate::Toolkit::load_data)
    /// - [`Toolkit::load_file`](crate::Toolkit::load_file)
    #[error("failed to load data: {0}")]
    LoadError(String),

    /// Failed to render the music notation.
    ///
    /// This can occur when:
    /// - No data has been loaded
    /// - The page number is out of range
    /// - Internal rendering error occurs
    ///
    /// # Returned by
    ///
    /// - [`Toolkit::render_to_svg`](crate::Toolkit::render_to_svg)
    /// - [`Toolkit::render_to_svg_with_declaration`](crate::Toolkit::render_to_svg_with_declaration)
    /// - [`Toolkit::render_all_pages`](crate::Toolkit::render_all_pages)
    /// - [`Toolkit::render_to_midi`](crate::Toolkit::render_to_midi)
    /// - [`Toolkit::render_to_pae`](crate::Toolkit::render_to_pae)
    /// - [`Toolkit::get_mei`](crate::Toolkit::get_mei)
    /// - [`Toolkit::get_humdrum`](crate::Toolkit::get_humdrum)
    #[error("failed to render: {0}")]
    RenderError(String),

    /// Invalid options provided to the toolkit.
    ///
    /// This can occur when:
    /// - Option values are out of valid range
    /// - JSON serialization fails
    /// - Unknown option keys are provided
    ///
    /// # Returned by
    ///
    /// - [`Toolkit::set_options`](crate::Toolkit::set_options)
    /// - [`Toolkit::set_scale`](crate::Toolkit::set_scale)
    /// - [`Toolkit::set_resource_path`](crate::Toolkit::set_resource_path)
    #[error("invalid options: {0}")]
    OptionsError(String),

    /// Failed to work with resource files.
    ///
    /// This variant is only available when the `bundled-data` feature is enabled.
    ///
    /// # Returned by
    ///
    /// - [`Toolkit::new`](crate::Toolkit::new) (when extracting bundled resources)
    #[cfg(feature = "bundled-data")]
    #[error("resource error: {0}")]
    ResourceError(#[from] verovioxide_data::DataError),

    /// I/O error occurred.
    ///
    /// This can occur when:
    /// - Reading a file fails
    /// - Writing output fails
    /// - Path operations fail
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// The requested file was not found.
    ///
    /// # Returned by
    ///
    /// - [`Toolkit::load_file`](crate::Toolkit::load_file)
    #[error("file not found: {}", .0.display())]
    FileNotFound(PathBuf),

    /// A string contained invalid UTF-8.
    #[error("invalid UTF-8 in string")]
    InvalidUtf8,

    /// A string contained a null byte.
    ///
    /// This occurs when passing strings with embedded null bytes to Verovio,
    /// which expects null-terminated C strings.
    #[error("string contains null byte")]
    NullByteInString(#[from] std::ffi::NulError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_initialization() {
        let err = Error::InitializationError("test error".to_string());
        assert_eq!(err.to_string(), "failed to initialize toolkit: test error");
    }

    #[test]
    fn test_error_display_load() {
        let err = Error::LoadError("invalid MEI".to_string());
        assert_eq!(err.to_string(), "failed to load data: invalid MEI");
    }

    #[test]
    fn test_error_display_render() {
        let err = Error::RenderError("page out of range".to_string());
        assert_eq!(err.to_string(), "failed to render: page out of range");
    }

    #[test]
    fn test_error_display_options() {
        let err = Error::OptionsError("invalid scale".to_string());
        assert_eq!(err.to_string(), "invalid options: invalid scale");
    }

    #[test]
    fn test_error_display_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let err: Error = io_err.into();
        assert!(err.to_string().contains("I/O error"));
    }

    #[test]
    fn test_error_display_file_not_found() {
        let err = Error::FileNotFound(PathBuf::from("/path/to/file.mei"));
        assert_eq!(err.to_string(), "file not found: /path/to/file.mei");
    }

    #[test]
    fn test_error_display_invalid_utf8() {
        let err = Error::InvalidUtf8;
        assert_eq!(err.to_string(), "invalid UTF-8 in string");
    }

    #[test]
    fn test_error_display_null_byte() {
        let nul_err = std::ffi::CString::new("test\0string").unwrap_err();
        let err: Error = nul_err.into();
        assert!(err.to_string().contains("null byte"));
    }

    #[test]
    fn test_error_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Error>();
    }

    #[test]
    fn test_error_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Error>();
    }

    #[cfg(feature = "bundled-data")]
    #[test]
    fn test_error_from_data_error() {
        let data_err = verovioxide_data::DataError::TempDirCreation(std::io::Error::new(
            std::io::ErrorKind::Other,
            "test",
        ));
        let err: Error = data_err.into();
        assert!(err.to_string().contains("resource error"));
    }
}
