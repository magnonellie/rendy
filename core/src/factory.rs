
use std::{any::Any, borrow::Borrow, collections::LinkedList, ffi::{CString, CStr}, ops::Range, ptr::null, sync::Arc};
use ash::{self, version::{DeviceV1_0, EntryV1_0, InstanceV1_0}};

use buffer;
use command;
use escape::Terminal;
use format;
use image;
use memory;
use object::VulkanObjects;
use tracker::GlobalTracker;


/// Vulkan entry functions.
type Entry = ash::Entry<ash::version::V1_0>;

/// Vulkan entry functions.
type Instance = ash::Instance<ash::version::V1_0>;

/// Vulkan entry functions.
type Device = ash::Device<ash::version::V1_0>;



#[derive(Clone, Debug)]
pub struct Config {
    pub app_name: String,
    pub app_version: u32,
    pub layers: Vec<String>,
    pub extensions: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
pub struct Layer<'a> {
    pub name: &'a str,
    pub spec_version: u32,
    pub implementation_version: u32,
    pub description: &'a str,
}

#[derive(Clone, Copy, Debug)]
pub struct Extension<'a> {
    pub name: &'a str,
    pub spec_version: u32,
}

#[derive(Clone, Debug)]
pub struct PhysicalDevice<'a> {
    pub api_version: u32,
    pub driver_version: u32,
    pub vendor_id: u32,
    pub device_id: u32,
    pub device_type: ash::vk::PhysicalDeviceType,
    pub device_name: &'a str,
    pub pipeline_cache_uuid: [u8; 16],
    pub limits: ash::vk::PhysicalDeviceLimits,
    pub sparse_properties: ash::vk::PhysicalDeviceSparseProperties,
}

#[derive(Clone, Copy, Debug)]
pub struct QueueFamilyProperties {
    pub capability: command::Capability,
    pub queue_count: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct CreateQueueFamily {
    pub family: u32,
    pub count: u32,
}


/// Loads Vulkan and builds factory step by step.
pub struct FactoryBuilder;

impl FactoryBuilder {
    fn new() -> Self {
        FactoryBuilder
    }

    /// Load Vulkan implementation.
    pub fn load(self) -> Result<FactoryLoaded, ash::LoadingError> {
        let entry = Entry::new()?;
        info!("Vulkan entry loaded");
        Ok(FactoryLoaded {
            entry,
        })
    }
}

pub struct FactoryLoaded {
    entry: Entry,
}

impl FactoryLoaded {
    /// Create vulkan instance.
    pub fn instantiate<F>(self, configure: F) -> Result<FactoryInstantiated, ash::InstanceError>
    where
        F: FnOnce(&[Layer], &[Extension]) -> Config,
    {
        let layer_properties = self.entry.enumerate_instance_layer_properties().map_err(ash::InstanceError::VkError)?;
        let extension_properties = self.entry.enumerate_instance_extension_properties().map_err(ash::InstanceError::VkError)?;

        debug!("Properties and extensions fetched");
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

            debug!("Config acquired");
            let app_name = CString::new(config.app_name).unwrap();
            let engine_name = CString::new("rendy").unwrap();
            let layers: Vec<CString> = config.layers.into_iter().map(|s| CString::new(s).unwrap()).collect();
            let extensions: Vec<CString> = config.extensions.into_iter().map(|s| CString::new(s).unwrap()).collect();

            let enabled_layers: Vec<*const ash::vk::c_char> = layers.iter().map(|s| s.as_ptr()).collect();
            let enabled_extensions: Vec<*const ash::vk::c_char> = extensions.iter().map(|s| s.as_ptr()).collect();

            self.entry.create_instance(
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
            )?
        };

        info!("Vulkan instance created");
        Ok(FactoryInstantiated {
            instance,
        })
    }
}

pub struct FactoryInstantiated {
    instance: Instance,
}

impl FactoryInstantiated {
    /// Create device.
    pub fn with_device<P, Q, E, F>(self, pick_physical: P, pick_families: Q, mut pick_extensions: E, pick_features: F) -> Result<Factory, ash::DeviceError>
    where
        P: FnOnce(&[PhysicalDevice]) -> usize,
        Q: FnOnce(&[QueueFamilyProperties]) -> Vec<CreateQueueFamily>,
        E: FnMut(&str) -> bool,
        F: FnOnce(ash::vk::PhysicalDeviceFeatures) -> ash::vk::PhysicalDeviceFeatures,
    {
        let mut physicals = self.instance.enumerate_physical_devices().map_err(ash::DeviceError::VkError)?;
        let properties = physicals.iter().map(|&physical| self.instance.get_physical_device_properties(physical)).collect::<Vec<_>>();
        let queue_properties;
        let memory_properties;
        let families;

        let (device, physical) = unsafe {
            let properties = properties.iter().map(|physical| PhysicalDevice {            
                api_version: physical.api_version,
                driver_version: physical.driver_version,
                vendor_id: physical.vendor_id,
                device_id: physical.device_id,
                device_type: physical.device_type,
                device_name: CStr::from_ptr(&physical.device_name[0]).to_str().unwrap(),
                pipeline_cache_uuid: physical.pipeline_cache_uuid,
                limits: physical.limits.clone(),
                sparse_properties: physical.sparse_properties.clone(),
            }).collect::<Vec<_>>();

            debug!("Physical devices fetched");
            trace!("Physical device properties: {:?}", properties);
            let picked = pick_physical(&properties);
            let physical = physicals.swap_remove(picked);
            info!("Physical device '{}' picked", picked);
            
            queue_properties = self.instance.get_physical_device_queue_family_properties(physical)
                .into_iter()
                .map(|properties| QueueFamilyProperties {
                    capability: properties.queue_flags.into(),
                    queue_count: properties.queue_count,
                })
                .collect::<Vec<_>>();
            trace!("Queues: {:?}", queue_properties);

            families = pick_families(&queue_properties);

            let priorities = vec![1f32; families.iter().max_by_key(|cqi| cqi.count).map_or(0, |cqi| cqi.count) as usize];

            let queue_create_infos = families.iter().map(|cqi| {
                ash::vk::DeviceQueueCreateInfo {
                    s_type: ash::vk::StructureType::DeviceQueueCreateInfo,
                    p_next: null(),
                    flags: ash::vk::DeviceQueueCreateFlags::empty(),
                    queue_family_index: cqi.family,
                    queue_count: cqi.count,
                    p_queue_priorities: priorities.as_ptr(),
                }
            }).collect::<Vec<_>>();

            let extension_properties = self.instance.enumerate_device_extension_properties(physical).map_err(ash::DeviceError::VkError)?;
            trace!("Extensions: {:?}", extension_properties);

            let enabled_extensions = extension_properties.iter().filter_map(|extension| {
                if extension.spec_version <= vk_make_version!(1, 0, 0) {
                    let name = unsafe {
                        CStr::from_ptr(&extension.extension_name[0]).to_str().unwrap()
                    };
                    if pick_extensions(name) {
                        Some(name.as_ptr())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }).collect::<Vec<*const _>>();

            let device_features = self.instance.get_physical_device_features(physical);
            trace!("Features: {:?}", device_features);

            let device_features = pick_features(device_features);

            memory_properties = self.instance.get_physical_device_memory_properties(physical);
            trace!("Memory: {:?}", memory_properties);

            let device = self.instance.create_device(
                physical,
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
                    p_enabled_features: &device_features,
                },
                None,
            )?;
            (device, physical)
        };

        let device = (Arc::new(device.fp_v1_0().clone()), device.handle());

        Ok(Factory {
            instance: self.instance,
            physical,
            families: families.iter().map(|cqi| {
                let id = command::FamilyId {
                    index: cqi.family,
                    capability: queue_properties[cqi.family as usize].capability,
                };
                unsafe { // Uses same values that was used in `Instance::create_device` method.
                    command::Family::from_device(device.0.clone(), device.1, id, cqi.count)
                }
            }).collect(),
            terminal: Terminal::new(),
            device,
        })
    }
}


/// Factory contains devices and queues.
/// It abstracts some details to provide safer API.
/// Yet it safely exposes internal objects because using them requires `unsafe`.
pub struct Factory {
    instance: Instance,
    physical: ash::vk::PhysicalDevice,
    device: (Arc<ash::vk::DeviceFnV1_0>, ash::vk::Device),
    families: Vec<command::Family>,
    terminal: Terminal,
}

impl Factory {
    fn instance(&self) -> ash::vk::Instance {
        self.instance.handle()
    }

    fn physical_device(&self) -> ash::vk::PhysicalDevice {
        self.physical
    }

    fn device(&self) -> ash::vk::Device {
        self.device.1
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
}

impl Factory {

    pub fn build() -> FactoryBuilder {
        FactoryBuilder
    }

    /// Take tracked resources from `GlobalTracker` and hold them until device stop using them.
    pub fn collect_from_tracker(&mut self, tracker: &mut GlobalTracker) {
        let objects = Arc::new(self.terminal.drain().collect::<VulkanObjects>());
        for queue in self.families.iter_mut().flat_map(command::Family::queues) {
            queue.push_track(objects.clone());
        }
    }
}

impl Drop for Factory {
    fn drop(&mut self) {
        self.families.clear();
        trace!("Queues stopped");
        unsafe {
            for object in self.terminal.drain() {
                object.destroy(&self.device.0, self.device.1);
            }
            trace!("Objects destroyed");
            self.device.0.destroy_device(self.device.1, null());
            trace!("Device destroyed");
            self.instance.destroy_instance(None);
            trace!("Instance destroyed");
        }
    }
}
