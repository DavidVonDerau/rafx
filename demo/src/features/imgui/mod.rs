use crate::features::imgui::extract::ImGuiExtractJobImpl;
use crate::imgui_support::ImGuiDrawData;
use crate::render_contexts::{
    RenderJobExtractContext, RenderJobPrepareContext, RenderJobWriteContext,
};
use rafx::api::{RafxPrimitiveTopology};
use rafx::nodes::ExtractJob;
use rafx::nodes::RenderFeature;
use rafx::nodes::RenderFeatureIndex;
use rafx::resources::{VertexDataLayout, VertexDataSetLayout};
use std::convert::TryInto;

mod extract;
mod prepare;
mod write;

pub fn create_imgui_extract_job() -> Box<dyn ExtractJob<RenderJobExtractContext, RenderJobPrepareContext, RenderJobWriteContext>> {
    Box::new(ImGuiExtractJobImpl::new())
}

/// Per-pass "global" data
pub type ImGuiUniformBufferObject = shaders::imgui_vert::ArgsUniform;

lazy_static::lazy_static! {
    pub static ref IMGUI_VERTEX_LAYOUT : VertexDataSetLayout = {
        use rafx::api::RafxFormat;

        let vertex = imgui::DrawVert {
            pos: Default::default(),
            col: Default::default(),
            uv: Default::default()
        };

        VertexDataLayout::build_vertex_layout(&vertex, |builder, vertex| {
            builder.add_member(&vertex.pos, "POSITION", RafxFormat::R32G32_SFLOAT);
            builder.add_member(&vertex.uv, "TEXCOORD", RafxFormat::R32G32_SFLOAT);
            builder.add_member(&vertex.col, "COLOR", RafxFormat::R8G8B8A8_UNORM);
        }).into_set(RafxPrimitiveTopology::TriangleList)
    };
}

rafx::declare_render_feature!(ImGuiRenderFeature, DEBUG_3D_FEATURE_INDEX);

pub(self) struct ExtractedImGuiData {
    imgui_draw_data: Option<ImGuiDrawData>,
}

#[derive(Debug)]
struct ImGuiDrawCall {
    first_element: u32,
    count: u32,
}
