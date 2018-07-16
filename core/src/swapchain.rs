
use std::ptr::null;
use ash;

use {OomError, DeviceLost};
use device::Device;
use surface::Surface;


#[derive(Clone, Debug, Fail)]
pub enum CreateSwapchainError {
    #[fail(display = "{}", _0)]
    OomError(OomError),

    #[fail(display = "{}", _0)]

    DeviceLost(DeviceLost),

    #[fail(display = "Surface lost")]
    SurfaceLost,

    #[fail(display = "Native window in use")]
    WindowInUse,
}

pub struct SwapchainConfig {
    pub min_image_count: u32,
    pub image_format: ash::vk::Format,
    pub image_extent: ash::vk::Extent2D,
    pub image_usage: ash::vk::ImageUsageFlags,
    pub present_mode: ash::vk::PresentModeKHR,
}

impl CreateSwapchainError {
    fn from_vk_result(result: ash::vk::Result) -> Self {
        match result {
            ash::vk::Result::ErrorOutOfHostMemory => CreateSwapchainError::OomError(OomError::OutOfHostMemory),
            ash::vk::Result::ErrorOutOfDeviceMemory => CreateSwapchainError::OomError(OomError::OutOfDeviceMemory),
            ash::vk::Result::ErrorDeviceLost => CreateSwapchainError::DeviceLost(DeviceLost),
            ash::vk::Result::ErrorSurfaceLostKhr => CreateSwapchainError::SurfaceLost,
            ash::vk::Result::ErrorNativeWindowInUseKhr => CreateSwapchainError::WindowInUse,
            _ => panic!("Unexpected result value"),
        }
    }
}

pub struct Swapchain {
    raw: ash::vk::SwapchainKHR,
}

impl Swapchain {
    pub fn extensions() -> Vec<&'static str> {
        vec![ash::extensions::Swapchain::name().to_str().unwrap()]
    }

    /// Create new swapchain
    pub fn create(device: &Device, surface: &Surface, config: SwapchainConfig, old_swapchain: Option<Self>) -> Result<Self, CreateSwapchainError> {
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
            error => Err(CreateSwapchainError::from_vk_result(error)),
        }
    }
}
