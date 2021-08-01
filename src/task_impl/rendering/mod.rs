use std::{
    sync::{
        Arc,
    }
};

use vulkano::{
    command_buffer::DynamicState,
    format::Format,
    image::{
        ImageUsage,
        swapchain::SwapchainImage,
        view::ImageView,
    },
    pipeline::{
        viewport::Viewport,
    },
    render_pass::{
        Framebuffer,
        FramebufferAbstract,
        RenderPass,
    },
    swapchain::{
        Capabilities,
        ColorSpace,
        CompositeAlpha,
        FullscreenExclusive,
        PresentMode,
        SurfaceTransform,
        Swapchain,
        SwapchainBuilder,
        SwapchainCreationError,
    },
};

use winit::{
    window,
    window::Window,
};

pub fn create_swapchain(builder: SwapchainBuilder<Window>, caps: Capabilities, format: Format, dimensions: [u32;2], alpha: CompositeAlpha) -> Result<(Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>), SwapchainCreationError> {
    builder
        .num_images(caps.min_image_count)
        .format(format)
        .dimensions(dimensions)
        .layers(1)
        .usage(ImageUsage::color_attachment())
        .transform(SurfaceTransform::Identity)
        .composite_alpha(alpha)
        .present_mode(PresentMode::Fifo)
        .fullscreen_exclusive(FullscreenExclusive::Default)
        .clipped(true)
        .color_space(ColorSpace::SrgbNonLinear)
        .build()
}

pub fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<window::Window>>],
    render_pass: Arc<RenderPass>,
    dynamic_state: &mut DynamicState,
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions();

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0..1.0,
    };
    dynamic_state.viewports = Some(vec![viewport]);

    images
        .iter()
        .map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(ImageView::new(image.clone()).unwrap())
                    .unwrap()
                    .build()
                    .unwrap(),
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        })
        .collect::<Vec<_>>()
}