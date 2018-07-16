
extern crate env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;
extern crate rendy;

use std::{fmt::Debug, iter::{once, empty}};
use failure::Error;

use rendy::{command::Capability, device::{Instance, InstanceConfig, Device, CreateQueueFamily}};

fn main() -> Result<(), Error> {
    env_logger::Builder::from_default_env()
        .filter_module("rendy", log::LevelFilter::Trace)
        .filter_module("rendy_core", log::LevelFilter::Trace)
        .filter_module("quad", log::LevelFilter::Trace)
        .init()
    ;
    
    let instance = Instance::new(|layers, _| InstanceConfig {
        app_name: "Quad example".into(),
        app_version: 1,
        layers: layers.iter().map(|layer| layer.name.into()).collect(),
        extensions: Vec::new(),
    })?;

    let physical_device = instance.enumerate_physical_devices()?
        .into_iter()
        .max_by_key(|physical| match physical.properties().device_type {
            rendy::ash::vk::PhysicalDeviceType::Other => 0,
            rendy::ash::vk::PhysicalDeviceType::Cpu => 1,
            rendy::ash::vk::PhysicalDeviceType::VirtualGpu => 2,
            rendy::ash::vk::PhysicalDeviceType::IntegratedGpu => 3,
            rendy::ash::vk::PhysicalDeviceType::DiscreteGpu => 4,
        })
        .ok_or(format_err!("No physical devices"))?;

    let family = physical_device.families()
        .into_iter()
        .enumerate()
        .find(|&(_, family)| family.capability.supports(rendy::command::Capability::Graphics))
        .map(|(index, _)| CreateQueueFamily {
            family: index as u32,
            count: 1,
        })
        .ok_or(format_err!("Can't find any graphics queues"))?;

    let device = Device::create(physical_device, once(family), empty(), Default::default())?;

    Ok(())
}
