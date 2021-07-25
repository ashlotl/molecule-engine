use crate::{
    metadata::versions::MoleculeApplicationVersion,
};

pub struct MoleculeApplicationData {
    pub name:&'static str,
    pub version:MoleculeApplicationVersion,
}