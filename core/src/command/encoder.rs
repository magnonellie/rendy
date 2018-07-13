
use std::ops::Range;
use capability::*;

/// Encoder is implemented by buffer in recording state.
///
pub trait Encoder<C> {
    unsafe fn fill_buffer(
        &mut self,
        buffer: B,
        offset: u64,
        size: u64,
        data: u32,
    )
    where
        C: Supports<Transfer>,
    ;

    unsafe fn update_buffer(
        &mut self,
        buffer: B,
        offset: u64,
        data: &[u8]
    )
    where
        C: Supports<Transfer>,
    ;

    unsafe fn copy_buffer(
        &mut self,
        src: B,
        dst: B,
        regions: &[ash::vk::BufferCopy],
    )
    where
        C: Supports<Transfer>,
    ;

    unsafe fn copy_image(
        &mut self,
        src: I,
        src_layout: ash::vk::ImageLayout,
        dst: I,
        dst_layout: ash::vk::ImageLayout,
        regions: &[ash::vk::ImageCopy],
    )
    where
        C: Supports<Transfer>,
    ;

    /// Clear color image
    ///
    /// # Parameters
    ///
    /// `layout`    - VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL or VK_IMAGE_LAYOUT_GENERAL or VK_IMAGE_LAYOUT_SHARED_PRESENT_KHR.
    unsafe fn clear_color_image(
        &mut self,
        image: I,
        layout: ash::vk::ImageLayout,
        clear_value: ClearValue,
        ranges: &[ash::vk::SubresourceRange],
    )
    where
        C: Supports<GraphicsOrCompute>,
    ;

    unsafe fn clear_depth_stencil_image(
        &mut self,
        image: I,
        layout: ash::vk::ImageLayout,
        depth_stencil: ash::vk::ClearDepthStencilValue,
        ranges: &[ash::vk::SubresourceRange],
    )
    where
        C: Supports<Graphics>,
    ;

    unsafe fn bind_index_buffer(
        &mut self,
        buffer: B,
        offset: u64,
        index_type: ash::vk::IndexType,
    )
    where
        C: Supports<Graphics>,
    ;

    unsafe fn draw(
        &mut self,
        vertices: Range<u32>,
        instances: Range<u32>,
    )
    where
        C: Supports<Graphics>,
    ;

    unsafe fn draw_indexed(
        &mut self,
        indices: Range<u32>,
        instances: Range<u32>,
        vertex_offset: u32,
    )
    where
        C: Supports<Graphics>,
    ;

    /// `buffer` - contains `count` `ash::vk::types::DrawIndirectCommand` with `stride` bytes between starting from `offset`
    unsafe fn draw_indirect(
        &mut self,
        buffer: B,
        offset: u64,
        count: u32,
        stride: u32,
    )
    where
        C: Supports<Graphics>,
    ;

    /// `buffer` - contains `count` `ash::vk::types::DrawIndexedIndirectCommand` with `stride` bytes between starting from `offset`
    unsafe fn draw_indexed_indirect(
        &mut self,
        buffer: B,
        offset: u64,
        count: u32,
        stride: u32,
    )
    where
        C: Supports<Graphics>,
    ;

    fn begin_render_pass(
        &'a mut self,
        render_pass: R,
        framebuffer: F,
        render_area: ash::vk::Rect2D,
        clear_values: &[ClearValue]
    ) -> RenderPassEncoder<'a, Self>
    where
        C: Supports<Graphics>,
    ;
}

pub struct RenderPassEncoder<'a, E> {
    encoder: &'a mut E,
}


pub enum ClearValue {
    ColorFloat([f32; 4]),
    ColorInt([i32; 4]),
    ColorUint([u32; 4]),
    DepthStencil(f32, u32),
}
