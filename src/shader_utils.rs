#[allow(dead_code)]
pub mod vs {
#[derive(VulkanoShader)]
#[ty = "vertex"]
#[src = "
    #version 450
    layout(location = 0) in vec2 position;
    layout(set = 0, binding = 0) uniform UniformMatrices {
        mat4 world;
    } uniforms;
    void main() {
        gl_Position = vec4(position, 0.0, 1.0) * uniforms.world;
    }
"]
struct Dummy;
}

#[allow(dead_code)]
pub mod fs {
#[derive(VulkanoShader)]
#[ty = "fragment"]
#[src = "
    #version 450
    layout(location = 0) out vec4 f_color;
    layout(set = 0, binding = 1) uniform MetaColor {
        vec4 incolor;
    } meta_color;
    void main() {
        f_color = meta_color.incolor;
    }
"]
struct Dummy;
}
