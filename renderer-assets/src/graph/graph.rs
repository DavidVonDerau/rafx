use super::*;
use fnv::{FnvHashSet, FnvHashMap};
use crate::vk_description as dsc;
use crate::vk_description::AttachmentReference;
use std::collections::HashMap;

pub struct DetermineImageConstraintsResult {
    images: FnvHashMap<RenderGraphImageUsageId, RenderGraphImageSpecification>,
}

impl DetermineImageConstraintsResult {
    pub fn specification(
        &self,
        image: RenderGraphImageUsageId,
    ) -> &RenderGraphImageSpecification {
        self.images.get(&image).unwrap()
    }
}

pub struct RenderGraphImageVersionLayout {
    write_layout: dsc::ImageLayout,
    write_pass_end_layout: dsc::ImageLayout,
    read_layout: dsc::ImageLayout,
}

pub struct DetermineImageLayoutsResult {
    //image_layouts : FnvHashMap<RenderGraphImageUsageId, dsc::ImageLayout>
    image_layouts : FnvHashMap<RenderGraphImageVersionId, RenderGraphImageVersionLayout>
}

#[derive(Debug)]
pub struct AssignPhysicalImagesResult {
    map_image_to_physical: FnvHashMap<RenderGraphImageUsageId, PhysicalImageId>,
    physical_image_usages: FnvHashMap<PhysicalImageId, Vec<RenderGraphImageUsageId>>,
    physical_image_versions: FnvHashMap<PhysicalImageId, Vec<RenderGraphImageVersionId>>,
}
/*
struct PhysicalImageState {
    layout: dsc::ImageLayout,
    // access_flags: vk::AccessFlags,
    // stage_flags: vk::PipelineStageFlags,
    // image_aspect_flags: vk::ImageAspectFlags,
}

impl Default for PhysicalImageState {
    fn default() -> Self {
        PhysicalImageState {
            layout: dsc::ImageLayout::Undefined,
            //access_flags: vk::AccessFlags::empty(),

        }
    }
}
*/
// struct PassSyncFlags {
//     read_access_flags: vk::AccessFlags,
//     read_stage_flags: vk::PipelineStageFlags,
//     write_access_flags: vk::AccessFlags,
//     write_stage_flags: vk::PipelineStageFlags,
// }

#[derive(Debug)]
pub struct RenderGraphInputImage {
    pub usage: RenderGraphImageUsageId,
    pub specification: RenderGraphImageSpecification,
}

#[derive(Debug)]
pub struct RenderGraphOutputImage {
    pub usage: RenderGraphImageUsageId,
    pub specification: RenderGraphImageSpecification,

    pub(super) final_layout: dsc::ImageLayout,
    pub(super) final_access_flags: vk::AccessFlags,
    pub(super) final_stage_flags: vk::PipelineStageFlags,
}

#[derive(Debug)]
pub struct RenderGraphImageBarrier {
    //layout: vk::ImageLayout,
    access_flags: vk::AccessFlags,
    stage_flags: vk::PipelineStageFlags
}

impl Default for RenderGraphImageBarrier {
    fn default() -> Self {
        RenderGraphImageBarrier {
            //layout: vk::ImageLayout::UNDEFINED,
            access_flags: vk::AccessFlags::empty(),
            stage_flags: vk::PipelineStageFlags::empty()
        }
    }
}

#[derive(Debug)]
pub struct RenderGraphPassImageBarriers {
    invalidate: RenderGraphImageBarrier,
    flush: RenderGraphImageBarrier,
    layout: vk::ImageLayout,
}

impl RenderGraphPassImageBarriers {
    fn new(layout: vk::ImageLayout) -> Self {
        RenderGraphPassImageBarriers {
            flush: Default::default(),
            invalidate: Default::default(),
            layout
        }
    }
}

#[derive(Debug)]
pub struct RenderGraphNodeImageBarriers {
    //invalidates: FnvHashMap<PhysicalImageId, RenderGraphImageBarrier>,
    //flushes: FnvHashMap<PhysicalImageId, RenderGraphImageBarrier>,
    barriers: FnvHashMap<PhysicalImageId, RenderGraphPassImageBarriers>
}

#[derive(Debug)]
pub struct RenderGraphSubpass {
    node: RenderGraphNodeId,

    color_attachments: [Option<usize>; 4], // could ref back to node
    resolve_attachments: [Option<usize>; 4],
    depth_attachment: Option<usize>,
}

pub enum AttachmentClearValue {
    Color(vk::ClearColorValue),
    DepthStencil(vk::ClearDepthStencilValue)
}

impl std::fmt::Debug for AttachmentClearValue {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            AttachmentClearValue::Color(value) => {
                f.debug_struct("AttachmentClearValue(Color)")
                    .finish()
            },
            AttachmentClearValue::DepthStencil(value) => {
                f.debug_struct("AttachmentClearValue(DepthStencil)")
                    .field("depth", &value.depth)
                    .field("stencil", &value.stencil)
                    .finish()
            }
        }
    }
}

#[derive(Debug)]
pub struct RenderGraphPassAttachment {
    image: PhysicalImageId,
    load_op: vk::AttachmentLoadOp,
    stencil_load_op: vk::AttachmentLoadOp,
    store_op: vk::AttachmentStoreOp,
    stencil_store_op: vk::AttachmentStoreOp,
    clear_color: Option<AttachmentClearValue>,
    format: vk::Format,
    samples: vk::SampleCountFlags,
    initial_layout: dsc::ImageLayout,
    final_layout: dsc::ImageLayout
}

#[derive(Debug)]
pub struct RenderGraphPass {
    attachments: Vec<RenderGraphPassAttachment>,
    subpasses: Vec<RenderGraphSubpass>,
    // clear colors?
}

// struct ResourceState
// {
//     VkImageLayout initial_layout = VK_IMAGE_LAYOUT_UNDEFINED;
//     VkImageLayout final_layout = VK_IMAGE_LAYOUT_UNDEFINED;
//     VkAccessFlags invalidated_types = 0;
//     VkAccessFlags flushed_types = 0;
//
//     VkPipelineStageFlags invalidated_stages = 0;
//     VkPipelineStageFlags flushed_stages = 0;
// };

//
// A collection of nodes and resources. Nodes represent an event or process that will occur at
// a certain time. Resources represent images and buffers that may be read/written by nodes.
//
#[derive(Default, Debug)]
pub struct RenderGraph {
    // Nodes that have been registered in the graph
    nodes: Vec<RenderGraphNode>,

    // Image resources that have been registered in the graph. These resources are "virtual" until
    // the graph is scheduled. In other words, we don't necessarily allocate an image for every
    // resource as some resources can share the same image internally if their lifetime don't
    // overlap. Additionally, a resource can be bound to input and output images. If this is the
    // case, we will try to use those images rather than creating new ones.
    image_resources: Vec<RenderGraphImageResource>,

    image_usages: Vec<RenderGraphImageUsage>,

    pub(super) input_images: Vec<RenderGraphInputImage>,
    pub(super) output_images: Vec<RenderGraphOutputImage>,
}

impl RenderGraph {
    pub(super) fn add_image_usage(
        &mut self,
        version: RenderGraphImageVersionId,
        usage_type: RenderGraphImageUsageType,
        preferred_layout: dsc::ImageLayout,
        access_flags: vk::AccessFlags,
        stage_flags: vk::PipelineStageFlags,
        image_aspect_flags: vk::ImageAspectFlags,
    ) -> RenderGraphImageUsageId {
        let usage_id = RenderGraphImageUsageId(self.image_usages.len());
        self.image_usages.push(RenderGraphImageUsage {
            usage_type,
            version,
            preferred_layout,
            //access_flags,
            //stage_flags,
            //image_aspect_flags
        });
        usage_id
    }

    // Add an image that can be used by nodes
    pub(super) fn add_image_create(
        &mut self,
        create_node: RenderGraphNodeId,
        attachment_type: RenderGraphAttachmentType,
        constraint: RenderGraphImageConstraint,
        preferred_layout: dsc::ImageLayout,
        access_flags: vk::AccessFlags,
        stage_flags: vk::PipelineStageFlags,
        image_aspect_flags: vk::ImageAspectFlags,
    ) -> RenderGraphImageUsageId {
        let version_id = RenderGraphImageVersionId {
            index: self.image_resources.len(),
            version: 0,
        };
        let usage_id = self.add_image_usage(
            version_id,
            RenderGraphImageUsageType::Create,
            preferred_layout,
            access_flags,
            stage_flags,
            image_aspect_flags
        );

        let mut resource = RenderGraphImageResource::new();

        let mut version_info = RenderGraphImageResourceVersionInfo::new(create_node, usage_id);
        resource.versions.push(version_info);

        // Add it to the graph
        self.image_resources.push(resource);

        self.nodes[create_node.0]
            .image_creates
            .push(RenderGraphImageCreate {
                //image: image_id,
                image: usage_id,
                constraint,
                attachment_type,
            });

        usage_id
    }

    pub(super) fn add_image_read(
        &mut self,
        read_node: RenderGraphNodeId,
        image: RenderGraphImageUsageId,
        attachment_type: RenderGraphAttachmentType,
        constraint: RenderGraphImageConstraint,
        preferred_layout: dsc::ImageLayout,
        access_flags: vk::AccessFlags,
        stage_flags: vk::PipelineStageFlags,
        image_aspect_flags: vk::ImageAspectFlags,
    ) -> RenderGraphImageUsageId {
        let version_id = self.image_usages[image.0].version;

        let usage_id = self.add_image_usage(
            version_id,
            RenderGraphImageUsageType::Read,
            preferred_layout,
            access_flags,
            stage_flags,
            image_aspect_flags
        );

        self.image_resources[version_id.index].versions[version_id.version]
            .add_read_usage(usage_id);

        self.nodes[read_node.0]
            .image_reads
            .push(RenderGraphImageRead {
                image: usage_id,
                constraint,
                attachment_type,
            });

        usage_id
    }

    pub(super) fn add_image_modify(
        &mut self,
        modify_node: RenderGraphNodeId,
        image: RenderGraphImageUsageId,
        attachment_type: RenderGraphAttachmentType,
        constraint: RenderGraphImageConstraint,
        preferred_layout: dsc::ImageLayout,
        read_access_flags: vk::AccessFlags,
        read_stage_flags: vk::PipelineStageFlags,
        read_image_aspect_flags: vk::ImageAspectFlags,
        write_access_flags: vk::AccessFlags,
        write_stage_flags: vk::PipelineStageFlags,
        write_image_aspect_flags: vk::ImageAspectFlags,
    ) -> (RenderGraphImageUsageId, RenderGraphImageUsageId) {
        let read_version_id = self.image_usages[image.0].version;

        let read_usage_id = self.add_image_usage(
            read_version_id,
            RenderGraphImageUsageType::ModifyRead,
            preferred_layout,
            read_access_flags,
            read_stage_flags,
            read_image_aspect_flags
        );

        self.image_resources[read_version_id.index].versions[read_version_id.version]
            .add_read_usage(read_usage_id);

        // Create a new version and add it to the image
        let version = self.image_resources[read_version_id.index].versions.len();

        let write_version_id = RenderGraphImageVersionId {
            index: read_version_id.index,
            version,
        };
        let write_usage_id = self.add_image_usage(
            write_version_id,
            RenderGraphImageUsageType::ModifyWrite,
            preferred_layout,
            write_access_flags,
            write_stage_flags,
            write_image_aspect_flags
        );

        let mut version_info =
            RenderGraphImageResourceVersionInfo::new(modify_node, write_usage_id);
        self.image_resources[read_version_id.index]
            .versions
            .push(version_info);

        self.nodes[modify_node.0]
            .image_modifies
            .push(RenderGraphImageModify {
                input: read_usage_id,
                output: write_usage_id,
                constraint,
                attachment_type,
            });

        (read_usage_id, write_usage_id)
    }

    fn set_color_attachment(
        &mut self,
        node: RenderGraphNodeId,
        color_attachment_index: usize,
        color_attachment: RenderGraphPassColorAttachmentInfo,
    ) {
        //TODO: Check constraint does not conflict with the matching resolve attachment, if there is one
        let mut node_color_attachments = &mut self.nodes[node.0].color_attachments;
        if node_color_attachments.len() <= color_attachment_index {
            node_color_attachments.resize_with(color_attachment_index + 1, || None);
        }

        assert!(node_color_attachments[color_attachment_index].is_none());
        node_color_attachments[color_attachment_index] = Some(color_attachment);
    }

    fn set_depth_attachment(
        &mut self,
        node: RenderGraphNodeId,
        depth_attachment: RenderGraphPassDepthAttachmentInfo,
    ) {
        let mut node_depth_attachment = &mut self.nodes[node.0].depth_attachment;
        assert!(node_depth_attachment.is_none());
        *node_depth_attachment = Some(depth_attachment);
    }

    fn set_resolve_attachment(
        &mut self,
        node: RenderGraphNodeId,
        resolve_attachment_index: usize,
        resolve_attachment: RenderGraphPassResolveAttachmentInfo,
    ) {
        //TODO: Check constraint is non-MSAA and is not conflicting with the matching color attachment, if there is one
        let mut node_resolve_attachments = &mut self.nodes[node.0].resolve_attachments;
        if node_resolve_attachments.len() <= resolve_attachment_index {
            node_resolve_attachments.resize_with(resolve_attachment_index + 1, || None);
        }

        assert!(node_resolve_attachments[resolve_attachment_index].is_none());
        node_resolve_attachments[resolve_attachment_index] = Some(resolve_attachment);
    }

    pub fn create_color_attachment(
        &mut self,
        node: RenderGraphNodeId,
        color_attachment_index: usize,
        clear_color_value: Option<vk::ClearColorValue>,
        constraint: RenderGraphImageConstraint,
    ) -> RenderGraphImageUsageId {
        let attachment_type = RenderGraphAttachmentType::Color(color_attachment_index);

        // Add the read to the graph
        let create_image = self.add_image_create(
            node,
            attachment_type,
            constraint,
            dsc::ImageLayout::ColorAttachmentOptimal,
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::ImageAspectFlags::COLOR
        );

        self.set_color_attachment(
            node,
            color_attachment_index,
            RenderGraphPassColorAttachmentInfo {
                attachment_type: RenderGraphPassAttachmentType::Create,
                clear_color_value,
                read_image: None,
                write_image: Some(create_image),
            },
        );

        create_image
    }

    pub fn create_depth_attachment(
        &mut self,
        node: RenderGraphNodeId,
        clear_depth_stencil_value: Option<vk::ClearDepthStencilValue>,
        constraint: RenderGraphImageConstraint,
    ) -> RenderGraphImageUsageId {
        let attachment_type = RenderGraphAttachmentType::Depth;

        // Add the read to the graph
        let create_image = self.add_image_create(
            node,
            attachment_type,
            constraint,
            dsc::ImageLayout::DepthStencilAttachmentOptimal,
            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::ImageAspectFlags::DEPTH
        );

        self.set_depth_attachment(
            node,
            RenderGraphPassDepthAttachmentInfo {
                attachment_type: RenderGraphPassAttachmentType::Create,
                clear_depth_stencil_value,
                read_image: None,
                write_image: Some(create_image),
            },
        );

        create_image
    }

    pub fn create_resolve_attachment(
        &mut self,
        node: RenderGraphNodeId,
        resolve_attachment_index: usize,
        constraint: RenderGraphImageConstraint,
    ) -> RenderGraphImageUsageId {
        let attachment_type = RenderGraphAttachmentType::Resolve(resolve_attachment_index);

        let create_image = self.add_image_create(
            node,
            attachment_type,
            constraint,
            dsc::ImageLayout::ColorAttachmentOptimal,
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::ImageAspectFlags::COLOR
        );

        self.set_resolve_attachment(
            node,
            resolve_attachment_index,
            RenderGraphPassResolveAttachmentInfo {
                attachment_type: RenderGraphPassAttachmentType::Create,
                write_image: create_image,
            },
        );

        create_image
    }

    pub fn read_color_attachment(
        &mut self,
        node: RenderGraphNodeId,
        image: RenderGraphImageUsageId,
        color_attachment_index: usize,
        constraint: RenderGraphImageConstraint,
    ) {
        let attachment_type = RenderGraphAttachmentType::Color(color_attachment_index);

        // Add the read to the graph
        let read_image = self.add_image_read(
            node,
            image,
            attachment_type,
            constraint,
            dsc::ImageLayout::ColorAttachmentOptimal,
            vk::AccessFlags::COLOR_ATTACHMENT_READ,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::ImageAspectFlags::COLOR
        );

        self.set_color_attachment(
            node,
            color_attachment_index,
            RenderGraphPassColorAttachmentInfo {
                attachment_type: RenderGraphPassAttachmentType::Read,
                clear_color_value: None,
                read_image: Some(read_image),
                write_image: None,
            },
        );
    }

    pub fn read_depth_attachment(
        &mut self,
        node: RenderGraphNodeId,
        image: RenderGraphImageUsageId,
        constraint: RenderGraphImageConstraint,
    ) {
        let attachment_type = RenderGraphAttachmentType::Depth;

        // Add the read to the graph
        let read_image = self.add_image_read(
            node,
            image,
            attachment_type,
            constraint,
            dsc::ImageLayout::DepthStencilAttachmentOptimal,
            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
            vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
            vk::ImageAspectFlags::DEPTH
        );

        self.set_depth_attachment(
            node,
            RenderGraphPassDepthAttachmentInfo {
                attachment_type: RenderGraphPassAttachmentType::Read,
                clear_depth_stencil_value: None,
                read_image: Some(read_image),
                write_image: None,
            },
        );
    }

    pub fn modify_color_attachment(
        &mut self,
        node: RenderGraphNodeId,
        image: RenderGraphImageUsageId,
        color_attachment_index: usize,
        constraint: RenderGraphImageConstraint,
    ) -> RenderGraphImageUsageId {
        let attachment_type = RenderGraphAttachmentType::Color(color_attachment_index);

        // Add the read to the graph
        let (read_image, write_image) = self.add_image_modify(
            node,
            image,
            attachment_type,
            constraint,
            dsc::ImageLayout::ColorAttachmentOptimal,
            vk::AccessFlags::COLOR_ATTACHMENT_READ,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::ImageAspectFlags::COLOR,
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::ImageAspectFlags::COLOR
        );

        self.set_color_attachment(
            node,
            color_attachment_index,
            RenderGraphPassColorAttachmentInfo {
                attachment_type: RenderGraphPassAttachmentType::Modify,
                clear_color_value: None,
                read_image: Some(read_image),
                write_image: Some(write_image),
            },
        );

        write_image
    }

    pub fn modify_depth_attachment(
        &mut self,
        node: RenderGraphNodeId,
        image: RenderGraphImageUsageId,
        constraint: RenderGraphImageConstraint,
    ) -> RenderGraphImageUsageId {
        let attachment_type = RenderGraphAttachmentType::Depth;

        // Add the read to the graph
        let (read_image, write_image) = self.add_image_modify(
            node,
            image,
            attachment_type,
            constraint,
            dsc::ImageLayout::DepthStencilAttachmentOptimal,
            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
            vk::ImageAspectFlags::DEPTH,
            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
            vk::ImageAspectFlags::DEPTH,
        );

        self.set_depth_attachment(
            node,
            RenderGraphPassDepthAttachmentInfo {
                attachment_type: RenderGraphPassAttachmentType::Modify,
                clear_depth_stencil_value: None,
                read_image: Some(read_image),
                write_image: Some(write_image),
            },
        );

        read_image
    }

    pub fn configure_image(
        &mut self,
        image_id: RenderGraphImageUsageId,
    ) -> RenderGraphImageResourceConfigureContext {
        RenderGraphImageResourceConfigureContext {
            graph: self,
            image_id,
        }
    }

    // Add a node which can use resources
    pub fn add_node(&mut self) -> RenderGraphNodeConfigureContext {
        let node_id = RenderGraphNodeId(self.nodes.len());
        self.nodes.push(RenderGraphNode::new(node_id));
        self.configure_node(node_id)
    }

    pub fn configure_node(
        &mut self,
        node_id: RenderGraphNodeId,
    ) -> RenderGraphNodeConfigureContext {
        RenderGraphNodeConfigureContext {
            graph: self,
            node_id,
        }
    }

    //
    // Get nodes
    //
    pub(super) fn node(
        &self,
        node_id: RenderGraphNodeId,
    ) -> &RenderGraphNode {
        &self.nodes[node_id.0]
    }

    pub(super) fn node_mut(
        &mut self,
        node_id: RenderGraphNodeId,
    ) -> &mut RenderGraphNode {
        &mut self.nodes[node_id.0]
    }

    //
    // Get images
    //
    pub(super) fn image_resource(
        &self,
        usage_id: RenderGraphImageUsageId,
    ) -> &RenderGraphImageResource {
        let version = self.image_usages[usage_id.0].version;
        &self.image_resources[version.index]
    }

    pub(super) fn image_resource_mut(
        &mut self,
        usage_id: RenderGraphImageUsageId,
    ) -> &mut RenderGraphImageResource {
        let version = self.image_usages[usage_id.0].version;
        &mut self.image_resources[version.index]
    }

    //
    // Get image version infos
    //
    pub(super) fn image_version_info(
        &self,
        usage_id: RenderGraphImageUsageId,
    ) -> &RenderGraphImageResourceVersionInfo {
        let version = self.image_usages[usage_id.0].version;
        &self.image_resources[version.index].versions[version.version]
    }

    pub(super) fn image_version_info_mut(
        &mut self,
        usage_id: RenderGraphImageUsageId,
    ) -> &mut RenderGraphImageResourceVersionInfo {
        let version = self.image_usages[usage_id.0].version;
        &mut self.image_resources[version.index].versions[version.version]
    }

    pub(super) fn image_version_id(
        &self,
        usage_id: RenderGraphImageUsageId,
    ) -> RenderGraphImageVersionId {
        self.image_usages[usage_id.0].version
    }

    // https://en.wikipedia.org/wiki/Topological_sorting#Depth-first_search
    // We
    fn visit_node(
        &self,
        node_id: RenderGraphNodeId,
        visited: &mut Vec<bool>,
        visiting: &mut Vec<bool>,
        visiting_stack: &mut Vec<RenderGraphNodeId>,
        ordered_list: &mut Vec<RenderGraphNodeId>,
    ) {
        // This node is already visited and inserted into ordered_list
        if visited[node_id.0] {
            return;
        }

        // This node is already being visited higher up in the stack. This indicates a cycle in the
        // graph
        if visiting[node_id.0] {
            println!("Found cycle in graph");
            println!("{:?}", self.node(node_id));
            for v in visiting_stack.iter().rev() {
                println!("{:?}", self.node(*v));
            }
            panic!("Graph has a cycle");
        }

        // When we enter the node, mark the node as being in-progress of being visited to help
        // detect cycles in the graph
        visiting[node_id.0] = true;
        visiting_stack.push(node_id);

        //
        // Visit children
        //
        //println!("  Begin visit {:?}", node_id);
        let node = self.node(node_id);

        //TODO: Does the order of visiting here matter? If we consider merging subpasses, trying to
        // visit a node we want to merge with last will show that there are no indirect dependencies
        // between the merge candidate and the node we are visiting now. However, if there are
        // indirect dependencies, visiting a different node might mark the merge candidate as
        // already visited. So while we could try to visit the merge candidate last, it won't
        // guarantee that the merge candidate will actually be inserted later in the ordered_list
        // than any other dependency
        //
        // Also if we are trying to merge with a dependency (which will execute before us), then any
        // other requirements we have will need to be fulfilled before that node starts
        //
        // Other priorities might be to front-load anything compute-light/GPU-heavy. We can do this
        // by some sort of flagging/priority system to influence this logic. An end-user could
        // also use arbitrary dependencies the do nothing but influence the ordering here.
        //
        // As a first pass implementation, just ensure any merge dependencies are visited last so
        // that we will be more likely to be able to merge passes

        //TODO: Could we recurse to generate a bitset that is returned indicating all dependencies
        // and then O(n^2) try to dequeue them all in priority order?

        //
        // Delay visiting these nodes so that we get the best chance possible of mergeable passes
        // being adjacent to each other in the orderered list
        //
        let mut merge_candidates = FnvHashSet::default();
        if let Some(depth_attachment) = &node.depth_attachment {
            if let Some(read_image) = depth_attachment.read_image {
                let upstream_node = self.image_version_info(read_image).creator_node;

                // This might be too expensive to check
                if self.can_passes_merge(upstream_node, node.id()) {
                    merge_candidates.insert(upstream_node);
                }
            }
        }

        for color_attachment in &node.color_attachments {
            // If this is an attachment we are reading, then the node that created it is a merge candidate
            if let Some(read_image) = color_attachment.as_ref().and_then(|x| x.read_image) {
                let upstream_node = self.image_version_info(read_image).creator_node;

                // This might be too expensive to check
                if self.can_passes_merge(upstream_node, node.id()) {
                    merge_candidates.insert(upstream_node);
                }
            }
        }

        //
        // Visit all the nodes we aren't delaying
        //
        for read in &node.image_reads {
            let upstream_node = self.image_version_info(read.image).creator_node;
            if !merge_candidates.contains(&upstream_node) {
                self.visit_node(
                    upstream_node,
                    visited,
                    visiting,
                    visiting_stack,
                    ordered_list,
                );
            }
        }

        for modify in &node.image_modifies {
            let upstream_node = self.image_version_info(modify.input).creator_node;
            if !merge_candidates.contains(&upstream_node) {
                self.visit_node(
                    upstream_node,
                    visited,
                    visiting,
                    visiting_stack,
                    ordered_list,
                );
            }
        }

        //
        // Now visit the nodes we delayed visiting
        //
        for merge_candidate in merge_candidates {
            self.visit_node(
                merge_candidate,
                visited,
                visiting,
                visiting_stack,
                ordered_list,
            );
        }

        // All our pre-requisites were visited, so it's now safe to push this node onto the
        // orderered list
        ordered_list.push(node_id);
        visited[node_id.0] = true;

        // We are no longer visiting this node
        //println!("  End visit {:?}", node_id);
        visiting_stack.pop();
        visiting[node_id.0] = false;
    }

    fn determine_node_order(&self) -> Vec<RenderGraphNodeId> {
        // As we depth-first traverse nodes, mark them as visiting and push them onto this stack.
        // We will use this to detect and print out cycles
        let mut visiting = vec![false; self.nodes.len()];
        let mut visiting_stack = Vec::default();

        // The order of nodes, upstream to downstream. As we depth-first traverse nodes, push nodes
        // with no unvisited dependencies onto this list and mark them as visited
        let mut visited = vec![false; self.nodes.len()];
        let mut ordered_list = Vec::default();

        // Iterate all the images we need to output. This will visit all the nodes we to execute,
        // potentially leaving out nodes we can cull.
        for output_image_id in &self.output_images {
            // Find the node that creates the output image
            let output_node = self.image_version_info(output_image_id.usage).creator_node;
            println!(
                "Traversing dependencies of output image created by node {:?} {:?}",
                output_node,
                self.node(output_node).name()
            );

            self.visit_node(
                output_node,
                &mut visited,
                &mut visiting,
                &mut visiting_stack,
                &mut ordered_list,
            );
        }

        ordered_list
    }

    fn merge_passes(
        &mut self,
        node_execution_order: &[RenderGraphNodeId],
    ) {
        for i in 0..node_execution_order.len() - 1 {
            let prev = RenderGraphNodeId(i);
            let next = RenderGraphNodeId(i + 1);

            if self.can_passes_merge(prev, next) {
                // merge - mainly push next's ID onto previous's merge list
            }
        }
    }

    fn can_passes_merge(
        &self,
        prev: RenderGraphNodeId,
        next: RenderGraphNodeId,
    ) -> bool {
        // Reasons to reject merging:
        // - Queues match and are not compute based
        // - Global flag to disable merging
        // - Don't need to mipmap previous outputs
        // - Next doesn't need to sample from any previous output (image or buffer)
        // - Using different depth attachment? Not sure why

        // Reasons to allow merging:
        // - They share any color or depth attachments

        false
    }

    /*
    fn determine_images_in_use(
        &self,
        node_execution_order: &[RenderGraphNodeId],
    ) -> FnvHashSet<RenderGraphImageResourceId> {
        let mut image_versions_in_use = FnvHashSet::default();
        //let mut images_in_use = vec![false; self.image_resources.len()];
        for node in node_execution_order {
            let node = self.node(*node);
            for create in &node.image_creates {
                image_versions_in_use.insert(create.image);
                //images_in_use[create.image.index] = true;
            }

            for read in &node.image_reads {
                image_versions_in_use.insert(read.image);
                //images_in_use[read.image.index] = true;
            }

            for modify in &node.image_modifies {
                image_versions_in_use.insert(modify.input);
                //images_in_use[modify.input.index] = true;

                image_versions_in_use.insert(modify.output);
                //images_in_use[modify.output.index] = true;
            }
        }

        //TODO: Should I add output images too?
        //TODO: This is per usage

        // println!("Images in use:");
        // for (image_index, _) in images_in_use.iter().enumerate() {
        //     println!("  Image {:?}", self.image_resources[image_index]);
        // }

        image_versions_in_use
    }
    */

    fn get_create_usage(
        &self,
        usage: RenderGraphImageUsageId,
    ) -> RenderGraphImageUsageId {
        let version = self.image_usages[usage.0].version;
        self.image_resources[version.index].versions[version.version].create_usage
    }

    fn determine_image_constraints(
        &self,
        node_execution_order: &[RenderGraphNodeId],
    ) -> DetermineImageConstraintsResult {
        let mut image_version_states: FnvHashMap<
            RenderGraphImageUsageId,
            RenderGraphImageConstraint,
        > = Default::default();

        println!("Propagating image constraints");

        println!("  Set up input images");

        //
        // Propagate input image state specifications into images. Inputs are fully specified and
        // their constraints will never be overwritten
        //
        for input_image in &self.input_images {
            println!(
                "    Image {:?} {:?}",
                input_image,
                self.image_resource(input_image.usage).name
            );
            image_version_states
                .entry(self.get_create_usage(input_image.usage))
                .or_default()
                .set(&input_image.specification);

            // Don't bother setting usage constraint for 0
        }

        println!("  Propagate image constraints FORWARD");

        //
        // Iterate forward through nodes to determine what states images need to be in. We only need
        // to handle operations that produce a new version of a resource. These operations do not
        // need to fully specify image info, but whatever they do specify will be carried forward
        // and not overwritten
        //
        for node_id in node_execution_order.iter() {
            let node = self.node(*node_id);
            println!("    node {:?} {:?}", node_id, node.name());

            //
            // Propagate constraints into images this node creates.
            //
            for image_create in &node.image_creates {
                let image = self.image_version_info(image_create.image);
                // An image cannot be created within the graph and imported externally at the same
                // time. (The code here assumes no input and will not produce correct results if there
                // is an input image)
                //TODO: Input images are broken, we don't properly represent an image being created
                // OR receiving an input. We probably need to make creator in
                // RenderGraphImageResourceVersionInfo Option or an enum with input/create options
                //assert!(image.input_image.is_none());

                println!(
                    "      Create image {:?} {:?}",
                    image_create.image,
                    self.image_resource(image_create.image).name
                );

                let mut version_state = image_version_states
                    .entry(self.get_create_usage(image_create.image))
                    .or_default();

                if !version_state.try_merge(&image_create.constraint) {
                    // Should not happen as this should be our first visit to this image
                    panic!("Unexpected constraints on image being created");
                }

                println!(
                    "        Forward propagate constraints {:?} {:?}",
                    image_create.image, version_state
                );

                // Don't bother setting usage constraint for 0
            }

            // We don't need to propagate anything forward on reads

            //
            // Propagate constraints forward for images being modified.
            //
            for image_modify in &node.image_modifies {
                println!(
                    "      Modify image {:?} {:?} -> {:?} {:?}",
                    image_modify.input,
                    self.image_resource(image_modify.input).name,
                    image_modify.output,
                    self.image_resource(image_modify.output).name
                );

                let image = self.image_version_info(image_modify.input);
                //println!("  Modify image {:?} {:?}", image_modify.input, self.image_resource(image_modify.input).name);
                let input_state = image_version_states
                    .entry(self.get_create_usage(image_modify.input))
                    .or_default();
                let mut image_modify_constraint = image_modify.constraint.clone();

                // Merge the input image constraints with this node's constraints
                if !image_modify_constraint
                    .partial_merge(&input_state /*.combined_constraints*/)
                {
                    // This would need to be resolved by inserting some sort of fixup

                    // We will detect this on the backward pass, no need to do anything here
                    /*
                    let required_fixup = ImageConstraintRequiredFixup::Modify(node.id(), image_modify.clone());
                    println!("        *** Found required fixup: {:?}", required_fixup);
                    println!("            {:?}", input_state.constraint);
                    println!("            {:?}", image_modify_constraint);
                    required_fixups.push(required_fixup);
                    */
                    //println!("Image cannot be placed into a form that satisfies all constraints:\n{:#?}\n{:#?}", input_state.combined_constraints, image_modify.constraint);
                }

                //TODO: Should we set the usage constraint here? For now will wait until backward propagation

                let mut output_state = image_version_states
                    .entry(self.get_create_usage(image_modify.output))
                    .or_default();

                // Now propagate forward to the image version we write
                if !output_state
                    //.combined_constraints
                    .partial_merge(&image_modify_constraint)
                {
                    // // This should only happen if modifying an input image
                    // assert!(image.input_image.is_some());
                    // This would need to be resolved by inserting some sort of fixup

                    // We will detect this on the backward pass, no need to do anything here
                    /*
                    let required_fixup = ImageConstraintRequiredFixup::Modify(node.id(), image_modify.clone());
                    println!("        *** Found required fixup {:?}", required_fixup);
                    println!("            {:?}", image_modify_constraint);
                    println!("            {:?}", output_state.constraint);
                    required_fixups.push(required_fixup);
                    */
                    //println!("Image cannot be placed into a form that satisfies all constraints:\n{:#?}\n{:#?}", output_state.constraint, input_state.constraint);
                }

                println!("        Forward propagate constraints {:?}", output_state);
            }
        }

        println!("  Set up output images");

        //
        // Propagate output image state specifications into images
        //
        for output_image in &self.output_images {
            println!(
                "    Image {:?} {:?}",
                output_image,
                self.image_resource(output_image.usage).name
            );
            let mut output_image_version_state = image_version_states
                .entry(self.get_create_usage(output_image.usage))
                .or_default();
            let output_constraint = output_image.specification.clone().into();
            if !output_image_version_state.partial_merge(&output_constraint) {
                // This would need to be resolved by inserting some sort of fixup
                println!("      *** Found required OUTPUT fixup");
                println!(
                    "          {:?}",
                    output_image_version_state //.combined_constraints
                );
                println!("          {:?}", output_image.specification);
                //println!("Image cannot be placed into a form that satisfies all constraints:\n{:#?}\n{:#?}", output_image_version_state.constraint, output_specification);
            }

            image_version_states.insert(
                output_image.usage,
                output_image.specification.clone().into(),
            );
        }

        println!("  Propagate image constraints BACKWARD");

        //
        // Iterate backwards through nodes, determining the state the image must be in at every
        // step
        //
        for node_id in node_execution_order.iter().rev() {
            let node = self.node(*node_id);
            println!("    node {:?} {:?}", node_id, node.name());

            // Don't need to worry about creates, we back propagate to them when reading/modifying
            // // Propagate backwards into creates (in case they weren't fully specified)
            // for image_create in &node.image_creates {
            //     let image = self.image_version_info(image_create.image);
            //     println!("  Create image {:?} {:?}", image_create.image, self.image_resource(image_create.image).name);
            //     let mut version_state = &mut image_version_states[image_create.image.index][image_create.image.version];
            //     if !version_state.constraint.partial_merge(&image_create.constraint) {
            //         // Note this to handle later?
            //         panic!("Image cannot be placed into a form that satisfies all constraints:\n{:#?}\n{:#?}", version_state.constraint, image_create.constraint);
            //     }
            // }

            //
            // Propagate backwards from reads
            //
            for image_read in &node.image_reads {
                println!(
                    "      Read image {:?} {:?}",
                    image_read.image,
                    self.image_resource(image_read.image).name
                );

                let version_state = image_version_states
                    .entry(self.get_create_usage(image_read.image))
                    .or_default();
                if !version_state
                    //.combined_constraints
                    .partial_merge(&image_read.constraint)
                {
                    // This would need to be resolved by inserting some sort of fixup
                    println!("        *** Found required READ fixup");
                    println!(
                        "            {:?}",
                        version_state /*.combined_constraints*/
                    );
                    println!("            {:?}", image_read.constraint);
                    //println!("Image cannot be placed into a form that satisfies all constraints:\n{:#?}\n{:#?}", version_state.constraint, image_read.constraint);
                }

                // If this is an image read with no output, it's possible the constraint on the read is incomplete.
                // So we need to merge the image state that may have information forward-propagated
                // into it with the constraints on the read. (Conceptually it's like we're forward
                // propagating here because the main forward propagate pass does not handle reads.
                // TODO: We could consider moving this to the forward pass
                let mut image_read_constraint = image_read.constraint.clone();
                image_read_constraint.partial_merge(&version_state /*.combined_constraints*/);
                println!(
                    "        Read constraints will be {:?}",
                    image_read_constraint
                );
                if let Some(spec) = image_read_constraint.try_convert_to_specification() {
                    image_version_states.insert(image_read.image, spec.into());
                } else {
                    panic!(
                        "Not enough information in the graph to determine the specification for image {:?} {:?} being read by node {:?} {:?}. Constraints are: {:?}",
                        image_read.image,
                        self.image_resource(image_read.image).name,
                        node.id(),
                        node.name(),
                        image_version_states.get(&image_read.image)
                    );
                }
            }

            //
            // Propagate backwards from modifies
            //
            for image_modify in &node.image_modifies {
                println!(
                    "      Modify image {:?} {:?} <- {:?} {:?}",
                    image_modify.input,
                    self.image_resource(image_modify.input).name,
                    image_modify.output,
                    self.image_resource(image_modify.output).name
                );
                // The output image constraint already takes image_modify.constraint into account from
                // when we propagated image constraints forward
                let output_image_constraint = image_version_states
                    .entry(self.get_create_usage(image_modify.output))
                    .or_default()
                    .clone();
                let mut input_state = image_version_states
                    .entry(self.get_create_usage(image_modify.input))
                    .or_default();
                if !input_state.partial_merge(&output_image_constraint) {
                    // This would need to be resolved by inserting some sort of fixup
                    println!("        *** Found required MODIFY fixup");
                    println!(
                        "            {:?}",
                        input_state /*.combined_constraints*/
                    );
                    println!("            {:?}", image_modify.constraint);
                    //println!("Image cannot be placed into a form that satisfies all constraints:\n{:#?}\n{:#?}", input_state.constraint, image_modify.constraint);
                }

                image_version_states.insert(image_modify.input, output_image_constraint.clone());
            }
        }

        let mut image_specs = FnvHashMap::default();

        for (k, v) in image_version_states {
            image_specs.insert(k, v.try_convert_to_specification().unwrap());
        }

        DetermineImageConstraintsResult {
            images: image_specs,
        }
    }

    fn insert_resolves(
        &mut self,
        node_execution_order: &[RenderGraphNodeId],
        image_constraint_results: &mut DetermineImageConstraintsResult,
    ) {
        println!("Insert resolves in graph where necessary");
        for node_id in node_execution_order {
            let mut resolves_to_add = Vec::default();

            let node = self.node(*node_id);
            println!("  node {:?}", node_id);
            // Iterate through all color attachments
            for (color_attachment_index, color_attachment) in
                node.color_attachments.iter().enumerate()
            {
                if let Some(color_attachment) = color_attachment {
                    println!("    color attachment {}", color_attachment_index);
                    // If this color attachment outputs an image
                    if let Some(write_image) = color_attachment.write_image {
                        let write_version = self.image_usages[write_image.0].version;
                        // Skip if it's not an MSAA image
                        let write_spec = image_constraint_results.specification(write_image);
                        if write_spec.samples == vk::SampleCountFlags::TYPE_1 {
                            println!("      already non-MSAA");
                            continue;
                        }

                        // Calculate the spec that we would have after the resolve
                        let mut resolve_spec = write_spec.clone();
                        resolve_spec.samples = vk::SampleCountFlags::TYPE_1;

                        let mut usages_to_move = vec![];

                        // Look for any usages we need to fix
                        for (usage_index, read_usage) in self
                            .image_version_info(write_image)
                            .read_usages
                            .iter()
                            .enumerate()
                        {
                            println!(
                                "      usage {}, {:?}",
                                usage_index, self.image_usages[read_usage.0].usage_type
                            );
                            let read_spec = image_constraint_results.specification(*read_usage);
                            if *read_spec == *write_spec {
                                continue;
                            } else if *read_spec == resolve_spec {
                                usages_to_move.push(*read_usage);
                                break;
                            } else {
                                println!("        incompatibility cannot be fixed via renderpass resolve");
                                println!("{:?}", resolve_spec);
                                println!("{:?}", read_spec);
                            }
                        }

                        if !usages_to_move.is_empty() {
                            resolves_to_add.push((
                                color_attachment_index,
                                resolve_spec,
                                usages_to_move,
                            ));
                        }
                    }
                }
            }

            for (resolve_attachment_index, resolve_spec, usages_to_move) in resolves_to_add {
                println!(
                    "        ADDING RESOLVE FOR NODE {:?} ATTACHMENT {}",
                    node_id, resolve_attachment_index
                );
                let image = self.create_resolve_attachment(
                    *node_id,
                    resolve_attachment_index,
                    resolve_spec.clone().into(),
                );
                image_constraint_results
                    .images
                    .insert(image, resolve_spec.into());

                for usage in usages_to_move {
                    let from = self.image_usages[usage.0].version;
                    let to = self.image_usages[image.0].version;
                    println!(
                        "          MOVE USAGE {:?} from {:?} to {:?}",
                        usage, from, to
                    );
                    self.move_read_usage_to_image(usage, from, to)
                }
            }
        }
    }

    fn move_read_usage_to_image(
        &mut self,
        usage: RenderGraphImageUsageId,
        from: RenderGraphImageVersionId,
        to: RenderGraphImageVersionId,
    ) {
        self.image_resources[from.index].versions[from.version].remove_read_usage(usage);
        self.image_resources[to.index].versions[to.version].add_read_usage(usage);
    }

    fn assign_physical_images(
        &mut self,
        node_execution_order: &[RenderGraphNodeId],
        image_constraint_results: &mut DetermineImageConstraintsResult,
    ) -> AssignPhysicalImagesResult {
        let mut map_image_to_physical: FnvHashMap<RenderGraphImageUsageId, PhysicalImageId> =
            FnvHashMap::default();
        let mut physical_image_usages: FnvHashMap<PhysicalImageId, Vec<RenderGraphImageUsageId>> =
            FnvHashMap::default();
        let mut physical_image_versions: FnvHashMap<
            PhysicalImageId,
            Vec<RenderGraphImageVersionId>,
        > = FnvHashMap::default();

        let mut image_allocator = PhysicalImageAllocator::default();
        //TODO: Associate input images here? We can wait until we decide which images are shared
        println!("Associate images written by nodes with physical images");
        for node in node_execution_order.iter() {
            let node = self.node(*node);
            println!("  node {:?} {:?}", node.id().0, node.name());

            // A list of all images we write to from this node. We will try to share the images
            // being written forward into the nodes of downstream reads. This can chain such that
            // the same image is shared by many nodes
            let mut written_images = vec![];

            for create in &node.image_creates {
                // An image that's created always allocates an image (we can try to alias/pool these later)
                let physical_image =
                    image_allocator.allocate(&image_constraint_results.specification(create.image));
                println!(
                    "    Create {:?} will use image {:?}",
                    create.image, physical_image
                );
                map_image_to_physical.insert(create.image, physical_image);
                physical_image_usages
                    .entry(physical_image)
                    .or_default()
                    .push(create.image);
                physical_image_versions
                    .entry(physical_image)
                    .or_default()
                    .push(self.image_usages[create.image.0].version);

                // Queue this image write to try to share the image forward
                written_images.push(create.image);
            }

            for modify in &node.image_modifies {
                // The physical image in the read portion of a modify must also be the write image.
                // The format of the input/output is guaranteed to match
                assert_eq!(
                    image_constraint_results.specification(modify.input),
                    image_constraint_results.specification(modify.output)
                );

                // Assign the image
                let physical_image = map_image_to_physical.get(&modify.input).unwrap().clone();
                println!(
                    "    Modify {:?} will pass through image {:?}",
                    modify.output, physical_image
                );
                map_image_to_physical.insert(modify.output, physical_image);
                physical_image_usages
                    .entry(physical_image)
                    .or_default()
                    .push(modify.output);
                physical_image_versions
                    .entry(physical_image)
                    .or_default()
                    .push(self.image_usages[modify.output.0].version);

                // Queue this image write to try to share the image forward
                written_images.push(modify.output);
            }

            for written_image in written_images {
                // Count the downstream users of this image based on if they need read-only access
                // or write access. We need this information to determine which usages we can share
                // the output data with.
                //TODO: This could be smarter to handle the case of a resource being read/written
                // in different lifetimes
                let written_image_version_info = self.image_version_info(written_image);
                let mut read_count = 0;
                let mut write_count = 0;
                for usage in &written_image_version_info.read_usages {
                    if self.image_usages[usage.0].usage_type.is_read_only() {
                        read_count += 1;
                    } else {
                        write_count += 1;
                    }
                }

                // If we don't already have an image
                // let written_physical_image = mapping.entry(written_image)
                //     .or_insert_with(|| image_allocator.allocate(&image_constraint_results.specification(written_image)));

                let write_physical_image = *map_image_to_physical.get(&written_image).unwrap();
                let write_type = self.image_usages[written_image.0].usage_type;

                for usage_resource_id in &written_image_version_info.read_usages {
                    // We can't share images if they aren't the same format
                    let specifications_match = image_constraint_results
                        .specification(written_image)
                        == image_constraint_results.specification(*usage_resource_id);

                    // We can't share images unless it's a read or it's an exclusive write
                    let is_read_or_exclusive_write = (read_count > 0
                        && self.image_usages[usage_resource_id.0]
                            .usage_type
                            .is_read_only())
                        || write_count <= 1;

                    let read_type = self.image_usages[usage_resource_id.0].usage_type;
                    if specifications_match && is_read_or_exclusive_write {
                        // it's a shared read or an exclusive write
                        println!(
                            "    Usage {:?} will share an image with {:?} ({:?} -> {:?})",
                            written_image, usage_resource_id, write_type, read_type
                        );
                        let overwritten_image =
                            map_image_to_physical.insert(*usage_resource_id, write_physical_image);
                        physical_image_usages
                            .get_mut(&write_physical_image)
                            .unwrap()
                            //.or_default()
                            .push(*usage_resource_id);
                        assert!(overwritten_image.is_none());
                    } else {
                        // allocate new image
                        let specification = image_constraint_results.specification(written_image);
                        let physical_image = image_allocator.allocate(&specification);
                        println!(
                            "    Allocate image {:?} for {:?} ({:?} -> {:?})",
                            physical_image, usage_resource_id, write_type, read_type
                        );
                        let overwritten_image =
                            map_image_to_physical.insert(*usage_resource_id, physical_image);
                        physical_image_usages
                            .get_mut(&physical_image)
                            .unwrap()
                            //.or_default()
                            .push(*usage_resource_id);
                        assert!(overwritten_image.is_none());
                    }
                }
            }
        }

        // vulkan image layouts: https://github.com/nannou-org/nannou/issues/271#issuecomment-465876622
        AssignPhysicalImagesResult {
            physical_image_usages,
            map_image_to_physical,
            physical_image_versions,
        }
    }

    fn can_merge_nodes(
        &self,
        before_node_id: RenderGraphNodeId,
        after_node_id: RenderGraphNodeId,
        image_constraints: &DetermineImageConstraintsResult,
        physical_images: &AssignPhysicalImagesResult,
    ) -> bool {
        let before_node = self.node(before_node_id);
        let after_node = self.node(after_node_id);

        //TODO: Reject if not on the same queue, and not both graphics nodes

        //TODO: Reject if after reads something that before writes

        //TODO: Check if depth attachments are not the same?
        // https://developer.arm.com/documentation/101897/0200/fragment-shading/multipass-rendering
        // implies that the depth buffer must not change but this could be mali specific

        //TODO: Verify that some color or depth attachment gets used between the passes to justify
        // merging them. Unclear if this is necessarily desirable but likely is

        // For now don't merge anything
        false
    }

    // fn determine_image_layouts(
    //     &self,
    //     node_execution_order: &[RenderGraphNodeId],
    //     image_constraints: &DetermineImageConstraintsResult,
    //     physical_images: &AssignPhysicalImagesResult,
    // ) -> DetermineImageLayoutsResult {
    //     let mut image_layouts : FnvHashMap<RenderGraphImageVersionId, RenderGraphImageVersionLayout> = Default::default();
    //
    //     for (physical_image, versions) in &physical_images.physical_image_versions {
    //         for version in versions {
    //             let mut version_layout = RenderGraphImageVersionLayout {
    //                 write_layout: dsc::ImageLayout::Undefined,
    //                 write_pass_end_layout: dsc::ImageLayout::Undefined,
    //                 read_layout: dsc::ImageLayout::Undefined
    //             };
    //
    //             let version_info = &self.image_resources[version.index].versions[version.version];
    //             let write_usage_info = &self.image_usages[version_info.create_usage.0];
    //             version_layout.write_layout = write_usage_info.preferred_layout;
    //             println!("Layout Assignment Image: {:?} Version: {:?} Write: {:?}", physical_image, version, version_layout.write_layout);
    //
    //             // If we only have one reader, defer the layout transition to occur before the read
    //             if version_info.read_usages.len() <= 1 {
    //                 if let Some(read_usage) = version_info.read_usages.first() {
    //                     let preferred_read_layout = self.image_usages[read_usage.0].preferred_layout;
    //                     version_layout.read_layout = preferred_read_layout;
    //
    //                     if self.image_usages[read_usage.0].usage_type == RenderGraphImageUsageType::Output {
    //                         version_layout.write_pass_end_layout = version_layout.read_layout;
    //                         println!("Layout Assignment Image: {:?} Version: {:?} Read: {:?}", physical_image, version, version_layout.read_layout);
    //                     } else {
    //                         version_layout.write_pass_end_layout = version_layout.write_layout;
    //                         println!("Layout Assignment Image: {:?} Version: {:?} Read: {:?}", physical_image, version, version_layout.write_layout);
    //                     }
    //                 } else {}
    //             } else {
    //                 //TODO: Make this flags based (so we can better support READ_ONLY vs. writable
    //                 //TODO: Transitioning to GENERAL if there are multiple conflicting readers may
    //                 // avoid some barriers if all the readers can start at the same time, but we could
    //                 // instead consider only finding intersections of compatible layouts if it's by
    //                 // the same node. This would be potentially less parallel but there's a good chance
    //                 // cases like this wouldn't always be able to run in parallel for other reasons
    //                 let mut all_reads_preferred_layout = None;
    //                 for read_usage in &version_info.read_usages {
    //                     let read_preferred_layout = self.image_usages[read_usage.0].preferred_layout;
    //                     if all_reads_preferred_layout.is_none() {
    //                         all_reads_preferred_layout = Some(read_preferred_layout);
    //                     } else if all_reads_preferred_layout.unwrap() != read_preferred_layout {
    //                         all_reads_preferred_layout = Some(dsc::ImageLayout::General);
    //                     }
    //                 }
    //
    //                 version_layout.read_layout = all_reads_preferred_layout.unwrap();
    //                 version_layout.write_pass_end_layout = version_layout.read_layout;
    //                 println!("Layout Assignment Image: {:?} Version: {:?} Read: {:?}", physical_image, version, version_layout.read_layout);
    //             }
    //             image_layouts.insert(*version, version_layout);
    //         }
    //     }
    //
    //     DetermineImageLayoutsResult {
    //         image_layouts
    //     }
    // }

    fn build_physical_passes(
        &self,
        node_execution_order: &[RenderGraphNodeId],
        image_constraints: &DetermineImageConstraintsResult,
        physical_images: &AssignPhysicalImagesResult,
        //determine_image_layouts_result: &DetermineImageLayoutsResult
    ) -> Vec<RenderGraphPass> {
        let mut pass_node_sets = Vec::default();

        let mut subpass_nodes = Vec::default();
        for node_id in node_execution_order {
            let mut add_to_current = true;
            for subpass_node in &subpass_nodes {
                if !self.can_merge_nodes(*subpass_node, *node_id, image_constraints, physical_images) {
                    add_to_current = false;
                    break;
                }
            }

            if add_to_current {
                subpass_nodes.push(*node_id);
            } else {
                pass_node_sets.push(subpass_nodes);
                subpass_nodes = Vec::default();
                subpass_nodes.push(*node_id);
            }
        }

        if !subpass_nodes.is_empty() {
            pass_node_sets.push(subpass_nodes);
        }

        //TODO: Populate this based on input images
        //let mut image_layouts : Vec<dsc::ImageLayout> = Vec::with_capacity(physical_images.physical_image_versions.len());
        //image_layouts.resize_with(physical_images.physical_image_versions.len(), || dsc::ImageLayout::Undefined);

        println!("gather pass info");
        let mut passes = Vec::default();
        for pass_node_set in pass_node_sets {
            println!("  nodes in pass: {:?}", pass_node_set);
            fn find_or_insert_attachment(attachments: &mut Vec<RenderGraphPassAttachment>, image: PhysicalImageId) -> (usize, bool) {
                if let Some(position) = attachments.iter().position(|x| x.image == image) {
                    (position, false)
                } else {
                    attachments.push(RenderGraphPassAttachment {
                        image,
                        load_op: vk::AttachmentLoadOp::DONT_CARE,
                        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                        store_op: vk::AttachmentStoreOp::DONT_CARE,
                        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                        clear_color: Default::default(),
                        format: vk::Format::UNDEFINED,
                        samples: vk::SampleCountFlags::TYPE_1,
                        initial_layout: dsc::ImageLayout::Undefined,
                        final_layout: dsc::ImageLayout::Undefined
                    });
                    (attachments.len() - 1, true)
                }
            }

            let mut pass = RenderGraphPass {
                attachments: Default::default(),
                subpasses: Default::default(),
            };

            for node_id in pass_node_set {
                println!("    subpass node: {:?}", node_id);
                let mut subpass = RenderGraphSubpass {
                    node: node_id,
                    color_attachments: Default::default(),
                    resolve_attachments: Default::default(),
                    depth_attachment: Default::default()
                };

                let subpass_node = self.node(node_id);

                for (color_attachment_index, color_attachment) in subpass_node.color_attachments.iter().enumerate() {
                    if let Some(color_attachment) = color_attachment {
                        let read_or_write_usage = color_attachment.read_image.or(color_attachment.write_image).unwrap();
                        let physical_image = physical_images.map_image_to_physical.get(&read_or_write_usage).unwrap();
                        let version_id = self.image_version_id(read_or_write_usage);
                        let specification = image_constraints.images.get(&read_or_write_usage).unwrap();
                        println!("      physical attachment (color): {:?}", physical_image);

                        let (pass_attachment_index, is_first_usage) = find_or_insert_attachment(&mut pass.attachments, *physical_image);
                        subpass.color_attachments[color_attachment_index] = Some(pass_attachment_index);

                        let mut attachment = &mut pass.attachments[pass_attachment_index];
                        if is_first_usage {
                            // Check if we load or clear
                            if color_attachment.read_image.is_some() {
                                attachment.load_op = vk::AttachmentLoadOp::LOAD;
                            } else if color_attachment.clear_color_value.is_some() {
                                attachment.load_op = vk::AttachmentLoadOp::CLEAR;
                                attachment.clear_color = Some(AttachmentClearValue::Color(color_attachment.clear_color_value.unwrap()))
                            };

                            attachment.format = specification.format;
                            attachment.samples = specification.samples;
                            //attachment.initial_layout = image_layouts[physical_image.0];

                            // attachment.initial_layout = if let Some(read_image) = color_attachment.read_image {
                            //     determine_image_layouts_result.image_layouts[&read_image]
                            // } else {
                            //     dsc::ImageLayout::Undefined
                            // };
                        };

                        let store_op = if let Some(write_image) = color_attachment.write_image {
                            if !self.image_version_info(write_image).read_usages.is_empty() {
                                vk::AttachmentStoreOp::STORE
                            } else {
                                vk::AttachmentStoreOp::DONT_CARE
                            }
                        } else {
                            vk::AttachmentStoreOp::DONT_CARE
                        };

                        attachment.store_op = store_op;
                        attachment.stencil_store_op = vk::AttachmentStoreOp::DONT_CARE;
                        //attachment.final_layout = determine_image_layouts_result.image_layouts[&version_id].write_pass_end_layout;
                        //image_layouts[physical_image.0] = attachment.final_layout;

                        // attachment.final_layout = if let Some(write_image) = color_attachment.write_image {
                        //     determine_image_layouts_result.image_layouts[&write_image]
                        // } else {
                        //     dsc::ImageLayout::Undefined
                        // };
                    }
                }

                for (resolve_attachment_index, resolve_attachment) in subpass_node.resolve_attachments.iter().enumerate() {
                    if let Some(resolve_attachment) = resolve_attachment {
                        let write_image = resolve_attachment.write_image;
                        let physical_image = physical_images.map_image_to_physical.get(&write_image).unwrap();
                        let version_id = self.image_version_id(write_image);
                        let specification = image_constraints.images.get(&write_image).unwrap();
                        println!("      physical attachment (resolve): {:?}", physical_image);

                        let (pass_attachment_index, is_first_usage) = find_or_insert_attachment(&mut pass.attachments, *physical_image);
                        subpass.resolve_attachments[resolve_attachment_index] = Some(pass_attachment_index);

                        assert!(is_first_usage); // Not sure if this assert is valid
                        let mut attachment = &mut pass.attachments[pass_attachment_index];
                        attachment.format = specification.format;
                        attachment.samples = specification.samples;
                        //attachment.initial_layout = image_layouts[physical_image.0];

                        //attachment.initial_layout = dsc::ImageLayout::Undefined;
                        //attachment.final_layout = determine_image_layouts_result.image_layouts[&write_image];

                        //TODO: Should we skip resolving if there is no reader?
                        let store_op = if !self.image_version_info(write_image).read_usages.is_empty() {
                            vk::AttachmentStoreOp::STORE
                        } else {
                            vk::AttachmentStoreOp::DONT_CARE
                        };

                        attachment.store_op = store_op;
                        attachment.stencil_store_op = vk::AttachmentStoreOp::DONT_CARE;
                        //attachment.final_layout = determine_image_layouts_result.image_layouts[&version_id].write_pass_end_layout;
                        //image_layouts[physical_image.0] = attachment.final_layout;
                    }
                }

                if let Some(depth_attachment) = &subpass_node.depth_attachment {
                    let read_or_write_usage = depth_attachment.read_image.or(depth_attachment.write_image).unwrap();
                    let physical_image = physical_images.map_image_to_physical.get(&read_or_write_usage).unwrap();
                    let version_id = self.image_version_id(read_or_write_usage);
                    let specification = image_constraints.images.get(&read_or_write_usage).unwrap();
                    println!("      physical attachment (depth): {:?}", physical_image);

                    let (pass_attachment_index, is_first_usage) = find_or_insert_attachment(&mut pass.attachments, *physical_image);
                    subpass.depth_attachment = Some(pass_attachment_index);

                    let mut attachment = &mut pass.attachments[pass_attachment_index];
                    if is_first_usage {
                        // Check if we load or clear
                        //TODO: Support load_op for stencil
                        if depth_attachment.read_image.is_some() {
                            attachment.load_op = vk::AttachmentLoadOp::LOAD;
                            attachment.stencil_load_op = vk::AttachmentLoadOp::LOAD;
                        } else if depth_attachment.clear_depth_stencil_value.is_some() {
                            attachment.load_op = vk::AttachmentLoadOp::CLEAR;
                            attachment.stencil_load_op = vk::AttachmentLoadOp::CLEAR;
                            attachment.clear_color = Some(AttachmentClearValue::DepthStencil(depth_attachment.clear_depth_stencil_value.unwrap()))
                        };

                        attachment.format = specification.format;
                        attachment.samples = specification.samples;
                        //attachment.initial_layout = image_layouts[physical_image.0];

                        // attachment.initial_layout = if let Some(read_image) = depth_attachment.read_image {
                        //     determine_image_layouts_result.image_layouts[&read_image]
                        // } else {
                        //     dsc::ImageLayout::Undefined
                        // };
                    };

                    let store_op = if let Some(write_image) = depth_attachment.write_image {
                        if !self.image_version_info(write_image).read_usages.is_empty() {
                            vk::AttachmentStoreOp::STORE
                        } else {
                            vk::AttachmentStoreOp::DONT_CARE
                        }
                    } else {
                        vk::AttachmentStoreOp::DONT_CARE
                    };

                    attachment.store_op = store_op;
                    attachment.stencil_store_op = store_op;
                    //attachment.final_layout = determine_image_layouts_result.image_layouts[&version_id].write_pass_end_layout;
                    //image_layouts[physical_image.0] = attachment.final_layout;

                    // attachment.final_layout = if let Some(write_image) = depth_attachment.write_image {
                    //     determine_image_layouts_result.image_layouts[&write_image]
                    // } else {
                    //     dsc::ImageLayout::Undefined
                    // };
                }

                //TODO: Input attachments

                pass.subpasses.push(subpass);
            }

            passes.push(pass);
        }

        passes
    }

    fn build_node_barriers(
        &self,
        node_execution_order: &[RenderGraphNodeId],
        image_constraints: &DetermineImageConstraintsResult,
        physical_images: &AssignPhysicalImagesResult,
        //determine_image_layouts_result: &DetermineImageLayoutsResult,
    ) -> Vec<RenderGraphNodeImageBarriers> {
        let mut barriers = Vec::default();

        for node_id in node_execution_order {
            let node = self.node(*node_id);
            //let mut invalidate_barriers: FnvHashMap<PhysicalImageId, RenderGraphImageBarrier> = Default::default();
            //let mut flush_barriers: FnvHashMap<PhysicalImageId, RenderGraphImageBarrier> = Default::default();
            let mut node_barriers : FnvHashMap<PhysicalImageId, RenderGraphPassImageBarriers> = Default::default();

            for (color_attachment_index, color_attachment) in node.color_attachments.iter().enumerate() {
                if let Some(color_attachment) = color_attachment {
                    let read_or_write_usage = color_attachment.read_image.or(color_attachment.write_image).unwrap();
                    let physical_image = physical_images.map_image_to_physical.get(&read_or_write_usage).unwrap();
                    let version_id = self.image_version_id(read_or_write_usage);

                    let mut barrier = node_barriers.entry(*physical_image)
                        .or_insert_with(|| RenderGraphPassImageBarriers::new(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL));

                    if let Some(read_image) = color_attachment.read_image {
                        barrier.invalidate.access_flags |= vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::COLOR_ATTACHMENT_READ;
                        barrier.invalidate.stage_flags |= vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
                        //barrier.invalidate.layout = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
                        //invalidate_barrier.layout = determine_image_layouts_result.image_layouts[&version_id].read_layout.into();
                    }

                    if let Some(write_image) = color_attachment.write_image {
                        barrier.flush.access_flags |= vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
                        barrier.flush.stage_flags |= vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
                        //barrier.flush.layout = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
                        //flush_barrier.layout = determine_image_layouts_result.image_layouts[&version_id].write_layout.into();
                    }
                }
            }

            for (resolve_attachment_index, resolve_attachment) in node.resolve_attachments.iter().enumerate() {
                if let Some(resolve_attachment) = resolve_attachment {
                    let physical_image = physical_images.map_image_to_physical.get(&resolve_attachment.write_image).unwrap();
                    let version_id = self.image_version_id(resolve_attachment.write_image);

                    let mut barrier = node_barriers.entry(*physical_image)
                        .or_insert_with(|| RenderGraphPassImageBarriers::new(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL));

                    barrier.flush.access_flags |= vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
                    barrier.flush.stage_flags |= vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
                    //barrier.flush.layout = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
                    //flush_barrier.layout = determine_image_layouts_result.image_layouts[&version_id].write_layout.into();
                }
            }

            if let Some(depth_attachment) = &node.depth_attachment {
                let read_or_write_usage = depth_attachment.read_image.or(depth_attachment.write_image).unwrap();
                let physical_image = physical_images.map_image_to_physical.get(&read_or_write_usage).unwrap();
                let version_id = self.image_version_id(read_or_write_usage);

                let mut barrier = node_barriers.entry(*physical_image)
                    .or_insert_with(|| RenderGraphPassImageBarriers::new(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL));

                if depth_attachment.read_image.is_some() && depth_attachment.write_image.is_some() {

                    //barrier.invalidate.layout = vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
                    //barrier.invalidate.layout = determine_image_layouts_result.image_layouts[&version_id].read_layout.into();
                    barrier.invalidate.access_flags |= vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;
                    barrier.invalidate.stage_flags |= vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS;

                    //barrier.flush.layout = vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
                    //barrier.flush.layout = determine_image_layouts_result.image_layouts[&version_id].write_layout.into();
                    barrier.flush.access_flags |= vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;
                    barrier.flush.stage_flags |= vk::PipelineStageFlags::LATE_FRAGMENT_TESTS;
                } else if depth_attachment.read_image.is_some() {
                    //barrier.invalidate.layout = vk::ImageLayout::DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL;
                    //barrier.invalidate.layout = determine_image_layouts_result.image_layouts[&version_id].read_layout.into();
                    barrier.invalidate.access_flags |= vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ;
                    barrier.invalidate.stage_flags |= vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS;
                } else {
                    assert!(depth_attachment.write_image.is_some());
                    //barrier.flush.layout = vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
                    //barrier.flush.layout = determine_image_layouts_result.image_layouts[&version_id].write_layout.into();
                    barrier.flush.access_flags |= vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;
                    barrier.flush.stage_flags |= vk::PipelineStageFlags::LATE_FRAGMENT_TESTS;
                }
            }

            // barriers.push(RenderGraphNodeImageBarriers {
            //     invalidates: invalidate_barriers,
            //     flushes: flush_barriers
            // });
            barriers.push(RenderGraphNodeImageBarriers {
                barriers: node_barriers
            })
        }

        barriers
    }

    // * At this point we know format, samples, load_op, stencil_load_op, and initial_layout. We also
    //   know what needs to be flushed/invalidated
    // * We want to determine store_op, stencil_store_op, final_layout. And the validates/flushes
    //   we actually need to insert
    fn build_pass_barriers(
        &self,
        node_execution_order: &[RenderGraphNodeId],
        image_constraints: &DetermineImageConstraintsResult,
        physical_images: &AssignPhysicalImagesResult,
        node_barriers: &[RenderGraphNodeImageBarriers],
        passes: &mut [RenderGraphPass]
    ) -> Vec<Vec<dsc::SubpassDependency>> {
        println!("-- build_pass_barriers --");
        const MAX_PIPELINE_FLAG_BITS : usize = 15;
        let ALL_GRAPHICS : vk::PipelineStageFlags = vk::PipelineStageFlags::from_raw(0b111_1111_1110);

        struct ImageState {
            layout: vk::ImageLayout,
            pending_flush_access_flags: vk::AccessFlags,
            pending_flush_pipeline_stage_flags: vk::PipelineStageFlags,
            // One per pipeline stage
            invalidated: [vk::AccessFlags; MAX_PIPELINE_FLAG_BITS],
        }

        impl Default for ImageState {
            fn default() -> Self {
                ImageState {
                    layout: vk::ImageLayout::UNDEFINED,
                    pending_flush_access_flags: Default::default(),
                    pending_flush_pipeline_stage_flags: Default::default(),
                    invalidated: Default::default()
                }
            }
        }

        // to support subpass, probably need image states for each previous subpass

        let mut image_states : Vec<ImageState> = Vec::with_capacity(physical_images.physical_image_versions.len());
        image_states.resize_with(physical_images.physical_image_versions.len(), || Default::default());

        let mut pass_dependencies = Vec::default();

        for (pass_index, pass) in passes.iter_mut().enumerate() {
            println!("pass {}", pass_index);
            let mut subpass_dependencies = Vec::default();
            let mut attachment_initial_layout : Vec<Option<dsc::ImageLayout>> = Default::default();
            attachment_initial_layout.resize_with(pass.attachments.len(), || None);

            //TODO: This does not support multipass
            assert_eq!(pass.subpasses.len(), 1);
            for (subpass_index, subpass) in pass.subpasses.iter_mut().enumerate() {
                println!("  subpass {}", subpass_index);
                let node = self.node(subpass.node);
                let node_barriers = &node_barriers[subpass.node.0];

                // Accumulate the invalidates here
                let mut invalidate_src_access_flags = vk::AccessFlags::empty();
                let mut invalidate_src_pipeline_stage_flags = vk::PipelineStageFlags::empty();
                let mut invalidate_dst_access_flags = vk::AccessFlags::empty();
                let mut invalidate_dst_pipeline_stage_flags = vk::PipelineStageFlags::empty();

                // Look at all the images we read and determine what invalidates we need
                for (physical_image_id, image_barrier) in &node_barriers.barriers {
                    println!("    image {:?}", physical_image_id);
                    let image_state = &mut image_states[physical_image_id.0];

                    // Include the previous writer's stage/access flags, if there were any
                    invalidate_src_access_flags |= image_state.pending_flush_access_flags;
                    invalidate_src_pipeline_stage_flags |= image_state.pending_flush_pipeline_stage_flags;

                    // layout changes are write operations and can cause hazards. We need to
                    // block on any stages before that are reading or writing
                    let layout_change = image_state.layout != image_barrier.layout;
                    if layout_change {
                        println!("      layout change! {:?} -> {:?}", image_state.layout, image_barrier.layout);
                        for i in 0..MAX_PIPELINE_FLAG_BITS {
                            if image_state.invalidated[i] != vk::AccessFlags::empty() {
                                // Add an execution barrier if we are transitioning the layout
                                // of something that is already being read from
                                let pipeline_stage = vk::PipelineStageFlags::from_raw(1 << i);
                                println!("        add src execution barrier on stage {:?}", pipeline_stage);
                                invalidate_src_pipeline_stage_flags |= pipeline_stage;
                                //invalidate_dst_pipeline_stage_flags |= image_barrier.invalidate.stage_flags;
                                //invalidate_dst_pipeline_stage_flags |= image_barrier.flush.stage_flags;
                            }

                            image_state.invalidated[i] = vk::AccessFlags::empty();
                        }

                        // And clear invalidation flag to require the image to be loaded
                        println!("        cleared all invalidated bits for image {:?}", physical_image_id);
                    }

                    // Requirements for this image
                    let mut image_invalidate_access_flags = image_barrier.invalidate.access_flags;
                    let mut image_invalidate_pipeline_stage_flags = image_barrier.invalidate.stage_flags;

                    //TODO: Should I OR in the flush access/stages? Right now the invalidate barrier is including write
                    // access flags in the invalidate **but only for modifies**
                    image_invalidate_access_flags |= image_barrier.flush.access_flags;
                    image_invalidate_pipeline_stage_flags |= image_barrier.flush.stage_flags;

                    // Check if we have already done invalidates for this image previously, allowing
                    // us to skip some now
                    for i in 0..MAX_PIPELINE_FLAG_BITS {
                        let pipeline_stage = vk::PipelineStageFlags::from_raw(1<<i);
                        if pipeline_stage.intersects(image_invalidate_pipeline_stage_flags) {
                            // If the resource has been invalidate in this stage, we don't need to include this stage
                            // in the invalidation barrier
                            if image_state.invalidated[i].contains(image_invalidate_access_flags) {
                                println!("      skipping invalidation for {:?} {:?}", pipeline_stage, image_invalidate_access_flags);
                                image_invalidate_pipeline_stage_flags &= !pipeline_stage;
                            }
                        }
                    }

                    // All pipeline stages have seen invalidates for the relevant access flags
                    // already, so we don't need to do invalidates at all.
                    if image_invalidate_pipeline_stage_flags == vk::PipelineStageFlags::empty() {
                        println!("      no invalidation required, clearing access flags");
                        image_invalidate_access_flags = vk::AccessFlags::empty();
                    }

                    println!("      Access Flags: {:?}", image_invalidate_access_flags);
                    println!("      Pipeline Stage Flags: {:?}", image_invalidate_pipeline_stage_flags);

                    // OR the requirements in
                    invalidate_dst_access_flags |= image_invalidate_access_flags;
                    invalidate_dst_pipeline_stage_flags |= image_invalidate_pipeline_stage_flags;

                    // Set the initial layout for the attachment, but only if it's the first time we've seen it
                    //TODO: This is bad and does not properly handle an image being used in multiple ways
                    for (attachment_index, attachment) in &mut pass.attachments.iter_mut().enumerate() {
                        //println!("      attachment {:?}", attachment.image);
                        if attachment.image == *physical_image_id {
                            if attachment_initial_layout[attachment_index].is_none() {
                                //println!("        initial layout {:?}", image_barrier.layout);
                                attachment_initial_layout[attachment_index] = Some(image_state.layout.into());
                                attachment.initial_layout = image_state.layout.into();
                            }

                            attachment.final_layout = image_barrier.layout.into();
                            break;
                        }
                    }

                    image_state.layout = image_barrier.layout;
                }
                //
                // for (physical_image_id, image_barrier) in &node_barriers.flushes {
                //     println!("    flush");
                //     let image_state = &mut image_states[physical_image_id.0];
                //
                //     for i in 0..MAX_PIPELINE_FLAG_BITS {
                //         if image_state.invalidated[i] != vk::AccessFlags::empty() {
                //             // Add an execution barrier if we are writing on something that
                //             // is already being read from
                //             let pipeline_stage = vk::PipelineStageFlags::from_raw(1 << i);
                //             invalidate_src_pipeline_stage_flags |= pipeline_stage;
                //             invalidate_dst_pipeline_stage_flags |= image_barrier.stage_flags;
                //         }
                //     }
                //
                //     for (attachment_index, attachment) in &mut pass.attachments.iter_mut().enumerate() {
                //         println!("      attachment {:?}", attachment.image);
                //         if attachment.image == *physical_image_id {
                //             println!("        final layout {:?}", image_barrier.layout);
                //             attachment.final_layout = image_barrier.layout.into();
                //             break;
                //         }
                //     }
                //
                //     assert!(image_state.layout == vk::ImageLayout::UNDEFINED || image_state.layout == image_barrier.layout);
                //     if image_state.layout != image_barrier.layout {
                //         invalidate_dst_pipeline_stage_flags |= image_barrier.stage_flags;
                //         invalidate_dst_access_flags |= image_barrier.access_flags;
                //     }
                //
                //     //image_state.layout = image_barrier.layout;
                // }

                // Update the image states
                for image_state in &mut image_states {
                    // Mark pending flushes as handled
                    //TODO: Check that ! is inverting bits
                    image_state.pending_flush_access_flags &= !invalidate_src_access_flags;
                    image_state.pending_flush_pipeline_stage_flags &= !invalidate_src_pipeline_stage_flags;

                    // Mark resources that we are invalidating as having been invalidated for
                    // the appropriate pipeline stages
                    //TODO: Invalidate all later stages?
                    for i in 0..MAX_PIPELINE_FLAG_BITS {
                        let pipeline_stage = vk::PipelineStageFlags::from_raw(1 << i);
                        if pipeline_stage.intersects(invalidate_dst_pipeline_stage_flags) {
                            image_state.invalidated[i] |= invalidate_dst_access_flags;
                        }
                    }
                }

                // Build a subpass dependency (EXTERNAL -> 0)
                let invalidate_subpass_dependency = dsc::SubpassDependency {
                    dependency_flags: dsc::DependencyFlags::Empty,
                    src_access_mask: dsc::AccessFlags::from_access_flag_mask(invalidate_src_access_flags),
                    src_stage_mask: dsc::PipelineStageFlags::from_pipeline_stage_mask(invalidate_src_pipeline_stage_flags),
                    dst_access_mask: dsc::AccessFlags::from_access_flag_mask(invalidate_dst_access_flags),
                    dst_stage_mask: dsc::PipelineStageFlags::from_pipeline_stage_mask(invalidate_dst_pipeline_stage_flags),
                    src_subpass: dsc::SubpassDependencyIndex::External,
                    dst_subpass: dsc::SubpassDependencyIndex::Index(0)
                };
                subpass_dependencies.push(invalidate_subpass_dependency);

                for (physical_image_id, image_barrier) in &node_barriers.barriers {
                    let image_state = &mut image_states[physical_image_id.0];

                    // Queue up flushes to happen later
                    image_state.pending_flush_pipeline_stage_flags |= image_barrier.flush.stage_flags;
                    image_state.pending_flush_access_flags |= image_barrier.flush.access_flags;

                    // If we write something, mark it as no longer invalidated
                    //TODO: Not sure if we invalidate specific stages or all stages
                    //TODO: Can we invalidate specific access instead of all access?
                    for i in 0..MAX_PIPELINE_FLAG_BITS {
                        image_state.invalidated[i] = vk::AccessFlags::empty();
                    }

                    // let layout_change = image_state.layout != image_barrier.layout;
                    // if layout_change {
                    //     for i in 0..MAX_PIPELINE_FLAG_BITS {
                    //         if image_state.invalidated[i] != vk::AccessFlags::empty() {
                    //             // Add an execution barrier if we are transitioning the layout
                    //             // of something that is already being read from
                    //             let pipeline_stage = vk::PipelineStageFlags::from_raw(1 << i);
                    //             invalidate_src_pipeline_stage_flags |= pipeline_stage;
                    //             invalidate_dst_pipeline_stage_flags |= image_barrier.stage_flags;
                    //         }
                    //     }
                    // }
                }



                //TODO: This hack clears final layout for attachments with DONT_CARE store_op. This is happening
                // for inserted resolves because the color attachment still has a write usage (and must have it
                // to put the attachment on the renderpass) but this is also creating a flush for the image which
                // means it gets placed into a layout
                for (attachment_index, attachment) in &mut pass.attachments.iter_mut().enumerate() {
                    if attachment.store_op == vk::AttachmentStoreOp::DONT_CARE && attachment.stencil_store_op == vk::AttachmentStoreOp::DONT_CARE {
                        attachment.final_layout = dsc::ImageLayout::Undefined;
                    }
                }

                // TODO: Figure out how to handle output images
                // TODO: This only works if no one else reads it?
                for (output_image_index, output_image) in self.output_images.iter().enumerate() {
                    if self.image_version_info(output_image.usage).creator_node == subpass.node {
                        //output_image.
                        //self.image_usages[output_image.usage]

                        let output_physical_image = physical_images.map_image_to_physical[&output_image.usage];

                        for (attachment_index, attachment) in &mut pass.attachments.iter_mut().enumerate() {
                            if attachment.image == output_physical_image {
                                attachment.final_layout = output_image.final_layout;
                            }
                        }

                        //TODO: Need a 0 -> EXTERNAL dependency here
                    }
                }


                //TODO: Need to do a dependency? Maybe by adding a flush?
            }

            pass_dependencies.push(subpass_dependencies);
        }

        pass_dependencies
    }

    pub fn prepare(&mut self) {
        //
        // Walk backwards through the DAG, starting from the output images, through all the upstream
        // dependencies of those images. We are doing a depth first search. Nodes that make no
        // direct or indirect contribution to an output image will not be included. As an
        // an implementation detail, we try to put renderpass merge candidates adjacent to each
        // other in this list
        //
        let node_execution_order = self.determine_node_order();

        // Print out the execution order
        println!("Execution order of unculled nodes:");
        for node in &node_execution_order {
            println!("  Node {:?} {:?}", node, self.node(*node).name());
        }

        //
        // Traverse the graph to determine specifications for all images that will be used. This
        // iterates forwards and backwards through the node graph. This allows us to specify
        // attributes about images (like format, sample count) in key areas and infer it elsewhere.
        // If there is not enough information to infer then the render graph cannot be used.
        //
        let mut image_constraint_results = self.determine_image_constraints(&node_execution_order);

        // Print out the constraints assigned to images
        self.print_image_constraints(&mut image_constraint_results);

        //
        // Add resolves to the graph - this will occur when a renderpass outputs a multisample image
        // to a renderpass that is expecting a non-multisampled image.
        //
        self.insert_resolves(&node_execution_order, &mut image_constraint_results);
        self.print_image_constraints(&mut image_constraint_results);

        // Print the cases where we can't reuse images
        self.print_image_compatibility(&image_constraint_results);

        //
        // Assign logical images to physical images. This should give us a minimal number of images.
        // This does not include aliasing images during graph execution. We handle this later.
        //
        let assign_physical_images_result =
            self.assign_physical_images(&node_execution_order, &mut image_constraint_results);
        println!("Physical image usage:");
        for (physical_image_id, logical_image_id_list) in
            &assign_physical_images_result.physical_image_usages
        {
            println!("  Physical image: {:?}", physical_image_id);
            for logical_image in logical_image_id_list {
                println!("    {:?}", logical_image);
            }
        }

        //let determine_image_layouts_result = self.determine_image_layouts(&node_execution_order, &image_constraint_results, &assign_physical_images_result);

        // Print the physical images
        self.print_physical_image_usage(&assign_physical_images_result/*, &determine_image_layouts_result*/);

        //
        // Combine nodes into passes where possible
        //
        let mut passes = self.build_physical_passes(&node_execution_order, &image_constraint_results, &assign_physical_images_result/*, &determine_image_layouts_result*/);
        println!("Merged Renderpasses:");
        for (index, pass) in passes.iter().enumerate() {
            println!("  pass {}", index);
            println!("    attachments:");
            for attachment in &pass.attachments {
                println!("      {:?}", attachment);
            }
            println!("    subpasses:");
            for subpass in &pass.subpasses {
                println!("      {:?}", subpass);
            }
        }

        let node_barriers = self.build_node_barriers(&node_execution_order, &image_constraint_results, &assign_physical_images_result/*, &determine_image_layouts_result*/);
        println!("Barriers:");
        for (index, pass) in node_barriers.iter().enumerate() {
            println!("  pass {}", index);
            println!("    invalidates");
            for (physical_id, barriers) in &pass.barriers {
                println!("      {:?}: {:?}", physical_id, barriers.invalidate);
            }
            println!("    flushes");
            for (physical_id, barriers) in &pass.barriers {
                println!("      {:?}: {:?}", physical_id, barriers.flush);
            }
        }

        //TODO: Figure out in/out layouts for passes? Maybe insert some other fixes? Drop transient
        // images?

        // Print out subpass
        let subpass_dependencies = self.build_pass_barriers(
            &node_execution_order,
            &image_constraint_results,
            &assign_physical_images_result,
            &node_barriers,
            &mut passes
        );

        println!("Merged Renderpasses:");
        for (index, pass) in passes.iter().enumerate() {
            println!("  pass {}", index);
            println!("    attachments:");
            for attachment in &pass.attachments {
                println!("      {:?}", attachment);
            }
            println!("    subpasses:");
            for subpass in &pass.subpasses {
                println!("      {:?}", subpass);
            }
            println!("    dependencies:");
            for subpass in &subpass_dependencies[index] {
                println!("      {:#?}", subpass);
            }
        }


        //TODO: Cull images that only exist within the lifetime of a single pass? (just passed among
        // subpasses)






        // struct PhysicalImageUsageInfo {
        //     // aspect and other bits?
        //     layout: vk::ImageLayout,
        //     //bit: vk::ImageAspectFlags,
        // }

        // let read_requirements = self.determine_physical_image_states(
        //     &node_execution_order,
        //     &image_constraint_results,
        //     &assign_physical_images_result,
        // );

        // At some point need to walk through nodes to place barriers and see about aliasing/pooling images

        //
        // Execute the graph
        //
        // println!("-------------- EXECUTE --------------");
        // self.create_renderpasses(
        //     &node_execution_order,
        //     &image_constraint_results,
        //     &assign_physical_images_result,
        // );





        /*
                for (node_index, node) in self.nodes.iter().enumerate() {
                    println!("Record node {:?} {:?}", node.id(), node.name());
                    // if let Some(action) = node_actions[node_index].take() {
                    //     action.record(self, &image_constraint_results);
                    // }

                    // Any output may need to be adjusted
                    for create in &node.image_creates {
                        println!("Process create {:?}", create);
                        let reader_count = self.image_version_info(create.image).read_usages.len();
                        println!("  read count: {}", reader_count);
                    }

                    // Any output may need to be adjusted
                    for modify in &node.image_modifies {
                        println!("Process modify {:?}", modify);
                        let reader_count = self.image_version_info(modify.output).read_usages.len();
                        println!("  read count: {}", reader_count);
                    }
                }
        */
        // * Traverse graphs and figure out all read/write requirements
        // * See if writers are able to change output to match their readers
        // * See if writers are able to merge with readers (must be single writer to single reader)
        //   - Don't bother with it right now
        // * Try to merge image usage?
        // *
        // * Allocate/reuse frame buffers?

        //
        // Find the lifetimes and sequence of events for all image resources
        //

        //
        // Merge passes? Make sure that if we merge A, B, C, that A ends up with all the reads/writes
        // from B and C. Or.. we can reorder to ensure mergable passes are next to each other
        //

        //
        // Optimize ordering to delay nodes that have dependencies that are nearby in the ordering.
        //

        //
        // Calculate physical images.
        //
        //TODO: If we are binding images to input/output names, we could have the same image bound
        // for both input and output. They need to have the same physical index. The other way
        // is we can use CLEAR/LOAD/STORE ops to know read/write and we bind via attachment name

        //
        // Merge render passes if not already done
        //

        //
        // Get rid of images that are only used between subpasses
        //

        //
        // Trace backwards to determine what states images should be in?
        //
        // Insert barriers? Modify passes? Alias images?
    }

    fn print_physical_image_usage(&mut self, assign_physical_images_result: &AssignPhysicalImagesResult/*, determine_image_layouts_result: &DetermineImageLayoutsResult*/) {
        println!("Physical image usage:");
        for (physical_image_id, versions) in &assign_physical_images_result.physical_image_versions {
            println!("  image: {:?}", physical_image_id);
            for version_id in versions {
                println!("  version_id {:?}", version_id);
                let version = &mut self.image_resources[version_id.index].versions[version_id.version];
                println!("  create: {:?}", version.create_usage);
                //println!("  create: {:?} {:?}", version.create_usage, self.image_usages[version.create_usage.0].preferred_layout);
                //println!("    create: {:?} {:?}", version.create_usage, determine_image_layouts_result.image_layouts[version_id].write_layout);
                for read in &version.read_usages {
                    println!("    read: {:?}", read);
                    //println!("    read: {:?} {:?}", read, self.image_usages[read.0].preferred_layout);
                    //println!("      read: {:?} {:?}", read, determine_image_layouts_result.image_layouts[version_id].read_layout);
                }
            }
        }
    }

    fn print_image_constraints(
        &self,
        image_constraint_results: &mut DetermineImageConstraintsResult,
    ) {
        println!("Image constraints:");
        for (image_index, image_resource) in self.image_resources.iter().enumerate() {
            println!("  Image {:?} {:?}", image_index, image_resource.name);
            for (version_index, version) in image_resource.versions.iter().enumerate() {
                println!("    Version {}", version_index);

                println!(
                    "      Writen as: {:?}",
                    image_constraint_results.specification(version.create_usage)
                );

                for (usage_index, usage) in version.read_usages.iter().enumerate() {
                    println!(
                        "      Read Usage {}: {:?}",
                        usage_index,
                        image_constraint_results.specification(*usage)
                    );
                }
            }
        }
    }

    fn print_image_compatibility(
        &self,
        image_constraint_results: &DetermineImageConstraintsResult,
    ) {
        println!("Image Compatibility Report:");
        for (image_index, image_resource) in self.image_resources.iter().enumerate() {
            println!("  Image {:?} {:?}", image_index, image_resource.name);
            for (version_index, version) in image_resource.versions.iter().enumerate() {
                let write_specification =
                    image_constraint_results.specification(version.create_usage);

                println!("    Version {}: {:?}", version_index, version);
                for (usage_index, usage) in version.read_usages.iter().enumerate() {
                    let read_specification = image_constraint_results.specification(*usage);

                    // TODO: Skip images we don't use?

                    if write_specification == read_specification {
                        println!("      read usage {} matches", usage_index);
                    } else {
                        println!("      read usage {} does not match", usage_index);
                        println!("        produced: {:?}", write_specification);
                        println!("        required: {:?}", read_specification);
                    }
                }
            }
        }
    }


    // fn collect_read_requirements(
    //     &self,
    //     node_execution_order: &[RenderGraphNodeId],
    //     image_constraints: &DetermineImageConstraintsResult,
    //     physical_images: &AssignPhysicalImagesResult,
    // ) {
    // }

    /*
        fn pick_write_layout(
            &self,
            physical_image_state: &mut PhysicalImageState,
            write_usage_id: Option<RenderGraphImageUsageId>,
        ) -> dsc::ImageLayout {
            if write_usage_id.is_none() {
                return dsc::ImageLayout::Undefined;
            }

            let version_info = self.image_version_info(write_usage_id.unwrap());

            let mut output_layout = None;
            //println!("pick_write_layout");
            for read_usage in &version_info.read_usages {
                let preferred_layout = self.image_usages[read_usage.0].preferred_layout;
                //println!("  reader {:?}", preferred_layout);
                if output_layout.is_none() {
                    output_layout = Some(preferred_layout);
                } else if *output_layout.as_ref().unwrap() != preferred_layout {
                    output_layout = Some(dsc::ImageLayout::General);
                }
            }

            let output_layout = output_layout
                .or_else(|| Some(dsc::ImageLayout::Undefined))
                .unwrap();
            physical_image_state.layout = output_layout;
            //println!("  output {:?}", output_layout);
            output_layout
        }
        //TODO: Do this per image?
        fn gather_renderpass_sync_flags(&self, node_id: RenderGraphNodeId) -> PassSyncFlags {
            let node = self.node(node_id);

            let mut read_access_flags = vk::AccessFlags::empty();
            let mut read_stage_flags = vk::PipelineStageFlags::empty();
            let mut write_access_flags = vk::AccessFlags::empty();
            let mut write_stage_flags = vk::PipelineStageFlags::empty();

            for attachment_info in &node.color_attachments {
                if let Some(attachment_info) = attachment_info {
                    if let Some(read_image) = attachment_info.read_image {
                        read_access_flags |= self.image_usages[read_image.0].access_flags;
                        read_stage_flags |= self.image_usages[read_image.0].stage_flags;
                    }

                    if let Some(write_image) = attachment_info.write_image {
                        write_access_flags |= self.image_usages[write_image.0].access_flags;
                        write_stage_flags |= self.image_usages[write_image.0].stage_flags;
                    }
                }
            }

            for attachment_info in &node.resolve_attachments {
                if let Some(attachment_info) = attachment_info {
                    if let Some(write_image) = attachment_info.write_image {
                        write_access_flags |= self.image_usages[write_image.0].access_flags;
                        write_stage_flags |= self.image_usages[write_image.0].stage_flags;
                    }
                }
            }

            if let Some(attachment_info) = &node.depth_attachment {
                if let Some(read_image) = attachment_info.read_image {
                    read_access_flags |= self.image_usages[read_image.0].access_flags;
                    read_stage_flags |= self.image_usages[read_image.0].stage_flags;
                }

                if let Some(write_image) = attachment_info.write_image {
                    write_access_flags |= self.image_usages[write_image.0].access_flags;
                    write_stage_flags |= self.image_usages[write_image.0].stage_flags;
                }
            }

            PassSyncFlags {
                read_access_flags,
                read_stage_flags,
                write_access_flags,
                write_stage_flags
            }
        }

        fn create_renderpasses(
            &self,
            node_execution_order: &[RenderGraphNodeId],
            image_constraints: &DetermineImageConstraintsResult,
            physical_images: &AssignPhysicalImagesResult,
        ) {
            let mut physical_image_states: FnvHashMap<PhysicalImageId, PhysicalImageState> =
                Default::default();

            #[derive(Default)]
            struct PhysicalImageSyncState {
                pending_write_access_flags: vk::AccessFlags,
                pending_write_pipeline_stage_flags: vk::PipelineStageFlags,
                readable_access_flags_per_stage: Vec<vk::AccessFlags>
            }

            let mut physical_image_sync_states: FnvHashMap<PhysicalImageId, PhysicalImageSyncState> =
                Default::default();

            #[derive(Debug)]
            struct ImageRead {
                image: RenderGraphImageUsageId,
                pipeline_stage_flags: vk::PipelineStageFlags,
                access_flags: vk::AccessFlags,
            }

            #[derive(Debug)]
            struct ImageWrite {
                image: RenderGraphImageUsageId,
                pipeline_stage_flags: vk::PipelineStageFlags,
                access_flags: vk::AccessFlags,
            }

            let physical_image_state : FnvHashMap<PhysicalImageId, PhysicalImageSyncState> = FnvHashMap::default();

            for node_id in node_execution_order {
                let mut attachments = Vec::default();
                let mut subpass = dsc::SubpassDescription::default();
                let mut dependencies = dsc::SubpassDependency::default();

                let node_sync_flags = self.gather_renderpass_sync_flags(*node_id);

                let node = self.node(*node_id);
                println!("record {:?}", node);

                let mut read_images = Vec::default();
                let mut write_images = Vec::default();

                for (index, attachment_info) in node.color_attachments.iter().enumerate() {
                    if attachment_info.is_none() {
                        subpass.color_attachments.push(dsc::AttachmentReference {
                            attachment: dsc::AttachmentIndex::Unused,
                            layout: dsc::ImageLayout::Undefined,
                        });

                        continue;
                    }

                    let attachment_info = attachment_info.as_ref().unwrap();

                    // Modifies have two images, but the spec will be the same for both of them. The
                    // algorithm that determines spec must ensure this because while we have distinct
                    // read/write IDs for the single modify, in the end it must be on a single physical
                    // image.
                    let read_or_write_image = attachment_info
                        .read_image
                        .or(attachment_info.write_image)
                        .unwrap();
                    let specification = image_constraints.specification(read_or_write_image);

                    let physical_image = physical_images
                        .map_image_to_physical
                        .get(&read_or_write_image)
                        .unwrap();
                    let physical_image_state =
                        physical_image_states.entry(*physical_image).or_default();

                    let from_layout = physical_image_state.layout;
                    let to_layout =
                        self.pick_write_layout(physical_image_state, attachment_info.write_image);

                    let attachment = RenderGraph::create_attachment_description(
                        specification,
                        attachment_info.attachment_type,
                        attachment_info.clear_color_value.is_some(),
                        from_layout,
                        to_layout,
                    );

                    //println!("  Color Attachment {}: {:?}", index, attachment);
                    subpass.color_attachments.push(dsc::AttachmentReference {
                        attachment: dsc::AttachmentIndex::Index(attachments.len() as u32),
                        layout: dsc::ImageLayout::ColorAttachmentOptimal,
                    });

                    attachments.push(attachment);

                    if let Some(read_image) = attachment_info.read_image {
                        let access_flags = if attachment_info.write_image.is_some() {
                            vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                        } else {
                            vk::AccessFlags::COLOR_ATTACHMENT_READ
                        };

                        read_images.push(ImageRead {
                            image: read_image,
                            pipeline_stage_flags: vk::PipelineStageFlags::FRAGMENT_SHADER,
                            access_flags
                        });
                    }

                    if let Some(write_image) = attachment_info.write_image {
                        write_images.push(ImageWrite {
                            image: write_image,
                            pipeline_stage_flags: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                            access_flags: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                        });
                    }
                }

                for (index, attachment_info) in node.resolve_attachments.iter().enumerate() {
                    if attachment_info.is_none() {
                        subpass.resolve_attachments.push(dsc::AttachmentReference {
                            attachment: dsc::AttachmentIndex::Unused,
                            layout: dsc::ImageLayout::Undefined,
                        });

                        continue;
                    }

                    let attachment_info = attachment_info.as_ref().unwrap();

                    // Modifies have two images, but the spec will be the same for both of them. The
                    // algorithm that determines spec must ensure this because while we have distinct
                    // read/write IDs for the single modify, in the end it must be on a single physical
                    // image.
                    let image = attachment_info.write_image.unwrap();
                    let specification = image_constraints.specification(image);
                    let physical_image = physical_images.map_image_to_physical.get(&image).unwrap();
                    let physical_image_state =
                        physical_image_states.entry(*physical_image).or_default();

                    let from_layout = physical_image_state.layout;
                    let to_layout = self.pick_write_layout(physical_image_state, Some(image));

                    let attachment = RenderGraph::create_attachment_description(
                        specification,
                        attachment_info.attachment_type,
                        false,
                        from_layout,
                        to_layout,
                    );

                    //println!("  Resolve Attachment {}: {:?}", index, attachment);
                    subpass.resolve_attachments.push(dsc::AttachmentReference {
                        attachment: dsc::AttachmentIndex::Index(attachments.len() as u32),
                        layout: dsc::ImageLayout::ColorAttachmentOptimal,
                    });

                    attachments.push(attachment);

                    if let Some(write_image) = attachment_info.write_image {
                        write_images.push(ImageWrite {
                            image: write_image,
                            pipeline_stage_flags: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                            access_flags: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                        });
                    }
                }

                if let Some(attachment_info) = &node.depth_attachment {
                    // Modifies have two images, but the spec will be the same for both of them. The
                    // algorithm that determines spec must ensure this because while we have distinct
                    // read/write IDs for the single modify, in the end it must be on a single physical
                    // image.
                    let read_or_write_image = attachment_info
                        .read_image
                        .or(attachment_info.write_image)
                        .unwrap();
                    let specification = image_constraints.specification(read_or_write_image);
                    let physical_image = physical_images
                        .map_image_to_physical
                        .get(&read_or_write_image)
                        .unwrap();
                    let physical_image_state =
                        physical_image_states.entry(*physical_image).or_default();

                    let from_layout = physical_image_state.layout;
                    let to_layout =
                        self.pick_write_layout(physical_image_state, attachment_info.write_image);

                    let attachment = RenderGraph::create_attachment_description(
                        specification,
                        attachment_info.attachment_type,
                        attachment_info.clear_depth_stencil_value.is_some(),
                        from_layout,
                        to_layout,
                    );

                    //println!("  Depth Attachment: {:?}", attachment);
                    subpass.depth_stencil_attachment = Some(AttachmentReference {
                        attachment: dsc::AttachmentIndex::Index(attachments.len() as u32),
                        layout: dsc::ImageLayout::DepthAttachmentOptimal, //TODO: There are read/write variants of this
                    });
                    attachments.push(attachment);

                    if let Some(read_image) = attachment_info.read_image {
                        let access_flags = if attachment_info.write_image.is_some() {
                            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE
                        } else {
                            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                        };

                        read_images.push(ImageRead {
                            image: read_image,
                            pipeline_stage_flags: vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                            access_flags
                        });
                    }

                    if let Some(write_image) = attachment_info.write_image {
                        write_images.push(ImageWrite {
                            image: write_image,
                            pipeline_stage_flags: vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                            access_flags: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                        });
                    }

                    //println!("  attachments\n{:#?}", attachments);
                    //println!("  subpass\n{:#?}", subpass);
                }

                println!("  read images: {:?}", read_images);
                println!("  write images: {:?}", write_images);
                println!("  reads: {:?}", read_images);
                println!("  writes: {:?}", write_images);




                for write_image in write_images {
                    let physical_image = *physical_images.map_image_to_physical.get(&write_image.image).unwrap();
                    let sync_state = physical_image_sync_states.get_mut(&physical_image).unwrap();
                    sync_state.pending_write_access_flags |= write_image.access_flags;
                    sync_state.pending_write_pipeline_stage_flags |= write_image.pipeline_stage_flags;
                }
            }
        }

        fn create_attachment_description(
            //attachment_info: &RenderGraphPassColorAttachmentInfo,
            specification: &RenderGraphImageSpecification,
            attachment_type: RenderGraphPassAttachmentType,
            has_clear_color: bool,
            from_layout: dsc::ImageLayout,
            to_layout: dsc::ImageLayout,
        ) -> dsc::AttachmentDescription {
            let flags = dsc::AttachmentDescriptionFlags::None;
            // TODO: Look up if aliasing
            let format = specification.format;
            let samples = specification.samples;
            let load_op = match attachment_type {
                RenderGraphPassAttachmentType::Create => {
                    if has_clear_color {
                        dsc::AttachmentLoadOp::Clear
                    } else {
                        dsc::AttachmentLoadOp::DontCare
                    }
                }
                RenderGraphPassAttachmentType::Read => dsc::AttachmentLoadOp::Load,
                RenderGraphPassAttachmentType::Modify => dsc::AttachmentLoadOp::Load,
            };

            let store_op = match attachment_type {
                RenderGraphPassAttachmentType::Read => dsc::AttachmentStoreOp::DontCare,
                RenderGraphPassAttachmentType::Create | RenderGraphPassAttachmentType::Modify => {
                    if to_layout != dsc::ImageLayout::Undefined {
                        dsc::AttachmentStoreOp::Store
                    } else {
                        dsc::AttachmentStoreOp::DontCare
                    }
                }
            };

            let stencil_load_op = dsc::AttachmentLoadOp::DontCare;
            let stencil_store_op = dsc::AttachmentStoreOp::DontCare;
            //TODO: Can we set DONT_CARE on an output if the downstream readers are culled?
            let attachment = dsc::AttachmentDescription {
                flags: dsc::AttachmentDescriptionFlags::None,
                format: dsc::AttachmentFormat::Format(format.into()),
                samples: dsc::SampleCountFlags::from_vk_sample_count_flags(samples).unwrap(),
                load_op,
                store_op,
                stencil_load_op,
                stencil_store_op,
                initial_layout: from_layout,
                final_layout: to_layout,
                // pub flags: AttachmentDescriptionFlags,
                // pub format: AttachmentFormat,
                // pub samples: SampleCountFlags,
                // pub load_op: AttachmentLoadOp,
                // pub store_op: AttachmentStoreOp,
                // pub stencil_load_op: AttachmentLoadOp,
                // pub stencil_store_op: AttachmentStoreOp,
                // pub initial_layout: ImageLayout,
                // pub final_layout: ImageLayout,
            };
            attachment
        }
        */

}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct PhysicalImageId(usize);

struct PhysicalImageInfo {
    specification: RenderGraphImageSpecification,
}

#[derive(Default)]
struct PhysicalImageAllocator {
    unused_images: FnvHashMap<RenderGraphImageSpecification, Vec<PhysicalImageId>>,
    allocated_images: Vec<PhysicalImageInfo>,
}

impl PhysicalImageAllocator {
    fn allocate(
        &mut self,
        specification: &RenderGraphImageSpecification,
    ) -> PhysicalImageId {
        if let Some(image) = self
            .unused_images
            .entry(specification.clone())
            .or_default()
            .pop()
        {
            image
        } else {
            let id = PhysicalImageId(self.allocated_images.len());
            self.allocated_images.push(PhysicalImageInfo {
                specification: specification.clone(),
            });
            id
        }
    }
}
