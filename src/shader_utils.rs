#[allow(dead_code)]
pub mod vs {
#[derive(VulkanoShader)]
#[ty = "vertex"]
#[src = "
    #version 450
    layout(location = 0) in vec2 position;

    layout(location = 1) in vec2 instance_position;
    layout(location = 2) in vec3 instance_color;

    layout(location = 0) out vec3 color;

    layout(set = 0, binding = 0) uniform UniformMatrices {
        mat4 world;
    } uniforms;

    void main() {
        color = instance_color;
        gl_Position = vec4(position + instance_position, -5.0, 1.0) * uniforms.world;
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
    layout(location = 0) in vec3 color;

    layout(location = 0) out vec4 f_color;

    void main() {
        f_color = vec4(color, 1.0);
    }
"]
struct Dummy;
}
