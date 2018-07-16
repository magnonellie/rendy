use ash;

#[derive(Clone, Copy, Debug, Fail)]
#[fail(display = "Device lost")]
pub struct DeviceLost;

impl DeviceLost {
    pub(crate) fn from_vk_result(result: ash::vk::Result) -> Self {
        match result {
            ash::vk::Result::ErrorDeviceLost => DeviceLost,
            _ => panic!("Unexpected result value"),
        }
    }
}

#[derive(Clone, Copy, Debug, Fail)]
#[fail(display = "Surface lost")]
pub struct SurfaceLost;

impl SurfaceLost {
    pub(crate) fn from_vk_result(result: ash::vk::Result) -> Self {
        match result {
            ash::vk::Result::ErrorSurfaceLostKhr => SurfaceLost,
            _ => panic!("Unexpected result value"),
        }
    }
}

/// Out of memory error.
#[derive(Clone, Copy, Debug, Fail)]
pub enum OomError {
    /// Host memory exhausted.
    #[fail(display = "Out of host memory")]
    OutOfHostMemory,

    /// Device memory exhausted.
    #[fail(display = "Out of device memory")]
    OutOfDeviceMemory,
}

impl OomError {
    pub(crate) fn from_vk_result(result: ash::vk::Result) -> Self {
        match result {
            ash::vk::Result::ErrorOutOfHostMemory => OomError::OutOfHostMemory,
            ash::vk::Result::ErrorOutOfDeviceMemory => OomError::OutOfDeviceMemory,
            _ => panic!("Unexpected result value"),
        }
    }
}

/// Possible errors returned by `Instance` and `PhysicalDevice`.
#[derive(Clone, Debug, Fail)]
pub enum InstanceError {
    #[fail(display = "Failed to load Vulkan library {}", _0)]
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
    pub(crate) fn from_loading_error(error: ash::LoadingError) -> Self {
        match error {
            ash::LoadingError::LibraryLoadError(name) => InstanceError::LibraryLoadError(name),
            ash::LoadingError::EntryLoadError(names) => InstanceError::LoadError(names),
            ash::LoadingError::StaticLoadError(names) => InstanceError::LoadError(names),
        }
    }

    pub(crate) fn from_instance_error(error: ash::InstanceError) -> Self {
        match error {
            ash::InstanceError::LoadError(names) => InstanceError::LoadError(names),
            ash::InstanceError::VkError(result) => InstanceError::from_vk_result(result),
        }
    }

    pub(crate) fn from_vk_result(result: ash::vk::Result) -> Self {
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
    pub(crate) fn from_device_error(error: ash::DeviceError) -> Self {
        match error {
            ash::DeviceError::LoadError(names) => DeviceError::LoadError(names),
            ash::DeviceError::VkError(result) => DeviceError::from_vk_result(result),
        }
    }

    pub(crate) fn from_vk_result(result: ash::vk::Result) -> Self {
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

#[derive(Clone, Debug, Fail)]
pub enum SurfaceError {
    #[fail(display = "{}", _0)]
    OomError(OomError),

    #[fail(display = "{}", _0)]
    DeviceLost(DeviceLost),

    #[fail(display = "Surface lost")]
    SurfaceLost(SurfaceLost),

    #[fail(display = "Native window in use")]
    WindowInUse,
}

impl SurfaceError {
    pub(crate) fn from_vk_result(result: ash::vk::Result) -> Self {
        match result {
            ash::vk::Result::ErrorOutOfHostMemory => SurfaceError::OomError(OomError::OutOfHostMemory),
            ash::vk::Result::ErrorOutOfDeviceMemory => SurfaceError::OomError(OomError::OutOfDeviceMemory),
            ash::vk::Result::ErrorDeviceLost => SurfaceError::DeviceLost(DeviceLost),
            ash::vk::Result::ErrorSurfaceLostKhr => SurfaceError::SurfaceLost(SurfaceLost),
            ash::vk::Result::ErrorNativeWindowInUseKhr => SurfaceError::WindowInUse,
            _ => panic!("Unexpected result value"),
        }
    }
}
