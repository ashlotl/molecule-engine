use std::{
    sync::{
        Arc,
        Mutex,
    },
    collections::hash_map::DefaultHasher,
    thread,
    thread::JoinHandle,
};

use crate::{
    voxels::{
        controller::{
            Controller,
        },
        networking::{
            Networking,
        },
        physics::{
            Physics,
        },
        renderer::{
            Renderer,
        },
        storage::{
            hybrid_octree::HybridOctree,
        },
    },
};

pub static mut WORLD_STOP:bool = false;


pub struct VoxelWorld {
    pub join_list: Option<Vec<JoinHandle<bool>>>,

    pub insecure_hasher: Option<DefaultHasher>,

    pub controller: Controller,
    pub networking: Networking,
    pub physics: Physics,
    pub renderer: Renderer,
    pub storage: HybridOctree,
}

impl VoxelWorld {
    pub fn start(world_mutex:Arc<Mutex<VoxelWorld>>) {
        println!("Voxel World Initializing");//need a logger

        let world = &mut *world_mutex.lock().unwrap();
        let copy_world_mutex = world_mutex.clone();
        world.join_list.as_mut().unwrap().push(thread::spawn(|| {
            return Renderer::any_start(copy_world_mutex);
        }));
        // self.physics.start();
        // self.controller.start();
    }
}

#[macro_export]
macro_rules! enum_none {
    ($x:ident) => {
        impl Default for $x {
            fn default() -> Self {
                Self::None
            }
        }
    };
}