
use std::{collections::LinkedList, ops::Range, sync::Arc};
use encode;

use buffer;
use command;
use format;
use image;
use memory;
use shop::Shop;
use tracker::GlobalTracker;

/// Factory contains devices and queues.
/// It abstracts some details to provide safer API.
/// Yet it safely exposes internal objects because using them requires `unsafe`.
pub struct Factory {
    instance: ash::vk::Instance,
    physical: ash::vk::PhysicalDevice,
    device: ash::vk::Device,
    families: Vec<command::Family>,
    terminal: Terminal,
}

impl Borrow<ash::vk::Instance> for Factory {
    fn borrow(&self) -> &ash::vk::Instance {
        &self.instance
    }
}

impl Borrow<ash::vk::PhysicalDevice> for Factory {
    fn borrow(&self) -> &ash::vk::PhysicalDevice {
        &self.physical
    }
}

impl Borrow<ash::vk::Device> for Factory {
    fn borrow(&self) -> &ash::vk::Device {
        &self.device
    }
}

impl Shop for Factory {
    /// Create new buffer.
    fn create_buffer(
        &mut self,
        align: u64,
        size: u64,
        usage: buffer::Usage,
        properties: memory::Properties,
    ) -> buffer::Buffer {
        unimplemented!()
    }

    /// Create new image.
    fn create_image(
        &mut self,
        kind: image::Kind,
        format: format::Format,
        layout: image::Layout,
        usage: image::Usage,
        properties: memory::Properties,
    ) -> image::Image {
        unimplemented!()
    }
}

impl Factory {
    /// Take tracked resources from `GlobalTracker` and hold them until device stop using them.
    pub fn collect_from_tracker(&mut self, tracker: &mut GlobalTracker) {
        let objects = Arc::new(self.terminal.drain().collect::<VulkanObjects>());
        for queue in self.families.iter_mut().flat_map(command::Family::queues) {
            queue.push_track(objects.clone());
        }
    }
}
