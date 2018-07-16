
use std::ptr::null;
use ash;

use errors::SurfaceError;
use device::Device;
use surface::Surface;

pub struct SwapchainConfig {
    pub min_image_count: u32,
    pub image_format: ash::vk::Format,
    pub image_extent: ash::vk::Extent2D,
    pub image_usage: ash::vk::ImageUsageFlags,
    pub present_mode: ash::vk::PresentModeKHR,
}

pub struct Swapchain {
    raw: ash::vk::SwapchainKHR,
}

impl Swapchain {
    pub fn extensions() -> Vec<&'static str> {
        vec![ash::extensions::Swapchain::name().to_str().unwrap()]
    }

    /// Create new swapchain
    pub fn create(device: &Device, surface: &Surface, old_swapchain: Option<Self>, config: SwapchainConfig) -> Result<Self, SurfaceError> {
        let mut swapchain = ash::vk::SwapchainKHR::null();
        let result = unsafe {
            device.swapchain.as_ref().unwrap().create_swapchain_khr(
                device.raw,
                &ash::vk::SwapchainCreateInfoKHR {
                    s_type: ash::vk::StructureType::SwapchainCreateInfoKhr,
                    p_next: null(),
                    flags: ash::vk::SwapchainCreateFlagsKHR::empty(),
                    surface: surface.raw,
                    min_image_count: config.min_image_count,
                    image_format: config.image_format,
                    image_color_space: ash::vk::ColorSpaceKHR::SrgbNonlinear,
                    image_extent: config.image_extent,
                    image_array_layers: 1,
                    image_usage: config.image_usage,
                    image_sharing_mode: ash::vk::SharingMode::Exclusive,
                    queue_family_index_count: 0,
                    p_queue_family_indices: null(),
                    pre_transform: ash::vk::SURFACE_TRANSFORM_INHERIT_BIT_KHR,
                    composite_alpha: ash::vk::COMPOSITE_ALPHA_INHERIT_BIT_KHR,
                    present_mode: config.present_mode,
                    clipped: 1,
                    old_swapchain: old_swapchain.map_or(ash::vk::SwapchainKHR::null(), |swapchain| swapchain.raw),
                },
                null(),
                &mut swapchain,
            )
        };

        match result {
            ash::vk::Result::Success => Ok(Swapchain {
                raw: swapchain,
            }),
            error => Err(SurfaceError::from_vk_result(error)),
        }
    }
}
