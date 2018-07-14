//! `Factory` manages creation and destruction of resources. Images and buffers.
//! It features smart memory allocator that manages all memory heaps and
//! allocates memory for buffers and images according with requirements based on usage and resource properties.
//!
//! Buffers and images allocated from `Factory` are safe to drop. Underlying resources won't be freed until all commands are complete.
//! Unless bare handles are retrieved. In which case user must ensure that command recorded with bare handles are complete before dropping the wrapper.
//! Alternatively resources may be returned to `Factory` with `free_*` methods that are slightly faster than dropping. Safety rules are the same as for dropping.
//! 
//! Also `Factory` manages uploading and downloading data from buffers and images.
//! It issues commands to the transfer queue and returns immediately with an object that can be fetched for the completion.
//!
//! `Factory` exposes command queues from which command pools may be created.
//! 

#[macro_use]
extern crate ash;
extern crate crossbeam_channel;
#[macro_use]
extern crate failure;
extern crate relevant;
extern crate smallvec;

mod escape;

pub mod buffer;
pub mod command;
pub mod factory;
pub mod format;
pub mod image;
pub mod memory;
mod object;
pub mod tracker;

#[derive(Clone, Copy, Debug, Fail)]
#[fail(display = "Device lost")]
pub struct DeviceLost;

/// Out of memory error.
#[derive(Clone, Copy, Debug, Fail)]
pub enum OomError {
    /// Host memory exhausted.
    #[fail(display = "Out of host memory")]
    OutOfHostMemory,

    /// Device memory exhausted.
    #[fail(display = "Out of device memory")]
    OutOfDeviceMemory,
}

#[derive(Clone, Copy, Debug, Fail)]
pub enum DeviceLostOrOomError {
    #[fail(display = "{}", _0)]
    OomError(OomError),
    #[fail(display = "{}", _0)]
    DeviceLost(DeviceLost)
}
