use crate::features::sprite::{SpriteDrawCall, SpriteRenderFeature};
use rafx::api::{RafxIndexBufferBinding, RafxIndexType, RafxResult, RafxVertexBufferBinding};
use rafx::framework::{BufferResource, DescriptorSetArc, MaterialPassResource, ResourceArc};
use rafx::nodes::{
    FeatureCommandWriter, RenderFeature, RenderFeatureIndex, RenderJobWriteContext,
    RenderPhaseIndex, RenderView, SubmitNodeId,
};

pub struct SpriteCommandWriter {
    pub vertex_buffers: Vec<ResourceArc<BufferResource>>,
    pub index_buffers: Vec<ResourceArc<BufferResource>>,
    pub draw_calls: Vec<SpriteDrawCall>,
    pub per_view_descriptor_sets: Vec<Option<DescriptorSetArc>>,
    pub sprite_material: ResourceArc<MaterialPassResource>,
}

impl FeatureCommandWriter for SpriteCommandWriter {
    fn apply_setup(
        &self,
        write_context: &mut RenderJobWriteContext,
        view: &RenderView,
        render_phase_index: RenderPhaseIndex,
    ) -> RafxResult<()> {
        let command_buffer = &write_context.command_buffer;

        let pipeline = write_context
            .resource_context
            .graphics_pipeline_cache()
            .get_or_create_graphics_pipeline(
                render_phase_index,
                &self.sprite_material,
                &write_context.render_target_meta,
                &super::SPRITE_VERTEX_LAYOUT,
            )
            .unwrap();

        command_buffer.cmd_bind_pipeline(&pipeline.get_raw().pipeline)?;

        // Bind per-pass data (UBO with view/proj matrix, sampler)
        self.per_view_descriptor_sets[view.view_index() as usize]
            .as_ref()
            .unwrap()
            .bind(command_buffer)?;

        command_buffer.cmd_bind_vertex_buffers(
            0,
            &[RafxVertexBufferBinding {
                buffer: &self.vertex_buffers[0].get_raw().buffer,
                byte_offset: 0,
            }],
        )?;

        command_buffer.cmd_bind_index_buffer(&RafxIndexBufferBinding {
            buffer: &self.index_buffers[0].get_raw().buffer,
            byte_offset: 0,
            index_type: RafxIndexType::Uint16,
        })?;

        Ok(())
    }

    fn render_element(
        &self,
        write_context: &mut RenderJobWriteContext,
        _view: &RenderView,
        _render_phase_index: RenderPhaseIndex,
        index: SubmitNodeId,
    ) -> RafxResult<()> {
        let command_buffer = &write_context.command_buffer;
        let draw_call = &self.draw_calls[index as usize];

        // Bind per-draw-call data (i.e. texture)
        draw_call.texture_descriptor_set.bind(command_buffer)?;

        command_buffer.cmd_draw_indexed(
            draw_call.index_buffer_count as u32,
            draw_call.index_buffer_first_element as u32,
            0,
        )?;

        Ok(())
    }

    fn feature_debug_name(&self) -> &'static str {
        SpriteRenderFeature::feature_debug_name()
    }

    fn feature_index(&self) -> RenderFeatureIndex {
        SpriteRenderFeature::feature_index()
    }
}
