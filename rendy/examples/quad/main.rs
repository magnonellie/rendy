
extern crate env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;
extern crate rendy;
extern crate winit;

use std::{fmt::Debug, iter::{once, empty}};
use failure::Error;

use rendy::{command::Capability, device::{Instance, PhysicalDevice, Device, InstanceConfig, CreateQueueFamily}, surface::Surface, swapchain::Swapchain};


fn main() -> Result<(), Error> {
    env_logger::Builder::from_default_env()
        .filter_module("rendy", log::LevelFilter::Trace)
        .filter_module("rendy_core", log::LevelFilter::Trace)
        .filter_module("quad", log::LevelFilter::Trace)
        .init()
    ;

    let mut events_loop = winit::EventsLoop::new();

    trace!("Creating window");
    let window = winit::WindowBuilder::new()
        .with_title("Rendy Quad Example")
        .with_dimensions((640, 480).into())
        .with_visibility(true)
        .build(&events_loop)?
    ;
    events_loop.poll_events(|_| ());

    let surface_extensions = Surface::extensions();
    let swapchain_extensions = Swapchain::extensions();

    trace!("Creating Instance");
    let instance = Instance::new(|layers, extensions| {
        debug!("Instance layers: {:#?}", layers);
        debug!("Instance extensions: {:#?}", extensions);

        assert!(
            surface_extensions.iter().all(|&surface_extension| extensions.iter().find(|extension| extension.name == surface_extension).is_some())
        );

        assert!(
            swapchain_extensions.iter().all(|&swapchain_extension| extensions.iter().find(|extension| extension.name == swapchain_extension).is_some())
        );

        InstanceConfig {
            app_name: "Quad example".into(),
            app_version: 1,
            layers: layers.iter().map(|layer| layer.name.into()).collect(),
            extensions: surface_extensions.into_iter().chain(swapchain_extensions).map(String::from).collect(),
        }
    })?;

    trace!("Create surface");
    let surface = Surface::create(&instance, window)?;

    trace!("Picking physical device");
    let physical_device = PhysicalDevice::enumerate(&instance)?
        .into_iter()
        .max_by_key(|physical| match physical.properties().device_type {
            rendy::ash::vk::PhysicalDeviceType::Other => 0,
            rendy::ash::vk::PhysicalDeviceType::Cpu => 1,
            rendy::ash::vk::PhysicalDeviceType::VirtualGpu => 2,
            rendy::ash::vk::PhysicalDeviceType::IntegratedGpu => 3,
            rendy::ash::vk::PhysicalDeviceType::DiscreteGpu => 4,
        })
        .ok_or(format_err!("No physical devices"))?;

    trace!("Picking family");
    let family = physical_device.families()
        .into_iter()
        .find(|family| {
            surface.supports_queue_family(&physical_device, family.index) &&
            family.capability.supports(rendy::command::Capability::Graphics)
        })
        .map(|family| CreateQueueFamily {
            family: family.index,
            count: 1,
        })
        .ok_or(format_err!("Can't find any graphics queues"))?;

    trace!("Creating device");
    let device = Device::create(physical_device, once(family), empty(), Default::default())?;

    trace!("Upload resources");
    trace!("Create pipeline");

    Ok(())
}
