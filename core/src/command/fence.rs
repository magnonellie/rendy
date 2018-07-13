
use std::iter::FromIterator;
use ash;
use relevant::Relevant;
use smallvec::SmallVec;

use {DeviceLostOrOomError, DeviceLost, OomError};
use command::{Capability, QueueId};

/// Fence that wasn't submitted to the queue
pub struct UnarmedFence {
    pub(crate) raw: ash::vk::Fence,
}

/// Fence that was submitted to the queue
pub struct ArmedFence<C = Capability> {
    pub(crate) raw: ash::vk::Fence,
    pub(crate) queue: QueueId<C>,
    pub(crate) epoch: u64,
    pub(crate) relevant: Relevant,
}

/// Fence that was successfully checked for signalling.
pub struct ReadyFence<C = Capability> {
    pub(crate) raw: ash::vk::Fence,
    pub(crate) queue: QueueId<C>,
    pub(crate) epoch: u64,
    pub(crate) relevant: Relevant,
}

/// Flag to wait for...
pub enum WaitFor {
    /// At least one fence.
    Any,

    /// All fences.
    All,
}

/// Wait for all fences.
/// Returns collection of `ReadyFence` if all fences are signalled.
/// Returns collection of `ArmedFence` if not all fences are signalled in before timeout.
/// Returns error if failed otherwise.
pub(crate) unsafe fn wait_for_all_fences<C, I>(fp: ash::vk::DeviceFnV1_0, device: ash::vk::Device, fences: I, timeout: u64) -> Result<Result<impl Iterator<Item = ReadyFence<C>>, impl Iterator<Item = ArmedFence<C>>>, DeviceLostOrOomError>
where
    I: IntoIterator<Item = ArmedFence<C>>,
{
    let fences = fences.into_iter().collect::<SmallVec<[_; 32]>>();
    let raws = fences.iter().map(|fence| fence.raw).collect::<SmallVec<[_; 32]>>();
    let result = fp.wait_for_fences(device, raws.len() as u32, raws.as_ptr(), 1, timeout);

    match result {
        ash::vk::Result::Success => {
            Ok(Ok(fences.into_iter().map(|fence| ReadyFence {
                raw: fence.raw,
                queue: fence.queue,
                epoch: fence.epoch,
                relevant: fence.relevant,
            })))
        },
        ash::vk::Result::Timeout => Ok(Err(fences.into_iter())),
        ash::vk::Result::ErrorOutOfHostMemory => Err(DeviceLostOrOomError::OomError(OomError::OutOfHostMemory)),
        ash::vk::Result::ErrorOutOfDeviceMemory => Err(DeviceLostOrOomError::OomError(OomError::OutOfDeviceMemory)),
        ash::vk::Result::ErrorDeviceLost => Err(DeviceLostOrOomError::DeviceLost(DeviceLost)),
        _ => unreachable!(),
    }
}

/// Check fence status.
/// Returns `ReadyFence` if fence is signalled.
/// Returns back `ArmedFence` if fence is not signalled.
/// Returns error if failed.
pub(crate) unsafe fn get_fence_status<C>(fp: ash::vk::DeviceFnV1_0, device: ash::vk::Device, fence: ArmedFence<C>) -> Result<Result<ReadyFence<C>, ArmedFence<C>>, DeviceLost> {
    match fp.get_fence_status(device, fence.raw) {
        ash::vk::Result::Success => {
            Ok(Ok(ReadyFence {
                raw: fence.raw,
                queue: fence.queue,
                epoch: fence.epoch,
                relevant: fence.relevant,
            }))
        },
        ash::vk::Result::NotReady => Ok(Err(fence)),
        ash::vk::Result::ErrorDeviceLost => Err(DeviceLost),
        _ => unreachable!(),
    }
}
