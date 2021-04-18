use rafx::nodes::RenderPhase;
use rafx::nodes::{RenderPhaseIndex, SubmitNode};
use std::cmp::Ordering;

rafx::declare_render_phase!(
    DepthPrepassRenderPhase,
    DEPTH_PREPASS_RENDER_PHASE_INDEX,
    depth_prepass_render_phase_sort_submit_nodes
);

#[profiling::function]
fn depth_prepass_render_phase_sort_submit_nodes(
    mut submit_nodes: Vec<SubmitNode>
) -> Vec<SubmitNode> {
    log::trace!(
        "Sort phase {}",
        DepthPrepassRenderPhase::render_phase_debug_name()
    );

    submit_nodes.sort_unstable_by(|a, b| {
        let ordering = a.sort_key().partial_cmp(&b.sort_key()).unwrap();
        if ordering == Ordering::Equal {
            a.distance().partial_cmp(&b.distance()).unwrap()
        } else {
            ordering
        }
    });

    submit_nodes
}
