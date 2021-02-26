use crate::{
    FeatureCommandWriter, FeatureSubmitNodes, FramePacket, MergedFrameSubmitNodes,
    PreparedRenderData, RenderFeatureIndex, RenderRegistry, RenderView,
};

pub trait PrepareJob<PrepareContextT, WriteContextT>: Send {
    fn prepare(
        self: Box<Self>,
        prepare_context: &PrepareContextT,
        frame_packet: &FramePacket,
        views: &[&RenderView],
    ) -> (
        Box<dyn FeatureCommandWriter<WriteContextT>>,
        FeatureSubmitNodes,
    );

    fn feature_debug_name(&self) -> &'static str;
    fn feature_index(&self) -> RenderFeatureIndex;
}

pub struct PrepareJobSet<PrepareContextT, WriteContextT> {
    prepare_jobs: Vec<Box<dyn PrepareJob<PrepareContextT, WriteContextT>>>,
}

impl<PrepareContextT, WriteContextT> PrepareJobSet<PrepareContextT, WriteContextT> {
    pub fn new(prepare_jobs: Vec<Box<dyn PrepareJob<PrepareContextT, WriteContextT>>>) -> Self {
        PrepareJobSet { prepare_jobs }
    }

    pub fn prepare(
        self,
        prepare_context: &PrepareContextT,
        frame_packet: &FramePacket,
        views: &[&RenderView],
        registry: &RenderRegistry,
    ) -> Box<PreparedRenderData<WriteContextT>> {
        let mut feature_command_writers = Vec::with_capacity(self.prepare_jobs.len());
        let mut all_submit_nodes = Vec::with_capacity(self.prepare_jobs.len());

        //TODO: Kick these to happen in parallel
        for prepare_job in self.prepare_jobs {
            let (writer, submit_nodes) = prepare_job.prepare(prepare_context, frame_packet, views);

            feature_command_writers.push(writer);
            all_submit_nodes.push(submit_nodes);
        }

        // Merge all submit nodes
        let merged_submit_nodes = MergedFrameSubmitNodes::new(all_submit_nodes, registry);

        Box::new(PreparedRenderData::new(
            feature_command_writers,
            merged_submit_nodes,
        ))
    }
}
