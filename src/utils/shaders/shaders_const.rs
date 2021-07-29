
pub mod standard_vertex_shader {
    vulkano_shaders::shader!{
        ty: "vertex",
        path: "glsl/standard_vertex.glsl"
    }
}

pub mod test_compute_shader {
    vulkano_shaders::shader!{
        ty: "compute",
        path: "glsl/test_compute.glsl"
    }
}

pub mod mandelbrot_compute_shader {
    vulkano_shaders::shader!{
        ty: "compute",
        path: "glsl/mandelbrot_compute.glsl"
    }
}

pub mod edges_shader {
    vulkano_shaders::shader!{
        ty: "fragment",
        path: "glsl/center.glsl",
    }
}

pub mod mandelbrot_shader {
    vulkano_shaders::shader!{
        ty: "fragment",
        path: "glsl/mandelbrot.glsl"
    }
}