
use std::{any::Any, borrow::Borrow, collections::LinkedList, ffi::{CString, CStr}, ops::{Deref, Range}, ptr::null, sync::Arc};
use ash::{self, version::{DeviceV1_0, EntryV1_0, InstanceV1_0}};
use relevant::Relevant;
use winit::Window;

use {OomError, DeviceLost};
use buffer;
use command;
use escape::Terminal;
use format;
use image;
use memory;
use object::VulkanObjects;
use surface::{Surface, SurfaceFn};
use swapchain::Swapchain;

/// Layer description
#[derive(Clone, Debug)]
pub struct Layer<'a> {
    pub name: &'a str,
    pub spec_version: u32,
    pub implementation_version: u32,
    pub description: &'a str,
}

/// Extension description
#[derive(Clone, Debug)]
pub struct Extension<'a> {
    pub name: &'a str,
    pub spec_version: u32,
}

/// Properties of the physical device.
#[derive(Clone, Debug)]
pub struct PhysicalDeviceProperties {
    pub api_version: u32,
    pub driver_version: u32,
    pub vendor_id: u32,
    pub device_id: u32,
    pub device_type: ash::vk::PhysicalDeviceType,
    pub device_name: String,
    pub pipeline_cache_uuid: [u8; 16],
    pub limits: ash::vk::PhysicalDeviceLimits,
    pub sparse_properties: ash::vk::PhysicalDeviceSparseProperties,
}

/// Properties of the command queue family.
#[derive(Clone, Copy, Debug)]
pub struct QueueFamilyProperties {
    pub index: u32,
    pub capability: command::Capability,
    pub queue_count: u32,
}

/// Config for vulkan instance.
#[derive(Clone, Debug)]
pub struct InstanceConfig {
    pub app_name: String,
    pub app_version: u32,
    pub layers: Vec<String>,
    pub extensions: Vec<String>,
}

/// Request for creating command queues.
#[derive(Clone, Copy, Debug)]
pub struct CreateQueueFamily {
    pub family: u32,
    pub count: u32,
}

/// Possible errors returned by `Instance` and `PhysicalDevice`.
#[derive(Clone, Debug, Fail)]
pub enum InstanceError {
    #[fail(display = "Failed to load vulkan library {}", _0)]
    LibraryLoadError(String),

    #[fail(display = "Failed to load functions {:?}", _0)]
    LoadError(Vec<&'static str>),

    #[fail(display = "OomError")]
    OomError(OomError),

    #[fail(display = "Initialization failed")]
    InitializationFailed,

    #[fail(display = "Layer not present")]
    LayerNotPresent,

    #[fail(display = "Extension not present")]
    ExtensionNotPresent,

    #[fail(display = "Incompatible driver")]
    IncompatibleDriver,
}

impl InstanceError {
    fn from_loading_error(error: ash::LoadingError) -> Self {
        match error {
            ash::LoadingError::LibraryLoadError(name) => InstanceError::LibraryLoadError(name),
            ash::LoadingError::EntryLoadError(names) => InstanceError::LoadError(names),
            ash::LoadingError::StaticLoadError(names) => InstanceError::LoadError(names),
        }
    }

    fn from_instance_error(error: ash::InstanceError) -> Self {
        match error {
            ash::InstanceError::LoadError(names) => InstanceError::LoadError(names),
            ash::InstanceError::VkError(result) => InstanceError::from_vk_result(result),
        }
    }

    fn from_vk_result(result: ash::vk::Result) -> Self {
        match result {
            ash::vk::Result::ErrorOutOfHostMemory => InstanceError::OomError(OomError::OutOfHostMemory),
            ash::vk::Result::ErrorOutOfDeviceMemory => InstanceError::OomError(OomError::OutOfDeviceMemory),
            ash::vk::Result::ErrorInitializationFailed => InstanceError::InitializationFailed,
            ash::vk::Result::ErrorLayerNotPresent => InstanceError::LayerNotPresent,
            ash::vk::Result::ErrorExtensionNotPresent => InstanceError::ExtensionNotPresent,
            ash::vk::Result::ErrorIncompatibleDriver => InstanceError::IncompatibleDriver,
            _ => panic!("Unexpected error value"),
        }
    }
}

/// Possible errors returned by `Device`.
#[derive(Clone, Debug, Fail)]
pub enum DeviceError {
    #[fail(display = "Failed to load device functions {:?}", _0)]
    LoadError(Vec<&'static str>),

    #[fail(display = "{}", _0)]
    OomError(OomError),

    #[fail(display = "{}", _0)]
    DeviceLost(DeviceLost),

    #[fail(display = "Initialization failed")]
    InitializationFailed,

    #[fail(display = "Extension not present")]
    ExtensionNotPresent,

    #[fail(display = "Feature not present")]
    FeatureNotPresent,

    #[fail(display = "Too many objects")]
    TooManyObjects,
}

impl DeviceError {
    fn from_device_error(error: ash::DeviceError) -> Self {
        match error {
            ash::DeviceError::LoadError(names) => DeviceError::LoadError(names),
            ash::DeviceError::VkError(result) => DeviceError::from_vk_result(result),
        }
    }

    fn from_vk_result(result: ash::vk::Result) -> Self {
        match result {
            ash::vk::Result::ErrorOutOfHostMemory => DeviceError::OomError(OomError::OutOfHostMemory),
            ash::vk::Result::ErrorOutOfDeviceMemory => DeviceError::OomError(OomError::OutOfDeviceMemory),
            ash::vk::Result::ErrorDeviceLost => DeviceError::DeviceLost(DeviceLost),
            ash::vk::Result::ErrorInitializationFailed => DeviceError::InitializationFailed,
            ash::vk::Result::ErrorExtensionNotPresent => DeviceError::ExtensionNotPresent,
            ash::vk::Result::ErrorFeatureNotPresent => DeviceError::FeatureNotPresent,
            ash::vk::Result::ErrorTooManyObjects => DeviceError::TooManyObjects,
            _ => panic!("Unexpected result value"),
        }
    }
}

pub(crate) struct InnerInstance {
    pub(crate) raw: ash::Instance<ash::version::V1_0>,
    pub(crate) surface: Option<SurfaceFn>,
}

impl Drop for InnerInstance {
    fn drop(&mut self) {
        unsafe {
            self.raw.destroy_instance(None)
        }
    }
}

/// Vulkan instance.
#[derive(Clone)]
pub struct Instance {
    pub(crate) inner: Arc<InnerInstance>,
}

impl Deref for Instance {
    type Target = ash::Instance<ash::version::V1_0>;

    fn deref(&self) -> &ash::Instance<ash::version::V1_0> {
        &self.inner.raw
    }
}

impl Instance {
    /// Create vulkan instance.
    pub fn new<F>(configure: F) -> Result<Instance, InstanceError>
    where
        F: FnOnce(&[Layer], &[Extension]) -> InstanceConfig,
    {
        let entry = ash::Entry::<ash::version::V1_0>::new().map_err(InstanceError::from_loading_error)?;
        let layer_properties = entry.enumerate_instance_layer_properties().map_err(InstanceError::from_vk_result)?;
        let extension_properties = entry.enumerate_instance_extension_properties().map_err(InstanceError::from_vk_result)?;
        let surface_enabled;

        trace!("Properties and extensions fetched");
        let instance = unsafe {
            let layers = layer_properties.iter().map(|layer| Layer {
                name: CStr::from_ptr(&layer.layer_name[0]).to_str().unwrap(),
                spec_version: layer.spec_version,
                implementation_version: layer.implementation_version,
                description: CStr::from_ptr(&layer.description[0]).to_str().unwrap(),
            }).collect::<Vec<_>>();

            let extensions = extension_properties.iter().map(|extension| Extension {
                name: CStr::from_ptr(&extension.extension_name[0]).to_str().unwrap(),
                spec_version: extension.spec_version,
            }).collect::<Vec<_>>();

            let config = configure(&layers, &extensions);

            trace!("Config acquired");
            let app_name = CString::new(config.app_name).unwrap();
            let engine_name = CString::new("rendy").unwrap();
            let layers: Vec<CString> = config.layers.into_iter().map(|s| CString::new(s).unwrap()).collect();
            let extensions: Vec<CString> = config.extensions.into_iter().map(|s| CString::new(s).unwrap()).collect();

            surface_enabled = SurfaceFn::extensions()
                .iter()
                .all(|&surface_extension| {
                    extensions.iter()
                        .find(|&name| &**name == surface_extension).is_some()
                })
            ;

            let enabled_layers: Vec<*const ash::vk::c_char> = layers.iter().map(|s| s.as_ptr()).collect();
            let enabled_extensions: Vec<*const ash::vk::c_char> = extensions.iter().map(|s| s.as_ptr()).collect();

            entry.create_instance(
                &ash::vk::InstanceCreateInfo {
                    s_type: ash::vk::StructureType::InstanceCreateInfo,
                    p_next: null(),
                    flags: ash::vk::InstanceCreateFlags::empty(),
                    p_application_info: &ash::vk::ApplicationInfo {
                        s_type: ash::vk::StructureType::ApplicationInfo,
                        p_next: null(),
                        p_application_name: app_name.as_ptr(),
                        application_version: config.app_version,
                        p_engine_name: engine_name.as_ptr(),
                        engine_version: 1,
                        api_version: vk_make_version!(1, 0, 0),
                    },
                    enabled_layer_count: enabled_layers.len() as u32,
                    pp_enabled_layer_names: enabled_layers.as_ptr(),
                    enabled_extension_count: enabled_extensions.len() as u32,
                    pp_enabled_extension_names: enabled_extensions.as_ptr(),
                },
                None
            ).map_err(InstanceError::from_instance_error)?
        };
        trace!("Vulkan instance created");

        let surface = if surface_enabled {
            Some(SurfaceFn::new(instance.handle(), entry.static_fn()).map_err(InstanceError::LoadError)?)
        } else {
            None
        };

        Ok(Instance {
            inner: Arc::new(InnerInstance {
                raw: instance,
                surface,
            })
        })
    }
}

pub struct PhysicalDevice<'a> {
    pub(crate) instance: &'a Instance,
    pub(crate) raw: ash::vk::PhysicalDevice,
}

impl<'a> PhysicalDevice<'a> {
    /// Enumerate physical devices
    pub fn enumerate(instance: &Instance) -> Result<impl IntoIterator<Item = PhysicalDevice>, InstanceError> {
        let physicals = instance.enumerate_physical_devices().map_err(InstanceError::from_vk_result)?;
        trace!("Physical device enumerated");
        Ok(physicals.into_iter().map(move |physical| PhysicalDevice { instance, raw: physical, }))
    }

    pub fn properties(&self) -> PhysicalDeviceProperties {
        let properties = self.instance.get_physical_device_properties(self.raw);
        PhysicalDeviceProperties {
            api_version: properties.api_version,
            driver_version: properties.driver_version,
            vendor_id: properties.vendor_id,
            device_id: properties.device_id,
            device_type: properties.device_type,
            device_name: unsafe { CStr::from_ptr(&properties.device_name[0]).to_str().unwrap().to_string() },
            pipeline_cache_uuid: properties.pipeline_cache_uuid,
            limits: properties.limits.clone(),
            sparse_properties: properties.sparse_properties.clone(),
        }
    }

    pub fn families(&self) -> impl IntoIterator<Item = QueueFamilyProperties> {
        self.instance
            .get_physical_device_queue_family_properties(self.raw)
            .into_iter()
            .enumerate()
            .map(|(index, properties)| QueueFamilyProperties {
                index: index as u32,
                capability: properties.queue_flags.into(),
                queue_count: properties.queue_count,
            })
    }

    pub fn extensions(&self) -> Result<impl IntoIterator<Item = String>, InstanceError> {
        let properties = self.instance.enumerate_device_extension_properties(self.raw).map_err(InstanceError::from_vk_result)?;

        Ok(
            properties
                .into_iter()
                .map(|extension| unsafe {
                    CStr::from_ptr(&extension.extension_name[0]).to_str().unwrap().to_string()
                })
        )
    }

    pub fn features(&self) -> ash::vk::PhysicalDeviceFeatures {
        self.instance.get_physical_device_features(self.raw)
    }
}

pub struct Device {
    pub(crate) fp: Arc<ash::vk::DeviceFnV1_0>,
    pub(crate) raw: ash::vk::Device,
    pub(crate) instance: Instance,
    pub(crate) physical: ash::vk::PhysicalDevice,
    pub(crate) families: Vec<command::Family>,
    pub(crate) terminal: Terminal,
    pub(crate) tracker: Option<DeviceTracker>,
    pub(crate) swapchain: Option<ash::vk::SwapchainFn>,
}

impl Device {
    /// Create device from given physical device.
    pub fn create<Q, E>(physical_device: PhysicalDevice, families: Q, extensions: E, features: ash::vk::PhysicalDeviceFeatures) -> Result<Self, DeviceError>
    where
        Q: IntoIterator,
        Q::Item: Borrow<CreateQueueFamily>,
        E: IntoIterator<Item = String>,
    {
        debug!("Create device for physical device: {:#?}", physical_device.properties());

        let families = families.into_iter().map(|cqi| cqi.borrow().clone()).collect::<Vec<_>>();

        debug!("Families for create: {:#?}", &families);

        let mut max_queues = families.iter().map(|cqi| cqi.count).max().unwrap_or(0);
        let priorities = vec![1f32; max_queues as usize];

        let mut queue_create_infos = families.iter().map(|cqi| {
            ash::vk::DeviceQueueCreateInfo {
                s_type: ash::vk::StructureType::DeviceQueueCreateInfo,
                p_next: null(),
                flags: ash::vk::DeviceQueueCreateFlags::empty(),
                queue_family_index: cqi.family,
                queue_count: cqi.count,
                p_queue_priorities: priorities.as_ptr(),
            }
        }).collect::<Vec<_>>();

        let extensions = extensions.into_iter().map(|extension| CString::new(extension).unwrap()).collect::<Vec<_>>();

        let swapchain_enabled = extensions.iter().find(|&name| &**name == ash::extensions::Swapchain::name()).is_some();

        debug!("Enabling extensions: {:#?}", &extensions);

        let enabled_extensions = extensions.iter().map(|string| string.as_ptr()).collect::<Vec<_>>();

        let device = unsafe {
            physical_device.instance.create_device(
                physical_device.raw,
                &ash::vk::DeviceCreateInfo {
                    s_type: ash::vk::StructureType::DeviceCreateInfo,
                    p_next: null(),
                    flags: ash::vk::DeviceCreateFlags::empty(),
                    queue_create_info_count: queue_create_infos.len() as u32,
                    p_queue_create_infos: queue_create_infos.as_ptr(),
                    enabled_layer_count: 0,
                    pp_enabled_layer_names: null(),
                    enabled_extension_count: enabled_extensions.len() as u32,
                    pp_enabled_extension_names: enabled_extensions.as_ptr() as _,
                    p_enabled_features: &features,
                },
                None,
            ).map_err(DeviceError::from_device_error)?
        };

        let fp = Arc::new(device.fp_v1_0().clone());
        let raw = device.handle();
        trace!("Device {:?} created", raw);

        let swapchain = if swapchain_enabled {
            Some(ash::vk::SwapchainFn::load(|name| unsafe {
                ::std::mem::transmute(physical_device.instance.get_device_proc_addr(
                    raw,
                    name.as_ptr(),
                ))
            }).map_err(DeviceError::LoadError)?)
        } else {
            None
        };

        let families = families.iter().map(|cqi| {
            let id = command::FamilyId {
                index: cqi.family,
                capability: physical_device.families().into_iter().nth(cqi.family as usize).unwrap().capability,
            };
            unsafe { // Uses same values that was used in `Instance::create_device` method.
                command::Family::from_device(fp.clone(), raw, id, cqi.count)
            }
        }).collect::<Vec<_>>();

        Ok(Device {
            fp,
            raw,
            instance: physical_device.instance.clone(),
            physical: physical_device.raw,
            families,
            terminal: Terminal::new(),
            tracker: Some(DeviceTracker {
                relevant: Relevant,
                device: raw,
            }),
            swapchain,
        })
    }


    fn instance(&self) -> &Instance {
        &self.instance
    }

    fn physical_device(&self) -> ash::vk::PhysicalDevice {
        self.physical
    }

    fn device(&self) -> ash::vk::Device {
        self.raw
    }

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

    /// Take resource tracker from the device.
    /// `DeviceTracker` is unique for `Device`.
    /// It can't be taken again until returned.
    pub fn take_tracker(&mut self) -> Option<DeviceTracker> {
        self.tracker.take()
    }

    /// Return taken `DeviceTracker`.
    /// User should return `DeviceTracker` after submitting all one-shot command buffers that used it.
    pub fn return_tracker(&mut self, tracker: DeviceTracker) {
        // assert_eq!(tracker.device, self.raw); `Eq` must be implemented for handles.
        debug_assert!(self.tracker.is_none());
        self.tracker = Some(tracker);
    }

    /// Cleanup entities.
    /// This function is expected to be called one in a while to free memory.
    /// This function expects that `DeviceTracker` wasn't taken after returned last time.
    /// Otherwise it can't guarantee that resources are not in use by the device before deleting them.
    pub fn cleanup(&mut self) {
        if self.tracker.is_some() {
            let objects = Arc::new(self.terminal.drain().collect::<VulkanObjects>());
            for queue in self.families.iter_mut().flat_map(command::Family::queues) {
                queue.push_track(objects.clone());
            }
            trace!("Resources cleaned");
        } else {
            warn!("Failed to cleanup resources. `DeviceTracker` is not returned");
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        self.families.clear();
        trace!("Queues stopped");
        unsafe {
            trace!("Objects destroyed");
            self.fp.destroy_device(self.raw, null());
            trace!("Device destroyed");
            self.instance.destroy_instance(None);
            trace!("Instance destroyed");

            let tracker = self.tracker.take().expect("Tracker must be returned");
            tracker.relevant.dispose();
        }
    }
}

/// Device resource tracker.
/// This object catches dropped resources
/// and ensures that they aren't used by device before actually destroying them.
/// It can preserve a resource for longer time than needed
/// but never destroys resource before device stops using it.
#[derive(Clone, Debug)]
pub struct DeviceTracker {
    relevant: Relevant,
    device: ash::vk::Device,
}

