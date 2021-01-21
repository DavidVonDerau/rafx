use crate::metal::{RafxDeviceContextMetal, RafxRenderTargetMetal, RafxRawImageMetal};
use crate::{RafxSwapchainDef, RafxFormat, RafxResult, RafxSwapchainImage, RafxRenderTargetDef, RafxExtents3D, RafxResourceType, RafxSampleCount, RafxTextureDimensions, RafxRenderTarget};
use raw_window_handle::HasRawWindowHandle;
use std::sync::atomic::{Ordering, AtomicPtr};

const SWAPCHAIN_IMAGE_COUNT : u32 = 3;

pub struct RafxSwapchainMetal {
    device_context: RafxDeviceContextMetal,
    layer: metal_rs::MetalLayer,
    drawable: AtomicPtr<metal_rs::CAMetalDrawable>,
    swapchain_def: RafxSwapchainDef,
    format: RafxFormat,
    // Just fake this
    next_swapchain_image_index: u32,
}

impl Drop for RafxSwapchainMetal {
    fn drop(&mut self) {
        self.swap_drawable(None);
    }
}

impl RafxSwapchainMetal {
    pub fn swapchain_def(&self) -> &RafxSwapchainDef {
        &self.swapchain_def
    }

    pub fn image_count(&self) -> usize {
        SWAPCHAIN_IMAGE_COUNT as usize
    }

    pub fn format(&self) -> RafxFormat {
        self.format
    }

    pub fn metal_layer(&self) -> &metal_rs::MetalLayerRef {
        self.layer.as_ref()
    }

    pub fn new(
        device_context: &RafxDeviceContextMetal,
        raw_window_handle: &dyn HasRawWindowHandle,
        swapchain_def: &RafxSwapchainDef,
    ) -> RafxResult<RafxSwapchainMetal> {
        let layer = match raw_window_handle.raw_window_handle() {
            #[cfg(target_os = "macos")]
            raw_window_handle::RawWindowHandle::MacOS(handle) => unsafe {
                raw_window_metal::macos::metal_layer_from_handle(handle)
            },
            #[cfg(target_os = "ios")]
            raw_window_handle::RawWindowHandle::IOS(handle) => unsafe {
                raw_window_metal::ios::metal_layer_from_handle(handle)
            },
            _ => return Err("Cannot create RafxSurfaceMetal on this operating system".into()),
        };

        let layer = match layer {
            raw_window_metal::Layer::Allocated(x) => Some(x),
            raw_window_metal::Layer::Existing(x) => Some(x),
            raw_window_metal::Layer::None => None,
        }.unwrap();

        let layer =
            unsafe { std::mem::transmute::<_, &metal_rs::MetalLayerRef>(layer).to_owned() };

        layer.set_device(device_context.device());
        //TODO: Don't hardcode pixel format
        // https://developer.apple.com/documentation/quartzcore/cametallayer/1478155-pixelformat
        layer.set_pixel_format(metal_rs::MTLPixelFormat::BGRA8Unorm);
        layer.set_presents_with_transaction(false);
        layer.set_display_sync_enabled(swapchain_def.enable_vsync);

        //TODO: disable timeout on acquire drawable?
        layer.set_drawable_size(metal_rs::CGSize::new(swapchain_def.width as f64, swapchain_def.height as f64));

        let swapchain_def = swapchain_def.clone();

        Ok(RafxSwapchainMetal {
            device_context: device_context.clone(),
            layer,
            drawable: Default::default(),
            swapchain_def,
            next_swapchain_image_index: 0,
            format: RafxFormat::B8G8R8A8_UNORM
        })
    }

    pub fn rebuild(
        &mut self,
        swapchain_def: &RafxSwapchainDef,
    ) -> RafxResult<()> {
        self.layer.set_drawable_size(metal_rs::CGSize::new(swapchain_def.width as f64, swapchain_def.height as f64));
        //TODO: Add to metal crate, following presents_with_transaction as an example
        //self.layer.set_display_sync_enabled(swapchain_def.enable_vsync);

        self.swapchain_def = swapchain_def.clone();
        Ok(())
    }

    pub fn acquire_next_image(
        &mut self,
    ) -> RafxResult<RafxSwapchainImage> {
        let drawable = self
            .layer
            .next_drawable()
            .ok_or("Timed out while trying to acquire drawable".to_string())?;

        let old = self.swap_drawable(Some(drawable.to_owned()));
        assert!(old.is_none());

        let raw_image = RafxRawImageMetal::Ref(drawable.texture().to_owned());

        // This ends up being cheap because it doesn't allocate anything. We could cache it but it doesn't
        // seem worthwhile
        let render_target = RafxRenderTargetMetal::from_existing(&self.device_context, Some(raw_image), &RafxRenderTargetDef {
            extents: RafxExtents3D {
                width: self.swapchain_def.width,
                height: self.swapchain_def.height,
                depth: 1
            },
            array_length: 1,
            mip_count: 1,
            format: self.format,
            resource_type: RafxResourceType::UNDEFINED,
            sample_count: RafxSampleCount::SampleCount1,
            dimensions: RafxTextureDimensions::Dim2D,
        })?;

        let swapchain_image_index = self.next_swapchain_image_index;
        self.next_swapchain_image_index += 1;
        if self.next_swapchain_image_index >= SWAPCHAIN_IMAGE_COUNT {
            self.next_swapchain_image_index = 0;
        }

        Ok(RafxSwapchainImage {
            render_target: RafxRenderTarget::Metal(render_target),
            swapchain_image_index
        })
    }

    pub(crate) fn swap_drawable(&self, drawable: Option<metal_rs::MetalDrawable>) -> Option<metal_rs::MetalDrawable> {
        use foreign_types_shared::ForeignType;
        use foreign_types_shared::ForeignTypeRef;

        let ptr = if let Some(drawable) = drawable {
            drawable.as_ptr()
        } else {
            std::ptr::null_mut() as _
        };

        let ptr = self.drawable.swap(ptr, Ordering::Relaxed);
        if !ptr.is_null() {
            unsafe {
                Some(metal_rs::MetalDrawable::from_ptr(ptr))
            }
        } else {
            None
        }
    }
}