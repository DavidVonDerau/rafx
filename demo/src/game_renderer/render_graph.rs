use ash::vk;
use renderer::assets::{vk_description as dsc, ResourceContext};
use renderer::assets::graph::*;
use renderer::assets::resources::ResourceManager;
use crate::VkDeviceContext;
use ash::prelude::VkResult;
use renderer::assets::resources::{ResourceArc, ImageViewResource, RenderPassResource};
use crate::render_contexts::{RenderJobWriteContextFactory, RenderJobWriteContext};
use renderer::nodes::{PreparedRenderData, RenderView};
use crate::phases::{OpaqueRenderPhase, UiRenderPhase};
use renderer::vulkan::SwapchainInfo;

pub struct BuildRenderGraphResult {
    pub opaque_renderpass: Option<ResourceArc<RenderPassResource>>,
    pub ui_renderpass: Option<ResourceArc<RenderPassResource>>,
    //pub bloom_extract_renderpass: Option<ResourceArc<RenderPassResource>>,
    pub executor: RenderGraphExecutor<RenderGraphExecuteContext>,
}

// Any data you want available within rendergraph execution callbacks should go here. This can
// include data that is not known until later after the extract/prepare phases have completed.
pub struct RenderGraphExecuteContext {
    pub prepared_render_data: Box<PreparedRenderData<RenderJobWriteContext>>,
    pub write_context_factory: RenderJobWriteContextFactory,
    //pub command_writer: DynCommandWriter, // command buffers
    //pub resource_context: ResourceContext
    //pub dyn_resource_allocators: DynResourceAllocatorSet, // images, image views, buffers
    //pub descriptor_set_alloctor_provider: DescriptorSetAllocatorProvider, // descriptor sets
}

impl RenderGraphExecuteContext {
    pub fn resource_context(&self) -> &ResourceContext {
        &self.write_context_factory.resource_context
    }
}

pub fn build_render_graph(
    device_context: &VkDeviceContext,
    resource_context: &ResourceContext,
    resource_manager: &mut ResourceManager,
    swapchain_surface_info: &dsc::SwapchainSurfaceInfo,
    swapchain_info: &SwapchainInfo,
    swapchain_image: ResourceArc<ImageViewResource>,
    main_view: RenderView,
) -> VkResult<BuildRenderGraphResult> {
    //TODO: Fix this back to be color format
    let color_format = swapchain_surface_info.surface_format.format;
    let depth_format = swapchain_surface_info.depth_format;
    let swapchain_format = swapchain_surface_info.surface_format.format;
    let samples = swapchain_surface_info.msaa_level.into();

    let mut graph = RenderGraphBuilder::default();
    let mut graph_callbacks = RenderGraphNodeCallbacks::<RenderGraphExecuteContext>::default();

    let opaque_pass = {
        struct Opaque {
            node: RenderGraphNodeId,
            color: RenderGraphImageUsageId,
            depth: RenderGraphImageUsageId,
        }

        let node = graph.add_node("Opaque", RenderGraphQueue::DefaultGraphics);

        let color = graph.create_color_attachment(
            node,
            0,
            Some(vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 0.0],
            }),
            RenderGraphImageConstraint {
                samples: Some(samples),
                format: Some(color_format),
                ..Default::default()
            },
        );
        graph.set_image_name(color, "color");

        let depth = graph.create_depth_attachment(
            node,
            Some(vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            }),
            RenderGraphImageConstraint {
                samples: Some(samples),
                format: Some(depth_format),
                ..Default::default()
            },
        );
        graph.set_image_name(depth, "depth");

        graph_callbacks.add_renderphase_dependency::<OpaqueRenderPhase>(node);

        let main_view = main_view.clone();
        graph_callbacks.set_renderpass_callback(
            node,
            move |command_buffer, _graph_context, user_context| {
                let mut write_context = user_context
                    .write_context_factory
                    .create_context(command_buffer);
                user_context
                    .prepared_render_data
                    .write_view_phase::<OpaqueRenderPhase>(&main_view, &mut write_context);
                Ok(())
            },
        );

        Opaque { node, color, depth }
    };

    let ui_pass = {
        struct Ui {
            node: RenderGraphNodeId,
            color: RenderGraphImageUsageId,
        }

        // This node has a single color attachment
        let node = graph.add_node("Ui", RenderGraphQueue::DefaultGraphics);
        let color = graph.modify_color_attachment(
            node,
            opaque_pass.color,
            0,
            RenderGraphImageConstraint {
                samples: Some(vk::SampleCountFlags::TYPE_1),
                ..Default::default()
            },
        );

        // Adding a phase dependency insures that we create all the pipelines for materials
        // associated with the phase. This controls how long we keep the pipelines allocated and
        // allows us to precache pipelines for materials as they are loaded
        graph_callbacks.add_renderphase_dependency::<UiRenderPhase>(node);

        // When the node is executed, we automatically set up the renderpass/framebuffer/command
        // buffer. Just add the draw calls.
        let main_view = main_view.clone();
        graph_callbacks.set_renderpass_callback(
            node,
            move |command_buffer, _graph_context, user_context| {
                // Can retrieve images created/used by the graph
                //let _color_attachment = graph_context.image(color).unwrap();

                // Kick the material system to emit all draw calls for the UiRenderPhase for the view
                let mut write_context = user_context
                    .write_context_factory
                    .create_context(command_buffer);
                user_context
                    .prepared_render_data
                    .write_view_phase::<UiRenderPhase>(&main_view, &mut write_context);
                Ok(())
            },
        );

        Ui { node, color }
    };
    /*
        let blur_extract_pass = {
            struct BlurExtractPass {
                node: RenderGraphNodeId,
                sdr_image: RenderGraphImageUsageId,
                hdr_image: RenderGraphImageUsageId,
            }

            let node = graph.add_node("BlurExtract", RenderGraphQueue::DefaultGraphics);

            //node.sample_image(ui_pass.node_id);
            let sdr_image =
                graph.create_color_attachment(node, 0, Default::default(), Default::default());
            let hdr_image = graph.create_color_attachment(
                node,
                1,
                Some(vk::ClearColorValue::default()),
                RenderGraphImageConstraint {
                    samples: Some(vk::SampleCountFlags::TYPE_1),
                    format: Some(color_format),
                    ..Default::default()
                },
            );

            graph.sample_image(node, ui_pass.color, Default::default());

            graph_callbacks.set_renderpass_callback(node, |command_buffer, graph_context. context| {
                //TODO:
                // - Resolve the pipeline from the cache using the blur_extract_pass material pass resource
                // - Get the image that corresponds with ui_pass.color
                // - Create a descriptor set for the pass using the image

                //context.

                // let descriptor_set_allocator_provider = context.descriptor_set_alloctor_provider.get_allocator();
                //
                // descriptor_set_allocator_provider.al
                //
                // //ui_pass.color


                // device_context.cmd_bind_pipeline(
                //     command_buffer,
                //     vk::PipelineBindPoint::GRAPHICS,
                //     self.pipeline_info.get_raw().pipelines[0],
                // );
                //
                // logical_device.cmd_bind_descriptor_sets(
                //     command_buffer,
                //     vk::PipelineBindPoint::GRAPHICS,
                //     self.pipeline_info
                //         .get_raw()
                //         .pipeline_layout
                //         .get_raw()
                //         .pipeline_layout,
                //     0,
                //     &[descriptor_set],
                //     &[],
                // );
                //
                // logical_device.cmd_draw(command_buffer, 3, 1, 0, 0);

                // bind?

                Ok(())
            });

            BlurExtractPass {
                node,
                sdr_image,
                hdr_image,
            }
        };
    */
    let _swapchain_output_image_id = graph.set_output_image(
        ui_pass.color,
        //blur_extract_pass.sdr_image,
        swapchain_image,
        RenderGraphImageSpecification {
            samples: vk::SampleCountFlags::TYPE_1,
            format: swapchain_format,
            aspect_flags: vk::ImageAspectFlags::COLOR,
            usage_flags: swapchain_info.image_usage_flags,
        },
        dsc::ImageLayout::PresentSrcKhr,
        vk::AccessFlags::empty(),
        vk::PipelineStageFlags::empty(),
        vk::ImageAspectFlags::COLOR,
    );

    //
    // Create the executor, it needs to have access to the resource manager to add framebuffers
    // and renderpasses to the resource lookups
    //
    let executor = RenderGraphExecutor::new(
        &device_context,
        &resource_context,
        resource_manager,
        graph,
        swapchain_surface_info,
        graph_callbacks,
    )?;

    let opaque_renderpass = executor.renderpass_resource(opaque_pass.node);
    let ui_renderpass = executor.renderpass_resource(ui_pass.node);
    //let bloom_extract_renderpass = executor.renderpass_resource(blur_extract_pass.node);
    Ok(BuildRenderGraphResult {
        executor,
        opaque_renderpass,
        ui_renderpass,
        //bloom_extract_renderpass: bloom_extract_renderpass
    })
}