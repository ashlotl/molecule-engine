pub mod vulkan_gpu_raytrace;

use std::sync::{
    Arc,
    Mutex,
};

use crate::enum_none;

use crate::voxels::{
    renderer::{
        vulkan_gpu_raytrace::{
            VulkanGPURaytrace,
        },
    },
    voxel_world::{
        VoxelWorld,
        WORLD_STOP,
    },
};

#[derive(Clone)]
pub enum Renderer {
    None,
    VulkanGPURaytrace(VulkanGPURaytrace),
}

enum_none!(Renderer);

pub static mut RENDER_STOP:bool=false;

impl Renderer {
    pub fn any_start(world:Arc<Mutex<VoxelWorld>>) -> bool {
        let world = &mut *world.lock().unwrap();
        if let Renderer::None = world.renderer {
            return false;
        }
        let renderer = match &world.renderer {
            Renderer::VulkanGPURaytrace(renderer) => {
                renderer
            }
            Renderer::None => {
                panic!("Reached impossible state.");
            }
        };
        
        renderer.init();
        loop {
            renderer.render();
            if unsafe {RENDER_STOP||WORLD_STOP} {
                println!("Renderer stopped");
                break;
            }
        }
        return true;
    }
}