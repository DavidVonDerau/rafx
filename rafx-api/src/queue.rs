#[cfg(feature = "rafx-metal")]
use crate::metal::RafxQueueMetal;
#[cfg(feature = "rafx-vulkan")]
use crate::vulkan::RafxQueueVulkan;
use crate::{
    RafxCommandBuffer, RafxCommandPool, RafxCommandPoolDef, RafxFence, RafxPresentSuccessResult,
    RafxQueueType, RafxResult, RafxSemaphore, RafxSwapchain,
};

/// A queue allows work to be submitted to the GPU
///
/// Work that has been submitted to the same queue has some ordering guarantees.
///
/// Resources may only be accessed from one queue type at a time. If a resource is to be used by
/// a different queue, a memory barrier on both the "sending" and "receiving" queue is required.
///
/// The default configuration is to return the same underlying queue every time create_queue() is
/// called. A mutex protects against multiple threads submitting work to the queue at the same time.
///
/// Most applications can just create and use graphics queues freely, relying on the API returning
/// the same underlying queue every time.
#[derive(Clone, Debug)]
pub enum RafxQueue {
    #[cfg(feature = "rafx-vulkan")]
    Vk(RafxQueueVulkan),
    #[cfg(feature = "rafx-metal")]
    Metal(RafxQueueMetal),
}

impl RafxQueue {
    /// Returns an opaque ID associated with this queue. It may be used to hash which queue a
    /// command pool is associated with
    pub fn queue_id(&self) -> u32 {
        match self {
            #[cfg(feature = "rafx-vulkan")]
            RafxQueue::Vk(inner) => inner.queue_id(),
            #[cfg(feature = "rafx-metal")]
            RafxQueue::Metal(_inner) => unimplemented!(),
        }
    }

    /// Get the type of queue that this is
    pub fn queue_type(&self) -> RafxQueueType {
        match self {
            #[cfg(feature = "rafx-vulkan")]
            RafxQueue::Vk(inner) => inner.queue_type(),
            #[cfg(feature = "rafx-metal")]
            RafxQueue::Metal(_inner) => unimplemented!(),
        }
    }

    /// Create a command pool for use with this queue
    pub fn create_command_pool(
        &self,
        command_pool_def: &RafxCommandPoolDef,
    ) -> RafxResult<RafxCommandPool> {
        Ok(match self {
            #[cfg(feature = "rafx-vulkan")]
            RafxQueue::Vk(inner) => {
                RafxCommandPool::Vk(inner.create_command_pool(command_pool_def)?)
            }
            #[cfg(feature = "rafx-metal")]
            RafxQueue::Metal(_inner) => unimplemented!(),
        })
    }

    /// Submit command buffers for processing by the GPU.
    ///
    /// Execution will not begin until all `wait_semaphores` are signaled.
    ///
    /// After execution, the given `signal_semaphores` and `signal_fence` are signaled as completed.
    pub fn submit(
        &self,
        command_buffers: &[&RafxCommandBuffer],
        wait_semaphores: &[&RafxSemaphore],
        signal_semaphores: &[&RafxSemaphore],
        signal_fence: Option<&RafxFence>,
    ) -> RafxResult<()> {
        match self {
            #[cfg(feature = "rafx-vulkan")]
            RafxQueue::Vk(inner) => {
                let command_buffers: Vec<_> = command_buffers
                    .iter()
                    .map(|x| x.vk_command_buffer().unwrap())
                    .collect();
                let wait_semaphores: Vec<_> = wait_semaphores
                    .iter()
                    .map(|x| x.vk_semaphore().unwrap())
                    .collect();
                let signal_semaphores: Vec<_> = signal_semaphores
                    .iter()
                    .map(|x| x.vk_semaphore().unwrap())
                    .collect();
                inner.submit(
                    &command_buffers,
                    &wait_semaphores,
                    &signal_semaphores,
                    signal_fence.map(|x| x.vk_fence().unwrap()),
                )
            }
            #[cfg(feature = "rafx-metal")]
            RafxQueue::Metal(_inner) => unimplemented!(),
        }
    }

    /// Presents an image in the swapchain.
    ///
    /// Execution will not begin until all `wait_semaphores` are signaled.
    pub fn present(
        &self,
        swapchain: &RafxSwapchain,
        wait_semaphores: &[&RafxSemaphore],
        image_index: u32,
    ) -> RafxResult<RafxPresentSuccessResult> {
        match self {
            #[cfg(feature = "rafx-vulkan")]
            RafxQueue::Vk(inner) => {
                let wait_semaphores: Vec<_> = wait_semaphores
                    .iter()
                    .map(|x| x.vk_semaphore().unwrap())
                    .collect();
                inner.present(
                    swapchain.vk_swapchain().unwrap(),
                    &wait_semaphores,
                    image_index,
                )
            }
            #[cfg(feature = "rafx-metal")]
            RafxQueue::Metal(_inner) => unimplemented!(),
        }
    }

    /// Wait until all work submitted to this queue is completed
    pub fn wait_for_queue_idle(&self) -> RafxResult<()> {
        match self {
            #[cfg(feature = "rafx-vulkan")]
            RafxQueue::Vk(inner) => inner.wait_for_queue_idle(),
            #[cfg(feature = "rafx-metal")]
            RafxQueue::Metal(_inner) => unimplemented!(),
        }
    }

    /// Get the underlying vulkan API object. This provides access to any internally created
    /// vulkan objects.
    #[cfg(feature = "rafx-vulkan")]
    pub fn vk_queue(&self) -> Option<&RafxQueueVulkan> {
        match self {
            #[cfg(feature = "rafx-vulkan")]
            RafxQueue::Vk(inner) => Some(inner),
            #[cfg(feature = "rafx-metal")]
            RafxQueue::Metal(_inner) => None,
        }
    }

    /// Get the underlying metal API object. This provides access to any internally created
    /// metal objects.
    #[cfg(feature = "rafx-metal")]
    pub fn metal_queue(&self) -> Option<&RafxQueueMetal> {
        match self {
            #[cfg(feature = "rafx-vulkan")]
            RafxQueue::Vk(_inner) => None,
            #[cfg(feature = "rafx-metal")]
            RafxQueue::Metal(inner) => Some(inner),
        }
    }
}
