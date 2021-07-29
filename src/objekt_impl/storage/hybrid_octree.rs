use std::{
    collections::hash_map::DefaultHasher,
    hash::{
        Hash,
        Hasher,
    },
    sync::{
        Arc,
        RwLock,
    },
};

use crate::{
    concurrency::molecule_objekt::MoleculeObjekt,
    math::{
        vectors::{
            Vector3U16,
            VoxelLocation,
        },
    },
    objekt_impl::{
        storage::{
            sorted_level_list::{
                SortedLevel,
                SortedLevelList,
            },
            particle::{
                MATERIAL_COUNT,
                ParticleVec,
            },
        },
    },
};

#[derive(Debug)]
pub struct LevelContents {
    pub loaded: bool,
    pub data: ParticleVec,
}

#[derive(Debug)]
pub struct Level {
    pub contents:RwLock<LevelContents>
}

#[derive(Clone)]
pub struct HybridOctreeObjekt {
    name: String,
    inner: Arc<RwLock<HybridOctree>>,
}

impl MoleculeObjekt for HybridOctreeObjekt {
    fn name(&self) -> String {
        self.name.clone()
    }
}

pub struct HybridOctree {
    pub level_depth:u64,//number of levels
    pub level_length:u64,
    pub levels:SortedLevelList,
}

impl HybridOctree {
    pub fn new(level_depth:u64, level_length:u64) -> Self {
        if level_length<=1 {
            panic!("Level sizes must exceed 1 on each axis");
        }

        HybridOctree {
            level_depth:level_depth,
            level_length:level_length,
            levels:SortedLevelList::new(),
        }
    }

    pub fn get_level(&self, pos:VoxelLocation) -> &Level {
                let index = self.levels.get(pos.linearize(self.level_length, self.level_length, self.level_length, self.level_depth));
                if !index.0 {
                    panic!("This Level is not loaded")
                }
                &self.levels.data[index.1].level
    }

    pub fn load_level(&mut self, pos:VoxelLocation, mut insecure_hasher:DefaultHasher) -> Option<DefaultHasher> {
        let linearized = pos.linearize(self.level_length, self.level_length, self.level_length, self.level_depth);
        let index = self.levels.get(linearized).1;
        match self.levels.data.get_mut(index) {
            None => {
                println!("Level that needs creation requested at {:?}", pos);
                self.levels.data.insert(
                    index,
                    SortedLevel {
                        ordinal: linearized,
                        location: pos,
                        level: Level {
                            contents:RwLock::new(LevelContents {
                                loaded:true,
                                data: {

                                    let volume = self.level_length.pow(3);
                                    let mut buffer = ParticleVec::new();
                                    //TODO: read from file
                                    let half_limit = 1<<15;
                                    for i in 0..volume {
                                        buffer.material.push(
                                            (0..MATERIAL_COUNT).map(
                                                |m| {
                                                    ((linearized*volume+(i*MATERIAL_COUNT+m)%256) as u8).hash(&mut insecure_hasher);
                                                    (insecure_hasher.finish()%256) as u8
                                                }
                                            ).collect()
                                        );
                                    }
                                    for _i in 0..volume {
                                        buffer.pos.push(
                                            Vector3U16 {
                                                x: half_limit,
                                                y: half_limit,
                                                z: half_limit,
                                            }
                                        )
                                    }
                                    buffer
                                },
                            }),
                        }
                    }
                );

            }
            Some(level) => {
                (*level.level.contents.write().expect("Could not lock Level for write access")).loaded=true;
            }
        }
        Some(insecure_hasher)
    }
}