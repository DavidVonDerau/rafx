use crate::features::debug3d::plugin::Debug3DStaticResources;
use crate::features::debug3d::prepare::Debug3dPrepareJobImpl;
use crate::features::debug3d::{Debug3dRenderFeature, DebugDraw3DResource, ExtractedDebug3dData};
use rafx::assets::AssetManagerRenderResource;
use rafx::nodes::{
    ExtractJob, FramePacket, PrepareJob, RenderFeature, RenderFeatureIndex,
    RenderJobExtractContext, RenderView,
};

pub struct Debug3dExtractJob {}

impl Debug3dExtractJob {
    pub fn new() -> Self {
        Self {}
    }
}

impl ExtractJob for Debug3dExtractJob {
    fn extract(
        self: Box<Self>,
        extract_context: &RenderJobExtractContext,
        _frame_packet: &FramePacket,
        _views: &[RenderView],
    ) -> Box<dyn PrepareJob> {
        profiling::scope!("Debug3d Extract");
        let asset_manager = extract_context
            .render_resources
            .fetch::<AssetManagerRenderResource>();

        let line_lists = extract_context
            .extract_resources
            .fetch_mut::<DebugDraw3DResource>()
            .take_line_lists();

        let debug3d_material = &extract_context
            .render_resources
            .fetch::<Debug3DStaticResources>()
            .debug3d_material;

        let debug3d_material_pass = asset_manager
            .committed_asset(&debug3d_material)
            .unwrap()
            .get_single_material_pass()
            .unwrap();

        Box::new(Debug3dPrepareJobImpl::new(
            debug3d_material_pass,
            ExtractedDebug3dData { line_lists },
        ))
    }

    fn feature_debug_name(&self) -> &'static str {
        Debug3dRenderFeature::feature_debug_name()
    }

    fn feature_index(&self) -> RenderFeatureIndex {
        Debug3dRenderFeature::feature_index()
    }
}
