//! Re-exports and type aliases for cleaner FFI organization.
//!
//! This module provides convenient type aliases and re-exports for working
//! with the Verovio C wrapper API.

use std::ffi::c_void;

/// Opaque pointer type for the Verovio toolkit instance.
///
/// This is an alias for `*mut c_void` to provide clearer semantics
/// when working with toolkit pointers.
pub type ToolkitPtr = *mut c_void;

/// A null toolkit pointer constant for convenience.
pub const NULL_TOOLKIT: ToolkitPtr = std::ptr::null_mut();

/// Check if a toolkit pointer is valid (non-null).
///
/// # Safety
///
/// This only checks for null; it does not validate that the pointer
/// actually points to a valid toolkit instance.
#[inline]
pub fn is_valid_toolkit(ptr: ToolkitPtr) -> bool {
    !ptr.is_null()
}
