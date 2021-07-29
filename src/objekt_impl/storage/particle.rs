use soa_derive::StructOfArray;

use crate::{
    math::vectors::{
        Vector3U16,
    }
};

pub const MATERIAL_COUNT:u64=512;

#[derive(Debug, PartialEq, StructOfArray)]
#[soa_derive = "Debug, PartialEq"]
pub struct Particle {
    pub _gpu_only_level_index_current: u32,// the level containing this Particle
    pub _gpu_only_level_index_next: u32, //the level contained by this Particle
    pub material: Vec<u8>,
    pub pos: Vector3U16,//within the bounds of the associated voxel
}