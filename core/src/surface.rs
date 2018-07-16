
use std::{ffi::CStr, ptr::{null, null_mut}};
use ash;
use winit::Window;

use errors::{OomError, SurfaceError};
use device::{Instance, PhysicalDevice};

pub struct Surface {
    pub(crate) raw: ash::vk::SurfaceKHR,
    pub(crate) window: Window,
}

impl Surface {
    /// Surface extensions.
    /// This extension must be enabled to create surfaces.
    pub fn extensions() -> Vec<&'static str> {
        SurfaceFn::extensions().into_iter().map(|string| string.to_str().unwrap()).collect()
    }

    /// Create surface.
    pub fn create(instance: &Instance, window: Window) -> Result<Self, SurfaceError> {
        let raw = instance.inner.surface.as_ref().unwrap().create_surface(instance.handle(), &window)?;

        Ok(Surface {
            raw,
            window,
        })
    }

    /// Check if surface presentation is supported by queue family.
    pub fn supports_queue_family(&self, physical_device: &PhysicalDevice, family_index: u32) -> Result<bool, SurfaceError> {
        physical_device.instance.inner.surface.as_ref().unwrap().supports_queue_family(physical_device.raw, self.raw, family_index)
    }

    pub fn supported_formats(&self, physical_device: &PhysicalDevice) -> Result<impl IntoIterator<Item = ash::vk::Format>, SurfaceError> {
        physical_device.instance.inner.surface.as_ref().unwrap().supported_formats(physical_device.raw, self.raw)
    }
}

#[cfg(target_os = "macos")]
type PlatformFn = ash::vk::MacOSSurfaceFn;

#[cfg(target_os = "ios")]
type PlatformFn = ash::vk::IOSSurfaceFn;

#[cfg(windows)]
type PlatformFn = ash::vk::Win32SurfaceFn;

pub struct SurfaceFn {
    fp: ash::vk::SurfaceFn,
    platform: PlatformFn,
}

impl SurfaceFn {
    /// Surface extensions.
    pub fn extensions() -> Vec<&'static CStr> {
        vec![
            ash::extensions::Surface::name(),
            #[cfg(target_os = "macos")]
            ash::extensions::MacOSSurface::name(),
        ]
    }

    pub fn load<F>(mut f: F) -> Result<Self, Vec<&'static str>>
    where
        F: FnMut(&CStr) -> *const ash::vk::c_void, 
    {
        Ok(SurfaceFn {
            fp: ash::vk::SurfaceFn::load(&mut f)?,
            platform: PlatformFn::load(&mut f)?,
        })
    }

    pub fn new(instance: ash::vk::Instance, entry: &ash::vk::StaticFn) -> Result<Self, Vec<&'static str>> {
        Self::load(|name| unsafe {
            ::std::mem::transmute(entry.get_instance_proc_addr(
                instance,
                name.as_ptr(),
            ))
        })
    }

    #[cfg(target_os = "macos")]
    fn create_surface(&self, instance: ash::vk::Instance, window: &Window) -> Result<ash::vk::SurfaceKHR, SurfaceError> {
        use objc::runtime::{BOOL, Class, Object};
        use cocoa::appkit::NSView;
        use winit::os::macos::WindowExt;

        let nsview: *mut Object = window.get_nsview() as _;
        unsafe {
            let layer = NSView::layer(nsview);
            let layer_class = class!(CAMetalLayer);
            let isKind: BOOL = msg_send![layer, isKindOfClass:layer_class];
            if isKind == 0 {
                let render_layer: *mut Object = msg_send![layer_class, new];
                msg_send![nsview, setLayer: render_layer];
            }
        }

        let mut surface = ash::vk::SurfaceKHR::null();
        let result = unsafe {
            self.platform.create_macos_surface_mvk(
                instance,
                &ash::vk::MacOSSurfaceCreateInfoMVK {
                    s_type: ash::vk::StructureType::MacOSSurfaceCreateInfoMvk,
                    p_next: null(),
                    flags: ash::vk::MacOSSurfaceCreateFlagsMVK::empty(),
                    p_view: nsview as _,
                },
                null(),
                &mut surface,
            )
        };

        match result {
            ash::vk::Result::Success => {
                trace!("MacOS surface created");
                Ok(surface)
            },
            error => Err(SurfaceError::from_vk_result(result)),
        }
    }

    fn supports_queue_family(&self, physical_device: ash::vk::PhysicalDevice, surface: ash::vk::SurfaceKHR, family_index: u32) -> Result<bool, SurfaceError> {
        let mut b = 0;
        let result = unsafe {
            self.fp.get_physical_device_surface_support_khr(physical_device, family_index, surface, &mut b)
        };

        match result {
            ash::vk::Result::Success => Ok(b > 0),
            error => Err(SurfaceError::from_vk_result(error)),
        }
    }

    fn supported_formats(&self, physical_device: ash::vk::PhysicalDevice, surface: ash::vk::SurfaceKHR) -> Result<impl IntoIterator<Item = ash::vk::Format>, SurfaceError> {
        unsafe {
            let mut count = 0;
            let result = self.fp.get_physical_device_surface_formats_khr(
                physical_device,
                surface,
                &mut count,
                null_mut(),
            );

            match result {
                ash::vk::Result::Success => {},
                error => return Err(SurfaceError::from_vk_result(error)),
            }

            let mut formats = Vec::with_capacity(count as usize);
            let result = self.fp.get_physical_device_surface_formats_khr(
                physical_device,
                surface,
                &mut count,
                formats.as_mut_ptr(),
            );

            match result {
                ash::vk::Result::Success => {
                    formats.set_len(count as usize);
                    Ok(formats.into_iter().map(|format| format.format))
                },
                error => Err(SurfaceError::from_vk_result(error))
            }
        }
    }
}
