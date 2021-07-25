pub mod shaders;
pub mod vulkan_test;

use std::{
    sync::{
        Arc,
        RwLock,
    }
};

use crate::enum_none;

use crate::voxels::{
    renderer::{
        vulkan_test::{
            VulkanTest,
        },
    },
    voxel_world::{
        VoxelWorld,
    },
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
    event_loop::{
        ControlFlow,
        EventLoop,
    },
    window,
};

pub enum Renderer {
    None,
    VulkanTest(VulkanTest),
}

enum_none!(Renderer);

impl Renderer {
    pub fn any_start(world_lock:Arc<RwLock<VoxelWorld>>) -> bool {
        println!("Starting Rendering Thread");
        

        let mut event_loop = Some(EventLoop::new());
        let surface = {
            let world = world_lock.read().unwrap();
            let renderer_ref = world.renderer.clone();
            let mut renderer_guard = renderer_ref.lock().unwrap();
            let renderer_enum = &mut *renderer_guard;
            if let Renderer::None = renderer_enum {
                println!("No renderer, ignoring");
                return false;
            }
            let renderer = match renderer_enum {
                Renderer::VulkanTest(renderer) => {
                    renderer
                }
                Renderer::None => {
                    panic!("Reached impossible state.");
                }
            };
            let event_loop_took = event_loop.take().unwrap();
            let surface = renderer.init(&world.application_data.name.clone(), &world.application_data.version.clone(), &event_loop_took);
            event_loop = Some(event_loop_took);
            surface
        };

        let mut tick = 0;
        // let mut times = vec![];
        let mut last_time = std::time::SystemTime::now();
        let mut closing = false;
        println!("Starting event loop");
        event_loop.unwrap().run( move |event, _, control_flow| {
            let dur = std::time::SystemTime::now().duration_since(last_time).unwrap();
            tick+=1;
            if !closing {
                surface.window().set_title(format!("oui oui {} {}", tick, dur.as_nanos()).as_str());
            }
            match event {
                winit::event::Event::WindowEvent { event: winit::event::WindowEvent::CloseRequested|winit::event::WindowEvent::Destroyed, .. } => {
                    closing = true;
                    surface.window().set_title("oui oui (Closing)");
                    println!("Close requested");
                    *control_flow = ControlFlow::Exit;
                },
                _ => ()
            }
            let world = world_lock.read().unwrap();
            let renderer_ref = world.renderer.clone();
            let mut renderer_guard = renderer_ref.lock().unwrap();
            let renderer_enum = &mut *renderer_guard;
            if let Renderer::None = renderer_enum {
                println!("No renderer, ignoring");
                *control_flow = ControlFlow::Exit;
                return;
            }
            let renderer = match renderer_enum {
                Renderer::VulkanTest(renderer) => {
                    renderer
                }
                Renderer::None => {
                    panic!("Reached impossible state.");
                }
            };
            
            let render_tick_result = renderer.render().expect("Error in render body: ");
            if render_tick_result != 0 {
                println!("Renderer stopped with code: {}", render_tick_result);
                *control_flow = ControlFlow::Exit;
                return;
            }
            last_time = std::time::SystemTime::now();
        });
    }
}

fn window_size_dependent_setup(
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