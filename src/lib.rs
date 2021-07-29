#[macro_use]
extern crate mopa;

pub mod concurrency;
pub mod math;
pub mod metadata;
pub mod objekt_impl;
pub mod task_impl;
pub mod utils;

use crate::{
    metadata::{
        versions::{
            MoleculeVersion,
        },
    },
};

const ENGINE_VERSION_MAJOR:&'static str = env!("CARGO_PKG_VERSION_MAJOR");
const ENGINE_VERSION_MINOR:&'static str = env!("CARGO_PKG_VERSION_MINOR");
const ENGINE_VERSION_PATCH:&'static str = env!("CARGO_PKG_VERSION_PATCH");

pub fn get_engine_version() -> MoleculeVersion {
    MoleculeVersion {
        major:ENGINE_VERSION_MAJOR.parse().unwrap(),
        minor:ENGINE_VERSION_MINOR.parse().unwrap(),
        patch:ENGINE_VERSION_PATCH.parse().unwrap(),
    }
}