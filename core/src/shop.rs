
use format;
use buffer;
use image;
use memory;

/// Shop is a part of the `Factory` that can allocate resources.
pub trait Shop {
    /// Create new buffer.
    fn create_buffer(
        &mut self,
        align: u64,
        size: u64,
        usage: buffer::Usage,
        properties: memory::Properties,
    ) -> buffer::Buffer;

    /// Create new image.
    fn create_image(
        &mut self,
        kind: image::Kind,
        format: format::Format,
        layout: image::Layout,
        usage: image::Usage,
        properties: memory::Properties,
    ) -> image::Image;
}