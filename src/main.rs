
use renderer_shell_vulkan::{RendererBuilder, LogicalSize, RendererEventListener, Window, VkDevice, VkSwapchain, Renderer, CreateRendererError, VkDeviceContext, VkTransferUpload, VkTransferUploadState, VkImage};
use renderer_shell_vulkan_sdl2::Sdl2Window;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use ash::prelude::VkResult;
use renderer_ext::imgui_support::{VkImGuiRenderPassFontAtlas};
use imgui::sys::ImGuiStorage_GetBoolRef;
use sdl2::mouse::MouseState;
use renderer_ext::GameRendererWithShell;
use image::{GenericImageView, load};
use atelier_assets::loader as atelier_loader;


use atelier_assets::core::asset_uuid;
use atelier_assets::core as atelier_core;

mod daemon;
use renderer_ext::asset_resource::AssetResource;
use renderer_ext::image_importer::ImageAsset;
use renderer_ext::image_utils::{DecodedTexture, enqueue_load_images};
use imgui::{Key, Image};
use renderer_ext::asset_storage::{StorageUploader, ResourceHandle};
use std::mem::ManuallyDrop;
//use renderer_ext::renderpass::sprite::LoadingSprite;
use std::sync::mpsc::{Sender, Receiver};
use std::time::Duration;
use atelier_loader::AssetLoadOp;
use std::error::Error;
use renderer_ext::renderpass::sprite::{VkSpriteResourceManager, SpriteUpdate, ImageUploadQueue, ImageUploader};

fn main() {
    // Setup logging
    env_logger::Builder::from_default_env()
        //.filter_level(log::LevelFilter::Error)
        .filter_level(log::LevelFilter::Debug)
        .init();

    // Spawn the daemon in a background thread. This could be a different process, but
    // for simplicity we'll launch it here.
    std::thread::spawn(move || {
        daemon::run();
    });

    let mut time = renderer_ext::time::TimeState::new();
    time.update();

    // Setup SDL
    let sdl_context = sdl2::init().expect("Failed to initialize sdl2");
    let video_subsystem = sdl_context
        .video()
        .expect("Failed to create sdl video subsystem");

    // Set up the coordinate system to be fixed at 900x600, and use this as the default window size
    // This means the drawing code can be written as though the window is always 900x600. The
    // output will be automatically scaled so that it's always visible.
    let logical_size = LogicalSize {
        width: 900,
        height: 600,
    };

    let sdl_window = video_subsystem
        .window("Renderer Prototype", logical_size.width, logical_size.height)
        .position_centered()
        .allow_highdpi()
        .resizable()
        .vulkan()
        .build()
        .expect("Failed to create window");
    log::info!("window created");

    let imgui_manager = renderer_ext::imgui_support::init_imgui_manager(&sdl_window);

    let window = Sdl2Window::new(&sdl_window);
    let renderer = GameRendererWithShell::new(&window, imgui_manager.build_font_atlas(), &time);

    // Check if there were error setting up vulkan
    if let Err(e) = renderer {
        log::error!("Error during renderer construction: {:?}", e);
        return;
    }

    log::info!("renderer created");

    let mut renderer = renderer.unwrap();

    // Increment a frame count so we can render something that moves
    let mut frame_count = 0;

    log::info!("Starting window event loop");
    let mut event_pump = sdl_context
        .event_pump()
        .expect("Could not create sdl event pump");

    // Handles routing data between the asset system and sprite resource manager
    let mut image_upload_queue = ImageUploadQueue::new(
        renderer.shell().device(),
        renderer.sprite_resource_manager().unwrap().sprite_update_tx().clone()
    );

    // Force an image to load and stay resident in memory
    let (mut asset_resource, image_handle) = {
        let device = renderer.shell().device();
        let device_context = &device.context;

        let mut asset_resource = AssetResource::default();
        asset_resource.add_storage_with_uploader::<ImageAsset, ImageUploader>(Box::new(ImageUploader::new(
            device_context.clone(),
            image_upload_queue.tx().clone()
        )));

        let asset_uuid = asset_uuid!("d60aa147-e1c7-42dc-9e99-40ba882544a7");

        use atelier_assets::loader::Loader;
        use atelier_assets::loader::handle::AssetHandle;

        let load_handle = asset_resource.loader().add_ref(asset_uuid);
        let image_handle = atelier_assets::loader::handle::Handle::<ImageAsset>::new(
            asset_resource.tx().clone(),
            load_handle,
        );
        (asset_resource, image_handle)
    };






    let mut print_time_event = renderer_ext::time::PeriodicEvent::default();

    'running: loop {
        for event in event_pump.poll_iter() {
            imgui_manager.handle_event(&event);
            if !imgui_manager.ignore_event(&event) {
                log::trace!("{:?}", event);
                match event {
                    //
                    // Halt if the user requests to close the window
                    //
                    Event::Quit { .. } => break 'running,

                    //
                    // Close if the escape key is hit
                    //
                    Event::KeyDown {
                        keycode: Some(keycode),
                        keymod: modifiers,
                        ..
                    } => {
                        log::trace!("Key Down {:?} {:?}", keycode, modifiers);
                        if keycode == Keycode::Escape {
                            break 'running;
                        }

                        if keycode == Keycode::D {
                            renderer.dump_stats();
                        }
                    }

                    _ => {}
                }
            }
        }

        let window = Sdl2Window::new(&sdl_window);
        imgui_manager.begin_frame(&sdl_window, &MouseState::new(&event_pump));

        asset_resource.update();
        image_upload_queue.update(renderer.shell().device());

        imgui_manager.with_ui(|ui| {
            let mut opened = true;
            ui.show_demo_window(&mut opened);
        });

        imgui_manager.render(&sdl_window);

        //
        // Redraw
        //
        renderer.draw(&window, &time).unwrap();
        time.update();

        if print_time_event.try_take_event(time.current_instant(), std::time::Duration::from_secs_f32(1.0)) {
            println!("FPS: {}", time.updates_per_second());
            //renderer.dump_stats();
        }
    }
}
