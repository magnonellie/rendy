
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

pub struct Config {
    app_name: String,
    app_version: u32,
    layer_names: Vec<String>,
    extension_names: Vec<String>,
}

/// Loads Vulkan and builds factory step by step.
pub struct FactoryBuilder;

impl FactoryBuilder {
    fn new() -> Self {
        FactoryBuilder
    }

    /// Load Vulkan implementation.
    pub fn load<'a, F>(self) -> Result<FactoryLoaded, ash::LoadingError> {
        Ok(FactoryLoaded {
            entry: Entry::new()?,
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
        F: FnOnce(&[ash::vk::LayerProperties], &[ash::vk::ExtensionProperties]) -> Config,
    {
        let layer_properties = self.entry.enumerate_instance_layer_properties().map_err(ash::InstanceError::VkError)?;
        let extension_properties = self.entry.enumerate_instance_extension_properties().map_err(ash::InstanceError::VkError)?;

        let config = configure(&layer_properties, &extension_properties);
        let mut app_name = CString::new(config.app_name).unwrap();
        let engine_name = CString::new("rendy").unwrap();
        let layer_names: Vec<CString> = config.layer_names.into_iter().map(|s| CString::new(s).unwrap()).collect();
        let extension_names: Vec<CString> = config.extension_names.into_iter().map(|s| CString::new(s).unwrap()).collect();

        let pp_enabled_layer_names: Vec<*const ash::vk::c_char> = layer_names.iter().map(|s| s.as_ptr()).collect();
        let pp_enabled_layer_names: Vec<*const ash::vk::c_char> = layer_names.iter().map(|s| s.as_ptr()).collect();

        let instance = unsafe {
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
                    enabled_layer_count: pp_enabled_layer_names.len() as u32,
                    pp_enabled_layer_names: pp_enabled_layer_names.as_ptr(),
                    enabled_extension_count: pp_enabled_layer_names.len() as u32,
                    pp_enabled_extension_names: pp_enabled_layer_names.as_ptr(),
                },
                None
            )?
        };

        Ok(FactoryInstantiated {
            instance,
        })
    }
}

pub struct FactoryInstantiated {
    instance: Instance,
}

pub struct CreateQueueFamilyInfo {
    family: u32,
    count: u32,
}

impl FactoryInstantiated {
    /// Create device.
    pub fn with_device<P, Q, E, F>(self, physical: P, families: Q, mut extensions: E, features: F) -> Result<Factory, ash::DeviceError>
    where
        P: FnOnce(&[ash::vk::PhysicalDeviceProperties]) -> usize,
        E: FnMut(&str) -> bool,
        Q: FnOnce(&[ash::vk::QueueFamilyProperties]) -> Vec<CreateQueueFamilyInfo>,
        F: FnOnce(ash::vk::PhysicalDeviceFeatures) -> ash::vk::PhysicalDeviceFeatures,
    {
        let mut physicals = self.instance.enumerate_physical_devices().map_err(ash::DeviceError::VkError)?;
        let properties = physicals.iter().map(|&physical| self.instance.get_physical_device_properties(physical)).collect::<Vec<_>>();
        let physical = physicals.swap_remove(physical(&properties));

        let extension_properties = self.instance.enumerate_device_extension_properties(physical).map_err(ash::DeviceError::VkError)?;
        let device_features = self.instance.get_physical_device_features(physical);
        let queue_properties = self.instance.get_physical_device_queue_family_properties(physical);
        let memory_properties = self.instance.get_physical_device_memory_properties(physical);

        let enabled_extension = extension_properties.iter().filter_map(|extension| {
            if extension.spec_version <= vk_make_version!(1, 0, 0) {
                let name = unsafe {
                    CStr::from_ptr(&extension.extension_name[0]).to_str().unwrap()
                };
                if extensions(name) {
                    Some(name.as_ptr())
                } else {
                    None
                }
            } else {
                None
            }
        }).collect::<Vec<*const _>>();

        let families = families(&queue_properties);

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

        let device_features = features(device_features);

        let device = unsafe {
            self.instance.create_device(
                physical,
                &ash::vk::DeviceCreateInfo {
                    s_type: ash::vk::StructureType::DeviceCreateInfo,
                    p_next: null(),
                    flags: ash::vk::DeviceCreateFlags::empty(),
                    queue_create_info_count: queue_create_infos.len() as u32,
                    p_queue_create_infos: queue_create_infos.as_ptr(),
                    enabled_layer_count: 0,
                    pp_enabled_layer_names: null(),
                    enabled_extension_count: enabled_extension.len() as u32,
                    pp_enabled_extension_names: enabled_extension.as_ptr() as _,
                    p_enabled_features: &device_features,
                },
                None,
            )?
        };

        let device = (Arc::new(device.fp_v1_0().clone()), device.handle());

        Ok(Factory {
            instance: self.instance,
            physical,
            families: families.iter().map(|cqi| {
                let id = command::FamilyId {
                    index: cqi.family,
                    capability: queue_properties[cqi.family as usize].queue_flags.into(),
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