use super::MeshCommandWriter;
use crate::features::mesh::{ExtractedFrameNodeMeshData, MeshPerObjectFragmentShaderParam, MeshPerViewFragmentShaderParam, MeshRenderFeature, ExtractedDirectionalLight, ExtractedPointLight, ExtractedSpotLight, PreparedSubmitNodeMeshData, ShadowMapData, LightId, ShadowMapRenderView};
use crate::phases::{OpaqueRenderPhase, ShadowMapRenderPhase};
use crate::render_contexts::{RenderJobPrepareContext, RenderJobWriteContext};
use fnv::{FnvHashMap, FnvHashSet};
use rafx::assets::assets::MaterialPass;
use rafx::nodes::{
    FeatureCommandWriter, FeatureSubmitNodes, FramePacket, PerViewNode, PrepareJob, RenderFeature,
    RenderFeatureIndex, RenderView, RenderViewIndex, ViewSubmitNodes,
};
use rafx::resources::{
    DescriptorSetAllocatorRef, DescriptorSetArc, DescriptorSetLayoutResource, ResourceArc,
};
use crate::components::{DirectionalLightComponent, PointLightComponent, PositionComponent, SpotLightComponent};

pub struct PreparedDirectionalLight<'a> {
    light: &'a DirectionalLightComponent,
    shadow_map_index: Option<usize>
}

pub struct PreparedPointLight<'a> {
    light: &'a PointLightComponent,
    position: &'a PositionComponent,
    shadow_map_index: Option<usize>
}

pub struct PreparedSpotLight<'a> {
    light: &'a SpotLightComponent,
    position: &'a PositionComponent,
    shadow_map_index: Option<usize>
}

pub struct MeshPrepareJob {
    pub(super) extracted_frame_node_mesh_data: Vec<Option<ExtractedFrameNodeMeshData>>,
    pub(super) directional_lights: Vec<ExtractedDirectionalLight>,
    pub(super) point_lights: Vec<ExtractedPointLight>,
    pub(super) spot_lights: Vec<ExtractedSpotLight>,
    pub(super) shadow_map_data: ShadowMapData,
}

impl PrepareJob<RenderJobPrepareContext, RenderJobWriteContext> for MeshPrepareJob {
    fn prepare(
        self: Box<Self>,
        prepare_context: &RenderJobPrepareContext,
        frame_packet: &FramePacket,
        views: &[&RenderView],
    ) -> (
        Box<dyn FeatureCommandWriter<RenderJobWriteContext>>,
        FeatureSubmitNodes,
    ) {
        profiling::scope!("Mesh Prepare");

        let mut descriptor_set_allocator = prepare_context
            .resource_context
            .create_descriptor_set_allocator();
        //let mut resource_allocator = prepare_context.resource_context.resources().

        //TODO: reserve sizes
        let mut opaque_per_view_descriptor_set_layouts =
            FnvHashSet::<ResourceArc<DescriptorSetLayoutResource>>::default();
        let mut prepared_submit_node_mesh_data = Vec::<PreparedSubmitNodeMeshData>::default();
        let mut per_view_descriptor_sets = FnvHashMap::<
            (RenderViewIndex, ResourceArc<DescriptorSetLayoutResource>),
            DescriptorSetArc,
        >::default();

        //
        // Iterate every mesh part to find all the per-view descriptor sets layouts
        //
        // Realistically all the materials (for now) have identical layouts so iterating though them
        // like this just avoids hardcoding to find the first mesh's first opaque pass
        //
        let mut submit_nodes = FeatureSubmitNodes::default();
        for mesh in &self.extracted_frame_node_mesh_data {
            if let Some(mesh) = mesh {
                for mesh_part in &*mesh.mesh_asset.inner.mesh_parts {
                    if let Some(mesh_part) = mesh_part {
                        opaque_per_view_descriptor_set_layouts.insert(
                            mesh_part.opaque_pass.descriptor_set_layouts
                                [super::PER_VIEW_DESCRIPTOR_SET_INDEX as usize]
                                .clone(),
                        );
                    }
                }
            }
        }

        //let mut shadow_maps = vec![];

        //
        // Assign all direction lights a shadow map slot
        //
        let mut prepared_directional_lights = Vec::with_capacity(self.directional_lights.len());
        for directional_light in &self.directional_lights {
            // let shadow_map_index = if let Some(x) = self.shadow_map_data.shadow_map_lookup.get(&LightId::DirectionalLight(directional_light.entity)) {
            //     let light_index = shadow_maps.len();
            //     shadow_maps.push((&self.shadow_map_data.shadow_map_image_views[*x], &self.shadow_map_data.shadow_map_render_views[*x]));
            //     Some(light_index)
            // } else {
            //     None
            // };

            prepared_directional_lights.push(PreparedDirectionalLight {
                light: &directional_light.light,
                shadow_map_index: self.shadow_map_data.shadow_map_lookup.get(&LightId::DirectionalLight(directional_light.entity)).copied()
            });
        }

        //
        // Assign all spot lights a shadow map slot
        //
        let mut prepared_spot_lights = Vec::with_capacity(self.spot_lights.len());
        for spot_light in &self.spot_lights {
            // let shadow_map_index = if let Some(x) = self.shadow_map_data.shadow_map_lookup.get(&LightId::SpotLight(spot_light.entity)) {
            //     let light_index = shadow_maps.len();
            //     shadow_maps.push((&self.shadow_map_data.shadow_map_image_views[*x], &self.shadow_map_data.shadow_map_render_views[*x]));
            //     Some(light_index)
            // } else {
            //     None
            // };

            prepared_spot_lights.push(PreparedSpotLight {
                light: &spot_light.light,
                position: &spot_light.position,
                shadow_map_index: self.shadow_map_data.shadow_map_lookup.get(&LightId::SpotLight(spot_light.entity)).copied()
            });
        }

        let mut prepared_point_lights = Vec::with_capacity(self.point_lights.len());
        for point_light in &self.point_lights {
            // let shadow_map_index = if let Some(x) = self.shadow_map_data.shadow_map_lookup.get(&LightId::PointLight(point_light.entity)) {
            //     let light_index = shadow_maps.len();
            //     shadow_maps.push((&self.shadow_map_data.shadow_map_image_views[*x], &self.shadow_map_data.shadow_map_render_views[*x]));
            //     Some(light_index)
            // } else {
            //     None
            // };

            prepared_point_lights.push(PreparedPointLight {
                light: &point_light.light,
                position: &point_light.position,
                shadow_map_index: self.shadow_map_data.shadow_map_lookup.get(&LightId::PointLight(point_light.entity)).copied()
            });
        }


        //
        // Create uniform data for each shadow map and a properly-sized static array of the images
        //
        //TODO: Pull this const from the shader
        const MAX_SHADOW_MAPS: usize = 48;
        let mut shadow_maps = [shaders::mesh_frag::ShadowMapDataStd140::default(); MAX_SHADOW_MAPS];
        let mut shadow_map_image_views = [None; MAX_SHADOW_MAPS];

        assert_eq!(self.shadow_map_data.shadow_map_render_views.len(), self.shadow_map_data.shadow_map_image_views.len());
        for (index, shadow_map_render_view) in &mut self
            .shadow_map_data
            .shadow_map_render_views
            .iter()
            .enumerate()
        {
            if index > shadow_map_image_views.len() {
                log::warn!("More shadow maps than the mesh shader can support");
                break;
            }

            match shadow_map_render_view {
                ShadowMapRenderView::Single(view) => {
                    shadow_maps[index] = shaders::mesh_frag::ShadowMapDataStd140 {
                        shadow_map_view_proj: view.view_proj().to_cols_array_2d(),
                        shadow_map_light_dir: view.view_dir().into(),
                        ..Default::default()
                    };
                },
                ShadowMapRenderView::Cube(views) => {
                    // Unused, don't initialize
                }
            }

            shadow_map_image_views[index] = Some(&self.shadow_map_data.shadow_map_image_views[index]);
        }

        // HACK: Placate vulkan validation for now
        if let Some(first) = self.shadow_map_data.shadow_map_image_views.first() {
            for index in self.shadow_map_data.shadow_map_image_views.len()..MAX_SHADOW_MAPS {
                shadow_map_image_views[index] = Some(first);
            }
        }

        //
        // Create per-view descriptors for all per-view descriptor layouts that are in our materials
        //
        for &view in views {
            let mut per_view_frag_data = self.create_per_view_frag_data(view, &prepared_directional_lights, &prepared_spot_lights, &prepared_point_lights);
            per_view_frag_data.shadow_map_count =
                self.shadow_map_data
                    .shadow_map_render_views
                    .len()
                    .min(per_view_frag_data.shadow_maps.len()) as u32;
            per_view_frag_data.shadow_maps = shadow_maps;

            if view.phase_is_relevant::<OpaqueRenderPhase>()
                || view.phase_is_relevant::<ShadowMapRenderPhase>()
            {
                for per_view_descriptor_set_layout in &opaque_per_view_descriptor_set_layouts {
                    let descriptor_set = descriptor_set_allocator
                        .create_descriptor_set(
                            &per_view_descriptor_set_layout,
                            shaders::mesh_frag::DescriptorSet0Args {
                                shadow_map_image_views: &shadow_map_image_views,
                                per_view_data: &per_view_frag_data,
                            },
                        )
                        .unwrap();

                    let old = per_view_descriptor_sets.insert(
                        (view.view_index(), per_view_descriptor_set_layout.clone()),
                        descriptor_set,
                    );
                    assert!(old.is_none());
                }
            }
        }

        //
        // Produce render nodes for every mesh
        //
        for view in views {
            let mut view_submit_nodes =
                ViewSubmitNodes::new(self.feature_index(), view.render_phase_mask());

            let view_nodes = frame_packet.view_nodes(view, self.feature_index());
            if let Some(view_nodes) = view_nodes {
                for view_node in view_nodes {
                    let extracted_data =
                        &self.extracted_frame_node_mesh_data[view_node.frame_node_index() as usize];
                    if let Some(extracted_data) = extracted_data {
                        let model_view = view.view_matrix() * extracted_data.world_transform;
                        let model_view_proj = view.projection_matrix() * model_view;

                        let per_object_param = MeshPerObjectFragmentShaderParam {
                            model: extracted_data.world_transform.to_cols_array_2d(),
                            model_view: model_view.to_cols_array_2d(),
                            model_view_proj: model_view_proj.to_cols_array_2d(),
                        };

                        for (mesh_part_index, mesh_part) in extracted_data
                            .mesh_asset
                            .inner
                            .mesh_parts
                            .iter()
                            .enumerate()
                        {
                            if let Some(mesh_part) = mesh_part {
                                //
                                // Write opaque render node, if it's relevant
                                //
                                if view.phase_is_relevant::<OpaqueRenderPhase>() {
                                    let submit_node_index = MeshPrepareJob::add_render_node(
                                        &mut descriptor_set_allocator,
                                        &mut prepared_submit_node_mesh_data,
                                        &per_view_descriptor_sets,
                                        &view,
                                        view_node,
                                        &per_object_param,
                                        mesh_part_index,
                                        &mesh_part.opaque_pass,
                                        Some(mesh_part.opaque_material_descriptor_set.clone()),
                                        false,
                                    );

                                    view_submit_nodes.add_submit_node::<OpaqueRenderPhase>(
                                        submit_node_index as u32,
                                        0,
                                        0.0,
                                    );
                                }

                                //
                                // Write shadow map render node, if it's relevant
                                //
                                if let Some(shadow_map_pass) = &mesh_part.shadow_map_pass {
                                    if view.phase_is_relevant::<ShadowMapRenderPhase>() {
                                        let submit_node_index = MeshPrepareJob::add_render_node(
                                            &mut descriptor_set_allocator,
                                            &mut prepared_submit_node_mesh_data,
                                            &per_view_descriptor_sets,
                                            &view,
                                            view_node,
                                            &per_object_param,
                                            mesh_part_index,
                                            shadow_map_pass,
                                            None,
                                            true,
                                        );

                                        view_submit_nodes.add_submit_node::<ShadowMapRenderPhase>(
                                            submit_node_index as u32,
                                            0,
                                            0.0,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }

            submit_nodes.add_submit_nodes_for_view(view, view_submit_nodes);
        }

        let writer = Box::new(MeshCommandWriter {
            extracted_frame_node_mesh_data: self.extracted_frame_node_mesh_data,
            prepared_submit_node_mesh_data,
        });

        (writer, submit_nodes)
    }

    fn feature_debug_name(&self) -> &'static str {
        MeshRenderFeature::feature_debug_name()
    }

    fn feature_index(&self) -> RenderFeatureIndex {
        MeshRenderFeature::feature_index()
    }
}

impl MeshPrepareJob {
    fn create_per_view_frag_data(
        &self,
        view: &RenderView,
        directional_lights: &[PreparedDirectionalLight],
        spot_lights: &[PreparedSpotLight],
        point_lights: &[PreparedPointLight],
    ) -> MeshPerViewFragmentShaderParam {
        let mut per_view_data = MeshPerViewFragmentShaderParam::default();

        per_view_data.ambient_light = glam::Vec4::new(0.03, 0.03, 0.03, 1.0).into();

        for light in directional_lights {
            let light_count = per_view_data.directional_light_count as usize;
            if light_count > per_view_data.directional_lights.len() {
                break;
            }

            let light_from = glam::Vec3::zero();
            let light_from_vs = (view.view_matrix() * light_from.extend(1.0)).truncate();
            let light_to = light.light.direction;
            let light_to_vs = (view.view_matrix() * light_to.extend(1.0)).truncate();

            let light_direction = (light_to - light_from).normalize();
            let light_direction_vs = (light_to_vs - light_from_vs).normalize();

            let out = &mut per_view_data.directional_lights[light_count];
            out.direction_ws = light_direction.into();
            out.direction_vs = light_direction_vs.into();
            out.color = light.light.color.into();
            out.intensity = light.light.intensity;
            out.shadow_map = light.shadow_map_index.map(|x| x as i32).unwrap_or(-1);

            per_view_data.directional_light_count += 1;
        }

        for light in point_lights {
            let light_count = per_view_data.point_light_count as usize;
            if light_count > per_view_data.point_lights.len() {
                break;
            }

            let out = &mut per_view_data.point_lights[light_count];
            out.position_ws = light.position.position.into();
            out.position_vs = (view.view_matrix() * light.position.position.extend(1.0))
                .truncate()
                .into();
            out.color = light.light.color.into();
            out.range = light.light.range;
            out.intensity = light.light.intensity;
            out.shadow_map = light.shadow_map_index.map(|x| x as i32).unwrap_or(-1);

            per_view_data.point_light_count += 1;
        }

        for light in spot_lights {
            let light_count = per_view_data.spot_light_count as usize;
            if light_count > per_view_data.spot_lights.len() {
                break;
            }

            let light_from = light.position.position;
            let light_from_vs = (view.view_matrix() * light_from.extend(1.0)).truncate();
            let light_to = light.position.position + light.light.direction;
            let light_to_vs = (view.view_matrix() * light_to.extend(1.0)).truncate();

            let light_direction = (light_to - light_from).normalize();
            let light_direction_vs = (light_to_vs - light_from_vs).normalize();

            let out = &mut per_view_data.spot_lights[light_count];
            out.position_ws = light_from.into();
            out.position_vs = light_from_vs.into();
            out.direction_ws = light_direction.into();
            out.direction_vs = light_direction_vs.into();
            out.spotlight_half_angle = light.light.spotlight_half_angle;
            out.color = light.light.color.into();
            out.range = light.light.range;
            out.intensity = light.light.intensity;
            out.shadow_map = light.shadow_map_index.map(|x| x as i32).unwrap_or(-1);

            per_view_data.spot_light_count += 1;
        }

        per_view_data
    }

    fn add_render_node(
        descriptor_set_allocator: &mut DescriptorSetAllocatorRef,
        prepared_submit_node_mesh_data: &mut Vec<PreparedSubmitNodeMeshData>,
        per_view_descriptor_sets: &FnvHashMap<
            (u32, ResourceArc<DescriptorSetLayoutResource>),
            DescriptorSetArc,
        >,
        view: &RenderView,
        view_node: &PerViewNode,
        per_object_param: &MeshPerObjectFragmentShaderParam,
        mesh_part_index: usize,
        material_pass: &MaterialPass,
        per_material_descriptor_set: Option<DescriptorSetArc>,
        is_shadow_pass: bool,
    ) -> usize {
        let per_view_descriptor_set = if !is_shadow_pass {
            let per_view_descriptor_set_layout = material_pass.descriptor_set_layouts
                [super::PER_VIEW_DESCRIPTOR_SET_INDEX as usize]
                .clone();

            Some(
                per_view_descriptor_sets[&(view.view_index(), per_view_descriptor_set_layout)]
                    .clone(),
            )
        } else {
            None
        };

        //
        // Create the per-instance descriptor set
        // TODO: Common case is that parts in the same mesh use same material, so only create new descriptor set if the material is
        // different between parts.
        //
        let per_instance_descriptor_set_layout = &material_pass.descriptor_set_layouts
            [super::PER_INSTANCE_DESCRIPTOR_SET_INDEX as usize];

        let per_instance_descriptor_set = descriptor_set_allocator
            .create_descriptor_set(
                per_instance_descriptor_set_layout,
                shaders::mesh_frag::DescriptorSet2Args {
                    per_object_data: &per_object_param,
                },
            )
            .unwrap();
        //
        // Create the submit node
        //
        let submit_node_index = prepared_submit_node_mesh_data.len();
        prepared_submit_node_mesh_data.push(PreparedSubmitNodeMeshData {
            material_pass: material_pass.clone(),
            per_view_descriptor_set,
            per_material_descriptor_set: per_material_descriptor_set.clone(),
            per_instance_descriptor_set,
            frame_node_index: view_node.frame_node_index(),
            mesh_part_index,
        });
        submit_node_index
    }
}
