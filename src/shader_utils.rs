#[allow(dead_code)]
pub mod vs {
#[derive(VulkanoShader)]
#[ty = "vertex"]
#[src = "
    #version 450
    layout(location = 0) in vec3 position;

    layout(location = 1) in vec3 instance_position;
    layout(location = 2) in vec3 instance_color;

    layout(location = 0) out vec3 color;

    layout(set = 0, binding = 0) uniform UniformMatrices {
        mat4 world;
        mat4 look_at;
    } uniforms;

    layout(set = 1, binding = 0) uniform DeltaUniform {
        float delta;
    } delta_uniform;

    void rotation(in float angle, in vec3 vector, out mat4 r_matrix) {
        float x = vector.x;
        float y = vector.y;
        float z = vector.z;

        float c = cos(angle);
        float s = sin(angle);

        float a1 = x*x*(1 - c) + c;
        float a2 = x*y*(1 - c) - z*s;
        float a3 = x*z*(1 - c) + y*s;

        float b1 = y*x*(1 - c) + z*s;
        float b2 = y*y*(1 - c) + c;
        float b3 = y*z*(1 - c) - x*s;

        float c1 = z*x*(1 - c) - y*s;
        float c2 = z*y*(1 - c) + x*s;
        float c3 = z*z*(1 - c) + c;


        r_matrix = mat4(
            vec4(a1, b1, c1, instance_position.x),
            vec4(a2, b2, c2, instance_position.y),
            vec4(a3, b3, c3, instance_position.z),
            vec4(0.0, 0.0, 0.0, 1.0)
        );
    }

    void main() {
        color = instance_color;

        mat4 r_matrix;
        rotation(delta_uniform.delta, vec3(0.0, 1.0, 0.0), r_matrix);

        mat4 final_world = r_matrix * uniforms.look_at * uniforms.world;

        gl_Position = vec4(position, 1.0) * final_world;
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
