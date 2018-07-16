//!
//! Yet another rendering engine.
//! 

pub extern crate ash;
#[doc(hidden)]
extern crate rendy_core;

/// Unsafe core of the rendy.
/// Mostly thin wrappers around vulkan objects.
pub use rendy_core::*;
