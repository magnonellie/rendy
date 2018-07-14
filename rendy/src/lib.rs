//!
//! Yet another rendering engine.
//! 

#[doc(hidden)]
pub extern crate rendy_core;

/// Unsafe core of the rendy.
/// Mostly thin wrappers around vulkan objects.
pub use rendy_core as core;
