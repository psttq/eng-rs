// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct ModelMatrixUniform {
    model_matrix: mat4x4<f32>,
};

@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

@group(2) @binding(0) // 1.
var<uniform> modelMat: ModelMatrixUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * modelMat.model_matrix * vec4<f32>(model.position, 1.0); // 2.
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(3) @binding(0)
var<uniform> object_id: u32;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<u32> {
    return vec4<u32>(170u+object_id, 0u, 0u, 255u);
}
