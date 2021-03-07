mod render_thread;
use render_thread::RenderThread;

mod swapchain_resources;
pub use swapchain_resources::SwapchainResources;

mod render_frame_job;
use render_frame_job::RenderFrameJob;

mod renderer_builder;
pub use renderer_builder::AssetSource;
pub use renderer_builder::RendererBuilder;
pub use renderer_builder::RendererBuilderResult;

//TODO: Find a way to not expose this
mod swapchain_handling;
pub use swapchain_handling::SwapchainHandler;

mod renderer;
pub use renderer::*;

pub mod daemon;
