#[derive(Clone, Debug)]
pub struct VoxelLocation {
    pub lod:u64,
    pub vec:Vector3U64,
}

impl VoxelLocation {
    pub fn linearize(&self, size_x:u64, size_y:u64, size_z:u64, level_depth:u64) -> u64 {
        ((self.vec.x*size_y*size_z+self.vec.y*size_z+self.vec.z)*(size_x*size_y*size_z)+2).pow((level_depth-self.lod) as u32)
    }
}

#[derive(Clone, Debug)]
pub struct Vector3U64 {
    pub x:u64,
    pub y:u64,
    pub z:u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Vector3U16 {
    pub x:u16,
    pub y:u16,
    pub z:u16,
}