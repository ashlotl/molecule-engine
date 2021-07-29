use std::{
    sync::{
        Arc,
    }
};

use vulkano::{
    command_buffer::DynamicState,
    image::{
        SwapchainImage,
        view::ImageView,
    },
    pipeline::viewport::Viewport,
    render_pass::{
        Framebuffer,
        FramebufferAbstract,
        RenderPass,
    },
};

use winit::{
    window,
};

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