use crate::enum_none;

#[derive(Clone)]
pub enum Controller {
    None,
}

enum_none!(Controller);