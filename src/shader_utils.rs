#[allow(dead_code)]
pub mod vs {
#[derive(VulkanoShader)]
#[ty = "vertex"]
#[src = "
    #version 450
    layout(location = 0) in vec2 position;
    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
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
    // layout(binding = 0) uniform meta {
    //     vec4 incolor;
    // };
    void main() {
        f_color = vec4(1.0, 0.0, 0.0, 1.0);
    }
"]
struct Dummy;
}
