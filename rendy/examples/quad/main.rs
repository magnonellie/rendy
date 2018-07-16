

extern crate env_logger;
#[macro_use]
extern crate log;
extern crate rendy;

use std::fmt::Debug;

use rendy::core::{command::Capability, factory::{Factory, Config, CreateQueueFamily, QueueFamilyProperties}};

fn pick_first_graphics_family(families: &[QueueFamilyProperties]) -> Vec<CreateQueueFamily> {
    let (index, family) = families.iter().enumerate().find(|&(_, ref family)| family.capability.supports(Capability::Graphics)).unwrap();
    vec![CreateQueueFamily {
        family: index as u32,
        count: 1,
    }]
}

fn run() -> Result<(), String> {
    let factory = Factory::build()
        .load().map_err(format_error)?
        .instantiate(|layers, extensions| Config {
            app_name: "Rendy's quad example".into(),
            app_version: 1,
            layers: layers.iter().map(|layer| layer.name.to_string()).collect(),
            extensions: Vec::new(),
        }).map_err(format_error)?
        .with_device(|_| 0, pick_first_graphics_family, |_| false, |features| features)
    ;

    trace!("Finish");
    Ok(())
}


fn main() {
    env_logger::init();
    run().unwrap_or_else(|err| error!("Failed with error: {}", err));
}

fn format_error<E>(error: E) -> String
where
    E: Debug,
{
    format!("{:?}", error)
}