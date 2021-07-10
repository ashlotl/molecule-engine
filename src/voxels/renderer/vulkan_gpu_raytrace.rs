#[derive(Clone)]
pub struct VulkanGPURaytrace {
    pub configs: Configs,
    pub shader_path: &'static str,
}

#[derive(Clone)]
pub struct Configs {
    pub user_config:&'static str,
    pub visual_config:&'static str,
}

impl VulkanGPURaytrace {
    pub fn init(&self) {
        
    }

    pub fn render(&self) {
        
    }
}