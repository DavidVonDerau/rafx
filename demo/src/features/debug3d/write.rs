rafx::declare_render_feature_write_job!();

use rafx::api::RafxPrimitiveTopology;
use rafx::api::RafxVertexBufferBinding;
use rafx::framework::{VertexDataLayout, VertexDataSetLayout};

/// Vertex format for vertices sent to the GPU
#[derive(Clone, Debug, Copy, Default)]
#[repr(C)]
pub struct Debug3DVertex {
    pub pos: [f32; 3],
    pub color: [f32; 4],
}

lazy_static::lazy_static! {
    pub static ref DEBUG_VERTEX_LAYOUT : VertexDataSetLayout = {
        use rafx::api::RafxFormat;

        VertexDataLayout::build_vertex_layout(&Debug3DVertex::default(), |builder, vertex| {
            builder.add_member(&vertex.pos, "POSITION", RafxFormat::R32G32B32_SFLOAT);
            builder.add_member(&vertex.color, "COLOR", RafxFormat::R32G32B32A32_SFLOAT);
        }).into_set(RafxPrimitiveTopology::LineStrip)
    };
}

use rafx::framework::{BufferResource, DescriptorSetArc, MaterialPassResource, ResourceArc};
use rafx::nodes::RenderViewIndex;

#[derive(Debug)]
pub struct Debug3DDrawCall {
    first_element: u32,
    count: u32,
}

pub struct FeatureCommandWriterImpl {
    vertex_buffer: Option<ResourceArc<BufferResource>>,
    draw_calls: Vec<Debug3DDrawCall>,
    debug3d_material_pass: ResourceArc<MaterialPassResource>,
    per_view_descriptor_sets: Vec<Option<DescriptorSetArc>>,
}

impl FeatureCommandWriterImpl {
    pub fn new(
        debug3d_material_pass: ResourceArc<MaterialPassResource>,
        num_line_lists: usize,
    ) -> Self {
        FeatureCommandWriterImpl {
            vertex_buffer: Default::default(),
            draw_calls: Vec::with_capacity(num_line_lists),
            debug3d_material_pass,
            per_view_descriptor_sets: Default::default(),
        }
    }

    pub fn push_per_view_descriptor_set(
        &mut self,
        view_index: RenderViewIndex,
        per_view_descriptor_set: DescriptorSetArc,
    ) {
        // Grow the array if necessary

        self.per_view_descriptor_sets.resize(
            self.per_view_descriptor_sets
                .len()
                .max(view_index as usize + 1),
            None,
        );

        self.per_view_descriptor_sets[view_index as usize] = Some(per_view_descriptor_set);
    }

    pub fn push_draw_call(
        &mut self,
        first_element: u32,
        count: usize,
    ) {
        self.draw_calls.push(Debug3DDrawCall {
            first_element,
            count: count as u32,
        });
    }

    pub fn set_vertex_buffer(
        &mut self,
        vertex_buffer: Option<ResourceArc<BufferResource>>,
    ) {
        self.vertex_buffer = vertex_buffer;
    }

    pub fn draw_calls(&self) -> &Vec<Debug3DDrawCall> {
        &self.draw_calls
    }
}

impl FeatureCommandWriter for FeatureCommandWriterImpl {
    fn apply_setup(
        &self,
        write_context: &mut RenderJobWriteContext,
        view: &RenderView,
        render_phase_index: RenderPhaseIndex,
    ) -> RafxResult<()> {
        profiling::scope!(apply_setup_scope);

        if let Some(vertex_buffer) = self.vertex_buffer.as_ref() {
            let pipeline = write_context
                .resource_context
                .graphics_pipeline_cache()
                .get_or_create_graphics_pipeline(
                    render_phase_index,
                    &self.debug3d_material_pass,
                    &write_context.render_target_meta,
                    &*DEBUG_VERTEX_LAYOUT,
                )?;

            let command_buffer = &write_context.command_buffer;
            command_buffer.cmd_bind_pipeline(&*pipeline.get_raw().pipeline)?;

            self.per_view_descriptor_sets[view.view_index() as usize]
                .as_ref()
                .unwrap()
                .bind(command_buffer)?;

            command_buffer.cmd_bind_vertex_buffers(
                0,
                &[RafxVertexBufferBinding {
                    buffer: &*vertex_buffer.get_raw().buffer,
                    byte_offset: 0,
                }],
            )?;
        }
        Ok(())
    }

    fn render_element(
        &self,
        write_context: &mut RenderJobWriteContext,
        _view: &RenderView,
        _render_phase_index: RenderPhaseIndex,
        index: SubmitNodeId,
    ) -> RafxResult<()> {
        profiling::scope!(render_element_scope);

        // The prepare phase emits a single node which will draw everything. In the future it might
        // emit a node per draw call that uses transparency
        if index == 0 {
            let command_buffer = &write_context.command_buffer;

            for draw_call in &self.draw_calls {
                command_buffer.cmd_draw(draw_call.count as u32, draw_call.first_element as u32)?;
            }
        }

        Ok(())
    }

    fn feature_debug_name(&self) -> &'static str {
        render_feature_debug_name()
    }

    fn feature_index(&self) -> RenderFeatureIndex {
        render_feature_index()
    }
}
