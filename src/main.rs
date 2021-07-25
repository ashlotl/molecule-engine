#[cfg(test)]
mod tests;

pub mod concurrency;
pub mod math;
pub mod metadata;
pub mod voxels;

use std::{
    collections::hash_map::DefaultHasher,
    sync::{
        Arc,
        Mutex,
        RwLock,
    },
};

use crate::{
    math::{
        vectors::{
            Vector3U64,
            VoxelLocation,
        }
    },
    metadata::{
        app::MoleculeApplicationData,
        versions::{
            MoleculeApplicationVersion,
            MoleculeVersion,
        },
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
            vulkan_test::{
                Configs,
                VulkanTest,
            },
        },
        storage::{
            hybrid_octree::{
                HybridOctree,
            },
        },
        voxel_world::{
            VoxelWorld,
        },
    },
};

const ENGINE_VERSION_MAJOR:&'static str = env!("CARGO_PKG_VERSION_MAJOR");
const ENGINE_VERSION_MINOR:&'static str = env!("CARGO_PKG_VERSION_MINOR");
const ENGINE_VERSION_PATCH:&'static str = env!("CARGO_PKG_VERSION_PATCH");

static mut ENGINE_VERSION:MoleculeVersion = MoleculeVersion {
    major:0,
    minor:0,
    patch:0,
};

fn main() {
    unsafe {
        ENGINE_VERSION = MoleculeVersion {
            major:ENGINE_VERSION_MAJOR.parse().unwrap(),
            minor:ENGINE_VERSION_MINOR.parse().unwrap(),
            patch:ENGINE_VERSION_PATCH.parse().unwrap(),
        }
    }
    let o_world = VoxelWorld {
        application_data: MoleculeApplicationData {
            name: "Molecule Development Test",
            version: MoleculeApplicationVersion {
                major:0,
                minor:0,
                patch:0,
            },
        },
        join_list:Arc::new(Mutex::new(Some(vec![]))),
        insecure_hasher:Some(DefaultHasher::new()),
        controller:Controller::None,
        networking:Networking::None,
        physics:Physics::None,
        renderer:Arc::new(Mutex::new(Renderer::VulkanTest(VulkanTest {//display for storage
            time: 0,
            state_ignore: None,
            configs: Configs {
                user_config: "assets/settings/renderer/vulkan_gpu_raytrace/user_config.json".to_string(),
                visual_config: "assets/settings/renderer/vulkan_gpu_raytrace/visual_config.json".to_string(),
            },
            shader_path: "assets/shaders/renderer/vulkan_gpu_raytrace/".to_string(),
        }))),
        storage:HybridOctree::new(
            5,
            4,
        ),
    };
    let world_lock = Arc::new(RwLock::new(o_world));
    
    {//scope important to drop world_main and trigger an unlock
        let world_main = world_lock.clone();
        let mut world = world_main.write().unwrap();
        let hasher = world.insecure_hasher.take();
        world.insecure_hasher =
            world.storage.load_level(
                VoxelLocation {
                    vec: Vector3U64 {
                        x: 0, y: 0, z: 0
                    },
                    lod: 5,
                },
                hasher.unwrap(),
            );
        let hasher = world.insecure_hasher.take();
        world.insecure_hasher = world.storage.load_level(
            VoxelLocation {
                vec: Vector3U64 {
                    x: 0, y: 0, z: 0
                },
                lod: 4,
            },
            hasher.unwrap(),
        );
        let hasher = world.insecure_hasher.take();
        world.insecure_hasher = world.storage.load_level(
            VoxelLocation {
                vec: Vector3U64 {
                    x: 0, y: 0, z: 0
                },
                lod: 3,
            },
            hasher.unwrap(),
        );
        let hasher = world.insecure_hasher.take();
        world.insecure_hasher = world.storage.load_level(
            VoxelLocation {
                vec: Vector3U64 {
                    x: 0, y: 0, z: 0
                },
                lod: 2,
            },
            hasher.unwrap(),
        );
        let hasher = world.insecure_hasher.take();
        world.insecure_hasher = world.storage.load_level(
            VoxelLocation {
                vec: Vector3U64 {
                    x: 0, y: 0, z: 0
                },
                lod: 1,
            },
            hasher.unwrap(),
        );
        println!("data length: {:#?}", world.storage.get_level(
            VoxelLocation {
                vec: Vector3U64 {
                    x: 0, y: 1, z: 0
                },
                lod: 5,
            },
        ).contents.read().expect("Failed to load level contents in main").data.len());
    }
    VoxelWorld::start(world_lock.clone());


    let world_wait = world_lock.clone();
    let world = world_wait.read().unwrap();
    let mut i = 0;
    for handle in world.join_list.lock().unwrap().take().unwrap() {
        i+=1;
        handle.join().expect(format!("Error joining thread! See thread in position {}", i).as_str());
    }
}
