
use std::{ops::Range, sync::Arc};
use ash;
use escape::Escape;
use memory;

pub type Usage = ash::vk::BufferUsageFlags;
pub type RawBuffer = ash::vk::Buffer;

pub struct Buffer {
    resource: Arc<Escape<ash::vk::Buffer>>,
    usage: Usage,
    memory: memory::RawMemory,
    range: Range<u64>,
}

impl Buffer {
    /// Get raw buffer handle.
    pub fn raw(&self) -> RawBuffer {
        **self.resource
    }
}
