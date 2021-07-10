mod math;
mod voxels;

use std::{
    collections::hash_map::DefaultHasher,
    sync::{
        Arc,
        Mutex,
    },
};

use crate::{
    math::{
        vectors::{
            Vector3U64,
            VoxelLocation,
        }
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
            vulkan_gpu_raytrace::{
                Configs,
                VulkanGPURaytrace,
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

fn main() {
    let o_world = VoxelWorld {
        join_list:Some(vec![]),
        insecure_hasher:Some(DefaultHasher::new()),
        controller:Controller::None,
        networking:Networking::None,
        physics:Physics::None,
        renderer:Renderer::VulkanGPURaytrace(VulkanGPURaytrace {//display for storage
            configs: Configs {
                user_config: "assets/settings/renderer/vulkan_gpu_raytrace/user_config.json",
                visual_config: "assets/settings/renderer/vulkan_gpu_raytrace/visual_config.json",
            },
            shader_path: "assets/shaders/renderer/vulkan_gpu_raytrace/",
        }),
        storage:HybridOctree::new(
            5,
            4,
        ),
    };
    let world_mutex = Arc::new(Mutex::new(o_world));
    VoxelWorld::start(world_mutex.clone());

    let world_main = world_mutex.clone();
    let mut world = world_main.lock().unwrap();
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
    let mut i = 0;
    for handle in world.join_list.take().unwrap() {
        i+=1;
        handle.join().expect(format!("Error joining thread! See thread in position {}", i).as_str());
    }
}
