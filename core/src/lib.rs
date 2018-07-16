//! `Device` manages creation and destruction of resources. Images and buffers.
//! It features smart memory allocator that manages all memory heaps and
//! allocates memory for buffers and images according with requirements based on usage and resource properties.
//!
//! Buffers and images allocated from `Device` are safe to drop. Underlying resources won't be freed until all commands are complete.
//! Unless bare handles are retrieved. In which case user must ensure that command recorded with bare handles are complete before dropping the wrapper.
//! Alternatively resources may be returned to `Device` with `free_*` methods that are slightly faster than dropping. Safety rules are the same as for dropping.
//! 
//! Also `Device` manages uploading and downloading data from buffers and images.
//! It issues commands to the transfer queue and returns immediately with an object that can be fetched for the completion.
//!
//! `Device` exposes command queues from which command pools may be created.
//! 

#[macro_use]
extern crate ash;
extern crate crossbeam_channel;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate relevant;
extern crate smallvec;
extern crate winit;

#[cfg(target_os = "macos")]
extern crate cocoa;
#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

mod escape;

pub mod buffer;
pub mod command;
pub mod device;
pub mod format;
pub mod image;
pub mod memory;
pub mod object;
pub mod surface;
pub mod swapchain;

#[derive(Clone, Copy, Debug, Fail)]
#[fail(display = "Device lost")]
pub struct DeviceLost;

impl DeviceLost {
    fn from_vk_result(result: ash::vk::Result) -> Self {
        match result {
            ash::vk::Result::ErrorDeviceLost => DeviceLost,
            _ => panic!("Unexpected result value"),
        }
    }
}

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

impl OomError {
    fn from_vk_result(result: ash::vk::Result) -> Self {
        match result {
            ash::vk::Result::ErrorOutOfHostMemory => OomError::OutOfHostMemory,
            ash::vk::Result::ErrorOutOfDeviceMemory => OomError::OutOfDeviceMemory,
            _ => panic!("Unexpected result value"),
        }
    }
}

#[derive(Clone, Copy, Debug, Fail)]
pub enum DeviceLostOrOomError {
    #[fail(display = "{}", _0)]
    OomError(OomError),
    #[fail(display = "{}", _0)]
    DeviceLost(DeviceLost)
}

impl DeviceLostOrOomError {
    fn from_vk_result(result: ash::vk::Result) -> Self {
        match result {
            ash::vk::Result::ErrorOutOfHostMemory => DeviceLostOrOomError::OomError(OomError::OutOfHostMemory),
            ash::vk::Result::ErrorOutOfDeviceMemory => DeviceLostOrOomError::OomError(OomError::OutOfDeviceMemory),
            ash::vk::Result::ErrorDeviceLost => DeviceLostOrOomError::DeviceLost(DeviceLost),
            _ => panic!("Unexpected result value"),
        }
    }
}
