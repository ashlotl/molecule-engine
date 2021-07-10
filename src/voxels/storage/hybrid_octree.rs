use std::{
    collections::hash_map::DefaultHasher,
    hash::{
        Hash,
        Hasher,
    },
    sync::RwLock,
};

use crate::{
    math::{
        constants::{
            EPSILON,
        },
        vectors::{
            Vector3U16,
            VoxelLocation,
        },
    },
    voxels::{
        storage::{
            sorted_level_list::SortedLevelList,
            particle::{
                MATERIAL_COUNT,
                Particle,
            },
        },
    },
};

#[derive(Debug, Clone)]
pub struct LevelContents {
    pub loaded:bool,
    pub data:Vec<Particle>,
}

#[derive(Debug)]
pub struct Level {
    pub contents:RwLock<LevelContents>
}

impl Clone for Level {
    fn clone(&self) -> Self {
        let lcontents = &*self.contents.read().expect("Could not lock Level for read access");
        Level {
            
            contents: RwLock::new(lcontents.clone())
        }
    }
}

#[derive(Clone, Default)]
pub struct HybridOctree {
    pub level_depth:u64,//number of levels
    pub level_length:u64,
    pub divisions_per_level:u32,
    pub levels:SortedLevelList,
}

impl HybridOctree {
    pub fn new(level_depth:u64, level_length:u64) -> Self {
        if level_length<=1 {
            panic!("Level sizes must exceed 1 on each axis");
        }
        
        let _divisions_per_level=((level_length as f64).log2()+EPSILON) as u32;

        HybridOctree {
            level_depth:level_depth,
            level_length:level_length,
            divisions_per_level:_divisions_per_level,
            
            levels:SortedLevelList::new(),
        }
    }

    pub fn get_level(&self, pos:VoxelLocation) -> &Level {
                let index = self.levels.get(pos.linearize(self.level_length, self.level_length, self.level_length, self.level_depth));
                if !index.0 {
                    panic!("This Level is not loaded")
                }
                &self.levels.data.get(index.1).unwrap().2
    }

    pub fn load_level(&mut self, pos:VoxelLocation, mut insecure_hasher:DefaultHasher) -> Option<DefaultHasher> {
        let linearized = pos.linearize(self.level_length, self.level_length, self.level_length, self.level_depth);
        let index = self.levels.get(linearized).1;
        match self.levels.data.get_mut(index) {
            None => {
                println!("Level that needs creation requested at {:?}", pos);
                self.levels.data.insert(
                    index,
                    (
                        linearized,
                        pos,
                        Level {
                            contents:RwLock::new(LevelContents {
                                loaded:true,
                                data: {

                                    let volume = self.level_length.pow(3);
                                    let mut buffer = Vec::with_capacity(volume as usize);
                                    //TODO: read from file
                                    let half_limit = 2<<15;
                                    for i in 0..volume {
                                        buffer.push(Particle {
                                            _gpu_only_level_index_current:0,
                                            _gpu_only_level_index_next:0,
                                            material: (0..MATERIAL_COUNT).map(|m|
                                                {
                                                    (((i*MATERIAL_COUNT+m)%256) as u8).hash(&mut insecure_hasher);
                                                    (insecure_hasher.finish()%256) as u8
                                                }
                                            ).into_iter().collect(),
                                            pos: Vector3U16 {x:half_limit, y:half_limit, z:half_limit}
                                        });
                                    }
                                    buffer
                                },
                            }),
                        }
                    )
                );

            }
            Some(level) => {
                (*level.2.contents.write().expect("Could not lock Level for write access")).loaded=true;
            }
        }
        Some(insecure_hasher)
    }
}