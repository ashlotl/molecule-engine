use crate::{
    math::vectors::VoxelLocation,
    voxels::storage::hybrid_octree::Level,
};

pub struct SortedLevel {
    pub level: Level,//inner data
    pub location: VoxelLocation,//the location of the voxel on the lowest position on each axis that is still within this level. Remember that VoxelLocations are always global, not local/relative.
    pub ordinal:u64,//how to position this level relative to other levels
}

pub struct SortedLevelList {
    pub data:Vec<SortedLevel>,
}

impl SortedLevelList {
    pub fn new() -> Self {
        Self {
            data:Vec::new(),
        }
    }

    fn get_recur(&self, pos:u64, lower_i:usize, upper_e:usize)->(bool, usize) {
        if upper_e==lower_i {
            return (false, lower_i)
        }
        let check_index = (upper_e-lower_i)/2+lower_i;
        let check = self.data[check_index].ordinal;
        if pos<check {
            self.get_recur(pos, lower_i, check_index)
        } else if pos>check {
            self.get_recur(pos, check_index+1, upper_e)
        } else {
            (true, check_index)
        }
    }

    pub fn get(&self, pos:u64) -> (bool,usize) {
        self.get_recur(pos, 0, self.data.len())
    }
}