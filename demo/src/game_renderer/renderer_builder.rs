use crate::game_renderer::{RendererPlugin, GameRenderer};
use legion::Resources;
use rafx::api::{RafxApi, RafxResult, RafxQueueType};
use crate::daemon::AssetDaemonOpt;
use crate::daemon;
use crate::assets::gltf::MeshAssetType;
use crate::assets::font::FontAssetType;
use rafx::assets::distill_impl::AssetResource;
use crate::features::sprite::SpriteRenderFeature;
use crate::features::mesh::MeshRenderFeature;
use crate::phases::{OpaqueRenderPhase, ShadowMapRenderPhase, TransparentRenderPhase, PostProcessRenderPhase, UiRenderPhase};
use crate::features::imgui::ImGuiRenderFeature;
use rafx::assets::AssetManager;

pub enum AssetSource {
    Packfile(std::path::PathBuf),
    Daemon {
        external_daemon: bool,
        daemon_args: AssetDaemonOpt,
    },
}

pub struct RendererBuilderResult {
    pub asset_resource: AssetResource,
    pub asset_manager: AssetManager,
    pub renderer: GameRenderer
}

#[derive(Default)]
pub struct RendererBuilder {
    plugins: Vec<Box<dyn RendererPlugin>>,
}

impl RendererBuilder {
    pub fn add_plugin(mut self, plugin: Box<dyn RendererPlugin>) -> Self {
        self.plugins.push(plugin);
        self
    }

    pub fn build(
        self,
        resources: &Resources,
        rafx_api: &RafxApi,
        asset_source: AssetSource,
    ) -> RafxResult<RendererBuilderResult> {
        let mut asset_resource = match asset_source {
            AssetSource::Packfile(packfile) => {
                log::info!("Reading from packfile {:?}", packfile);

                // Initialize the packfile loader with the packfile path
                daemon::init_distill_packfile(&packfile)
            }
            AssetSource::Daemon {
                external_daemon,
                daemon_args
            } => {
                if !external_daemon {
                    log::info!("Hosting local daemon at {:?}", daemon_args.address);

                    let asset_daemon = rafx::assets::distill_impl::default_daemon();

                    for plugin in &self.plugins {
                        plugin.configure_asset_daemon();
                    }

                    let asset_daemon = asset_daemon
                        .with_importer("basis", rafx::assets::BasisImageImporter)
                        .with_importer("gltf", crate::assets::gltf::GltfImporter)
                        .with_importer("glb", crate::assets::gltf::GltfImporter)
                        .with_importer("ttf", crate::assets::font::FontImporter)
                        .with_db_path(daemon_args.db_dir)
                        .with_address(daemon_args.address)
                        .with_asset_dirs(daemon_args.asset_dirs);

                    // Spawn the daemon in a background thread.
                    std::thread::spawn(move || {
                        asset_daemon.run();
                    });
                } else {
                    log::info!("Connecting to daemon at {:?}", daemon_args.address);
                }

                // Connect to the daemon we just launched
                daemon::init_distill_daemon(daemon_args.address.to_string())
            }
        };

        let mut render_registry_builder = rafx::nodes::RenderRegistryBuilder::default();
        for plugin in &self.plugins {
            render_registry_builder = plugin.configure_render_registry(render_registry_builder);
        }

        render_registry_builder = render_registry_builder
            .register_feature::<SpriteRenderFeature>()
            .register_feature::<MeshRenderFeature>()
            .register_render_phase::<OpaqueRenderPhase>("Opaque")
            .register_render_phase::<ShadowMapRenderPhase>("ShadowMap")
            .register_render_phase::<TransparentRenderPhase>("Transparent")
            .register_render_phase::<PostProcessRenderPhase>("PostProcess")
            .register_render_phase::<UiRenderPhase>("Ui");

        #[cfg(feature = "use-imgui")]
        {
            render_registry_builder = render_registry_builder.register_feature::<ImGuiRenderFeature>();
        }

        let render_registry = render_registry_builder.build();

        let device_context = rafx_api.device_context();

        let graphics_queue = device_context.create_queue(RafxQueueType::Graphics)?;
        let transfer_queue = device_context.create_queue(RafxQueueType::Transfer)?;

        let mut asset_manager = rafx::assets::AssetManager::new(
            &device_context,
            &render_registry,
            rafx::assets::UploadQueueConfig {
                max_concurrent_uploads: 4,
                max_new_uploads_in_single_frame: 4,
                max_bytes_per_upload: 64 * 1024 * 1024,
            },
            &graphics_queue,
            &transfer_queue,
        );

        asset_manager.register_default_asset_types(&mut asset_resource);

        asset_manager.register_asset_type::<MeshAssetType>(&mut asset_resource);
        asset_manager.register_asset_type::<FontAssetType>(&mut asset_resource);

        let renderer = GameRenderer::new(
            resources,
            &mut asset_resource,
            &mut asset_manager,
            &graphics_queue,
            &transfer_queue,
            self.plugins
        )?;

        Ok(RendererBuilderResult {
            asset_resource,
            asset_manager,
            renderer
        })
    }
}