use std::{
    sync::{
        Arc,
        Mutex,
        RwLock,
    },
    collections::hash_map::DefaultHasher,
    // thread,
    thread::JoinHandle,
};

use crate::{
    metadata::{
        app::MoleculeApplicationData,
    },
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

pub struct VoxelWorld {
    pub application_data:MoleculeApplicationData,
    
    pub join_list: Arc<Mutex<Option<Vec<JoinHandle<bool>>>>>,

    pub insecure_hasher: Option<DefaultHasher>,

    pub controller: Controller,
    pub networking: Networking,
    pub physics: Physics,
    pub renderer: Arc<Mutex<Renderer>>,
    pub storage: HybridOctree,
}

impl VoxelWorld {
    pub fn start(world_lock:Arc<RwLock<VoxelWorld>>) {
        println!("Voxel World Initializing");//need a logger

        // let world = &mut *world_lock.write().unwrap();
        let copy_world_lock = world_lock.clone();
        // world.join_list.lock().unwrap().as_mut().unwrap().push(thread::spawn(|| {
        //     self.physics.start();
        // }));
        // world.join_list.lock().unwrap().as_mut().unwrap().push(thread::spawn(|| {
        //     self.controller.start();
        // }));
        Renderer::any_start(copy_world_lock);
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