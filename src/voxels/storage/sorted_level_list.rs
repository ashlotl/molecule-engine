use crate::{
    math::vectors::VoxelLocation,
    voxels::storage::hybrid_octree::Level,
};

#[derive(Clone, Default)]
pub struct SortedLevelList {
    pub data:Vec<(u64, VoxelLocation, Level)>,
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
        let check = self.data[check_index].0;
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