use std::{
    borrow::Cow,
    sync::{
        Arc,
    },
    thread,
};

use vulkano::{
    buffer::{
        BufferUsage,
        CpuAccessibleBuffer,
        cpu_pool::CpuBufferPool,
    },
    command_buffer::{
        AutoCommandBufferBuilder,
        CommandBufferUsage,
        DynamicState,
        PrimaryCommandBuffer,
        SubpassContents,
    },
    descriptor::{
        descriptor_set::{
            PersistentDescriptorSet,
            PersistentDescriptorSetImg,
        },
    },
    device::{
        Device,
        DeviceExtensions,
        Features,
        Queue,
    },
    format::{
        ClearValue,
        Format,
    },
    image::{
        ImageDimensions,
        ImageUsage,
        StorageImage,
        view::ImageView,
    },
    instance::{
        ApplicationInfo,
        Instance,
        PhysicalDevice,
        Version as VulkanVersion,
    },
    pipeline::{
        ComputePipeline,
        ComputePipelineAbstract,
        GraphicsPipeline,
        GraphicsPipelineAbstract,
        vertex::SingleBufferDefinition,
    },
    render_pass::{
        FramebufferAbstract,
        Subpass,
    },
    swapchain,
    swapchain::{
        ColorSpace,
        FullscreenExclusive,
        PresentMode,
        SurfaceTransform,
        Swapchain,
    },
    sync,
    sync::{
        FlushError,
        GpuFuture,
    },
};

use vulkano_win::VkSurfaceBuild;

use winit::{
    event_loop::EventLoop,
    window,
    window::WindowBuilder,
};

use crate::{
    ENGINE_VERSION,
    metadata::versions::{
        MoleculeApplicationVersion,
    },
    voxels::{
        renderer,
        renderer::{
            shaders,
        },
    },
};

pub struct VulkanState {
    instance:Option<Arc<Instance>>,
    cpu_buffers_u8_slice: Vec<Option<Arc<CpuAccessibleBuffer<[u8]>>>>,
    cpu_buffers_vertex_slice: Vec<Option<Arc<CpuAccessibleBuffer<[Vertex]>>>>,
    compute_pipelines:Vec<Option<Arc<ComputePipeline>>>,
    dynamic_state: Option<DynamicState>,
    framebuffers: Option<Vec<Arc<dyn FramebufferAbstract + Send + Sync>>>,
    graphics_pipelines: Vec<Option<Arc<GraphicsPipeline<SingleBufferDefinition<Vertex>>>>>,
    graphics_set: Option<std::sync::Arc<vulkano::descriptor::descriptor_set::PersistentDescriptorSet<((), vulkano::descriptor::descriptor_set::PersistentDescriptorSetBuf<vulkano::buffer::cpu_pool::CpuBufferPoolSubbuffer<shaders::edges_shader::ty::Data, std::sync::Arc<vulkano::memory::pool::StdMemoryPool>>>)>>>,
    queue:Option<Arc<Queue>>,
    storage_image_persistent_descriptor_sets: Vec<Option<Arc<PersistentDescriptorSet<((), PersistentDescriptorSetImg<Arc<ImageView<Arc<StorageImage>>>>)>>>>,
    storage_images:Vec<Option<Arc<StorageImage>>>,
    swapchain: Option<Arc<Swapchain<window::Window>>>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
}

unsafe impl Send for VulkanState {}

pub struct VulkanTest {
    pub time:u32,

    pub state_ignore: Option<VulkanState>, //as the name suggests, don't worry about this parameter if you're just using the api--Set it to None
    pub configs: Configs,
    pub shader_path: String,
}

pub struct Configs {
    pub user_config:String,
    pub visual_config:String,
}

#[derive(Default, Debug, Clone)]
struct Vertex {
    position: [f32; 3],
}

impl VulkanTest {
    pub fn init(&mut self, application_name:&'static str, application_version:&MoleculeApplicationVersion, event_loop:&EventLoop<()>) -> std::sync::Arc<vulkano::swapchain::Surface<winit::window::Window>> {
        self.state_ignore = Some(VulkanState {
            cpu_buffers_u8_slice: vec![],
            cpu_buffers_vertex_slice: vec![],
            instance: Some(match Instance::new(
                Some(
                    &ApplicationInfo {
                        application_name: Some(Cow::from(application_name)),
                        application_version: Some(VulkanVersion {
                            major:application_version.major,
                            minor:application_version.minor,
                            patch:application_version.patch,
                        }),
                        engine_name: Some(Cow::from("Molecule Engine")),
                        engine_version: Some(unsafe {VulkanVersion {
                            major:ENGINE_VERSION.major,
                            minor:ENGINE_VERSION.minor,
                            patch:ENGINE_VERSION.patch,
                        }})
                    }
                ),
                VulkanVersion::V1_2,
                &vulkano_win::required_extensions(),
                None,
            ) {
                Ok(instance) => {instance.clone()},
                Err(e) => panic!("Error creating Vulkan Instance: {}", e)
            },
            ),
            compute_pipelines: vec![],
            dynamic_state: None,
            framebuffers: None,
            graphics_pipelines: vec![],
            graphics_set: None,
            previous_frame_end: None,
            queue: None,
            storage_image_persistent_descriptor_sets: vec![],
            storage_images: vec![],
            swapchain: None,
        });
        let mut state_ignore = self.state_ignore.take().unwrap();
        
        // Let's replicate some stuff from the Vulkano rs tutorial to make sure everything's working and provide a simple compute shader template

        let instance = state_ignore.instance.take().unwrap();

        let surface = WindowBuilder::new().build_vk_surface(event_loop, instance.clone()).unwrap();

        let physical = PhysicalDevice::enumerate(&instance).next().unwrap();

        let caps = surface.capabilities(physical).expect("Could not get surface capabilities");

        let dimensions = caps.current_extent.unwrap_or([1024, 1024]);
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;

        let queue_family = physical.queue_families()
            .find(|&family| family.supports_graphics())
            .expect("Couldn't find a queue family.");
        
        let (device, mut queues) = {
            Device::new(physical, &Features::none(),
                &DeviceExtensions {
                    khr_swapchain:true,
                    .. DeviceExtensions::none()
                },
                [(queue_family, 0.5)].iter().cloned()
            ).expect("failed to create device")
        };

        let queue = queues.next().unwrap();
        state_ignore.queue = Some(queue.clone());

        let previous_frame_end = Some(sync::now(device.clone()).boxed());
        state_ignore.previous_frame_end = previous_frame_end;

        let (swapchain, images) = Swapchain::start(device.clone(), surface.clone())
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
            .build().unwrap();

        state_ignore.swapchain = Some(swapchain.clone());

        let source_iter = 0 .. 65536 as u32;
        let source_buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, source_iter).expect("Could not create source CpuAccessibleBuffer!");
        
        let image = StorageImage::new(device.clone(), ImageDimensions::Dim2d {width: 1024, height: 1024, array_layers:1}, Format::R8G8B8A8Unorm, Some(queue.family())).unwrap();
        state_ignore.storage_images.push(Some(image.clone()));

        let image_dest_buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, (0..1024*1024*4).map(|_| 0u8)).expect("Could not create image destination CpuAccessibleBuffer");
        state_ignore.cpu_buffers_u8_slice.push(Some(image_dest_buffer.clone()));

        let vertex_buffer = {
            
            vulkano::impl_vertex!(Vertex, position);
    
            CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage::all(),
                false,
                [
                    Vertex {
                        position: [-1.0, -1.0, 1.0],
                    },
                    Vertex {
                        position: [-1.0, 1.0, 1.0],
                    },
                    Vertex {
                        position: [1.0, 1.0, 1.0],
                    },
                    Vertex {
                        position: [1.0, 1.0, 1.0],
                    },
                    Vertex {
                        position: [1.0, -1.0, 1.0],
                    },
                    Vertex {
                        position: [-1.0, -1.0, 1.0],
                    },
                ]
                .iter()
                .cloned(),
            )
            .unwrap()
        };
        state_ignore.cpu_buffers_vertex_slice.push(Some(vertex_buffer));

        let uniform_buffer = CpuBufferPool::<shaders::edges_shader::ty::Data>::new(device.clone(), BufferUsage::all());

        let uniform_subbuffer = {
            let uniform_data = shaders::edges_shader::ty::Data {
                window_dimensions: [800, 600],
            };

            uniform_buffer.next(uniform_data).unwrap()
        };

        let test_compute_shader = shaders::test_compute_shader::Shader::load(device.clone()).expect("Failed to load compute Shader module.");
        let mandelbrot_compute_shader = shaders::mandelbrot_compute_shader::Shader::load(device.clone()).expect("Failed to load compute Shader module.");
        let mandelbrot_shader = shaders::mandelbrot_shader::Shader::load(device.clone()).expect("Failed to load graphical Shader module");
        let standard_shader = shaders::standard_vertex_shader::Shader::load(device.clone()).expect("Failed to load graphical Shader module.");

        let render_pass = Arc::new(
            vulkano::single_pass_renderpass!(
                device.clone(),
                attachments: {
                    // `color` is a custom name we give to the first and only attachment.
                    color: {
                        // `load: Clear` means that we ask the GPU to clear the content of this
                        // attachment at the start of the drawing.
                        load: Clear,
                        // `store: Store` means that we ask the GPU to store the output of the draw
                        // in the actual image. We could also ask it to discard the result.
                        store: Store,
                        // `format: <ty>` indicates the type of the format of the image. This has to
                        // be one of the types of the `vulkano::format` module (or alternatively one
                        // of your structs that implements the `FormatDesc` trait). Here we use the
                        // same format as the swapchain.
                        format: swapchain.format(),
                        // TODO:
                        samples: 1,
                    }
                },
                pass: {
                    // We use the attachment named `color` as the one and only color attachment.
                    color: [color],
                    // No depth-stencil attachment is indicated with empty brackets.
                    depth_stencil: {}
                }
            )
            .unwrap(),
        );

        let graphics_pipeline = Arc::new(
            GraphicsPipeline::start()
                // We need to indicate the layout of the vertices.
                // The type `SingleBufferDefinition` actually contains a template parameter corresponding
                // to the type of each vertex. But in this code it is automatically inferred.
                .vertex_input_single_buffer::<Vertex>()
                // A Vulkan shader can in theory contain multiple entry points, so we have to specify
                // which one. The `main` word of `main_entry_point` actually corresponds to the name of
                // the entry point.
                .vertex_shader(standard_shader.main_entry_point(), ())
                // The content of the vertex buffer describes a list of triangles.
                .triangle_list()
                // Use a resizable viewport set to draw over the entire window
                .viewports_dynamic_scissors_irrelevant(1)
                // See `vertex_shader`.
                .fragment_shader(mandelbrot_shader.main_entry_point(), ())
                // We have to indicate which subpass of which render pass this pipeline is going to be used
                // in. The pipeline will only be usable from this particular subpass.
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                // Now that our builder is filled, we call `build()` to obtain an actual pipeline.
                .build(device.clone())
                .unwrap(),
        );
        let graphics_layout = graphics_pipeline.layout().clone();
        let graphics_set = Arc::new(
            PersistentDescriptorSet::start(graphics_layout.clone().descriptor_set_layout(0).unwrap().clone())
                .add_buffer(uniform_subbuffer)
                .unwrap()
                .build()
                .unwrap(),
        );
        state_ignore.graphics_set = Some(graphics_set);
        state_ignore.graphics_pipelines.push(Some(graphics_pipeline));
        
        let mut dynamic_state = DynamicState {
            line_width: None,
            viewports: None,
            scissors: None,
            compare_mask: None,
            write_mask: None,
            reference: None,
        };
        

        let framebuffers = renderer::window_size_dependent_setup(&images, render_pass.clone(), &mut dynamic_state);
        state_ignore.framebuffers = Some(framebuffers);

        state_ignore.dynamic_state = Some(dynamic_state);

        let test_compute_pipeline = Arc::new(ComputePipeline::new(device.clone(), &test_compute_shader.main_entry_point(), &(), None).unwrap());
        state_ignore.compute_pipelines.push(Some(test_compute_pipeline.clone()));
        
        let layout_test = test_compute_pipeline.layout().descriptor_set_layout(0).unwrap();

        let test_set = Arc::new(PersistentDescriptorSet::start(layout_test.clone())
            .add_buffer(source_buffer.clone()).unwrap()
            .build().unwrap());

        let mandelbrot_compute_pipeline = Arc::new(ComputePipeline::new(device.clone(), &mandelbrot_compute_shader.main_entry_point(), &(), None).unwrap());
        state_ignore.compute_pipelines.push(Some(mandelbrot_compute_pipeline.clone()));

        let layout_mandelbrot = mandelbrot_compute_pipeline.layout().descriptor_set_layout(0).unwrap();

        let mandelbrot_set = Arc::new(PersistentDescriptorSet::start(layout_mandelbrot.clone())
            .add_image(ImageView::new(image.clone()).unwrap())
                .unwrap()
            .build().unwrap());

        state_ignore.storage_image_persistent_descriptor_sets.push(Some(mandelbrot_set.clone()));

        {
            let mut command_buffer_builder_1 = AutoCommandBufferBuilder::primary(device.clone(), queue.family(), CommandBufferUsage::MultipleSubmit).unwrap();
            command_buffer_builder_1
                .dispatch([1024, 1, 1], test_compute_pipeline.clone(), test_set.clone(), (), 0..0).unwrap()
                .clear_color_image(image.clone(), ClearValue::Float([0.1, 0.0, 1.0, 1.0])).unwrap();

            let command_buffer_1 = command_buffer_builder_1.build().unwrap();
            
            let _ = command_buffer_1.execute(queue.clone()).unwrap();

            // let secondary_thread_lock = state_ignore.wait_channels.Vulkan_Test_Secondary_Thread_Done.0.clone();

            // let main_thread_lock = state_ignore.wait_channels.Vulkan_Test_Main_Thread_Received_Done.1.clone();

            thread::spawn(move || {
                let mut iteration = 0;
                loop {
                    let mut command_buffer_builder_2 = AutoCommandBufferBuilder::primary(device.clone(), queue.family(), CommandBufferUsage::MultipleSubmit).unwrap();
                    command_buffer_builder_2
                        .clear_color_image(image.clone(), ClearValue::Float([(iteration%70) as f32*0.1, 0.0, 1.0, 1.0])).unwrap()
                        .dispatch([128, 128, 1], mandelbrot_compute_pipeline.clone(), mandelbrot_set.clone(), (), 0..0).unwrap()
                        .copy_image_to_buffer(image.clone(), image_dest_buffer.clone()).unwrap();
                    let command_buffer_2 = command_buffer_builder_2.build().unwrap();
                    let future_1 = sync::now(device.clone())
                        .then_execute(queue.clone(), command_buffer_2).unwrap()
                        .then_signal_fence_and_flush().unwrap();
        
                    future_1.wait(None).unwrap(); //completely optional if you're not reading to CPU

                    // secondary_thread_lock.lock().unwrap().send(());
                    // main_thread_lock.lock().unwrap().recv();
                    iteration+=1;
                }
            });
        }

        
        let mut i = 0;
        for val in source_buffer.clone().read().unwrap().iter() {
            i+=1;
            println!("{}: {}",i, val);
        }

        state_ignore.instance = Some(instance);
        self.state_ignore = Some(state_ignore);

        surface
    }

    pub fn render(&mut self) -> Result<i32, &'static str> {//a lot of this handles window management, but we'll probably leave that in it's own system and run the vast majority of vulkan code in Tasks
        let mut state_ignore = self.state_ignore.take().unwrap();
        let instance = state_ignore.instance.take().unwrap();

        let mandelbrot_compute_pipeline = state_ignore.compute_pipelines[1].as_mut().unwrap();
        let graphics_pipeline = state_ignore.graphics_pipelines[0].as_mut().unwrap();

        let device = mandelbrot_compute_pipeline.device();
        let queue = state_ignore.queue.take().unwrap();

        let framebuffers = state_ignore.framebuffers.take().unwrap();
        let dynamic_state = state_ignore.dynamic_state.take().unwrap();

        let image = state_ignore.storage_images[0].take().unwrap();
        let image_dest_buffer = state_ignore.cpu_buffers_u8_slice[0].take().unwrap();
        let mandelbrot_set = state_ignore.storage_image_persistent_descriptor_sets[0].take().unwrap();

        let graphics_set = state_ignore.graphics_set.take().unwrap();

        let vertex_buffer = state_ignore.cpu_buffers_vertex_slice[0].take().unwrap();

        let swapchain = state_ignore.swapchain.take().unwrap();

        let swapchain_next = swapchain::acquire_next_image(swapchain.clone(), None).unwrap();

        let clear_values = vec![[1.0, 0.0, 1.0, 1.0].into()];

        let mut command_buffer_builder_1 = AutoCommandBufferBuilder::primary(device.clone(), queue.family(), CommandBufferUsage::MultipleSubmit).unwrap();
        command_buffer_builder_1
            .begin_render_pass(framebuffers[swapchain_next.0].clone(), SubpassContents::Inline, clear_values).unwrap()
            .draw(graphics_pipeline.clone(), &dynamic_state, vertex_buffer.clone(), graphics_set.clone(), (800, 600), 0..0).unwrap()
            .end_render_pass().unwrap();

        let command_buffer_1 = command_buffer_builder_1.build().unwrap();
        

        let future = state_ignore.previous_frame_end.take().unwrap()
            .join(swapchain_next.2)
            .join(sync::now(device.clone()))
            .then_execute(queue.clone(), command_buffer_1).unwrap()
            .then_swapchain_present(queue.clone(), swapchain.clone(), swapchain_next.0)
            .then_signal_fence_and_flush();      
        

        match future {
            Ok(future) => {
                static mut EVERY_10:u32 = 0;
                let go = unsafe {EVERY_10==9};
                unsafe {EVERY_10+=1};
                if go {
                    let future = future.join(sync::now(device.clone()));
                    // state_ignore.wait_channels.Vulkan_Test_Secondary_Thread_Done.1.lock().unwrap().recv().unwrap();
                    //do something with buffer
                    // state_ignore.wait_channels.Vulkan_Test_Main_Thread_Received_Done.0.lock().unwrap().send(()).unwrap();
                    state_ignore.previous_frame_end = Some(future.boxed());
                } else {
                    state_ignore.previous_frame_end = Some(future.boxed());
                }
            }
            Err(FlushError::OutOfDate) => {
                // recreate_swapchain = true;
                state_ignore.previous_frame_end = Some(sync::now(device.clone()).boxed());
            }
            Err(e) => {
                panic!("Failed to flush future: {:?}", e);
                // state_ignore.previous_frame_end = Some(sync::now(device.clone()).boxed());
            }
        }


        
        state_ignore.dynamic_state = Some(dynamic_state);
        state_ignore.instance = Some(instance);
        state_ignore.queue = Some(queue);
        state_ignore.storage_images[0] = Some(image);
        state_ignore.cpu_buffers_u8_slice[0] = Some(image_dest_buffer);
        state_ignore.cpu_buffers_vertex_slice[0] = Some(vertex_buffer);
        state_ignore.framebuffers = Some(framebuffers);
        state_ignore.graphics_set = Some(graphics_set);
        state_ignore.storage_image_persistent_descriptor_sets[0] = Some(mandelbrot_set);
        state_ignore.swapchain = Some(swapchain);
        self.state_ignore = Some(state_ignore);
        self.time+=1;
        Result::Ok(0)
    }
}