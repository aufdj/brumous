struct CameraUniform {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct Light {
    pos: vec3<f32>,
    col: vec3<f32>,
}
@group(1) @binding(0)
var<uniform> light: Light;


struct VertexInput {
    @location(0) pos: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) col: vec3<f32>,
};

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    let scale = 0.005;
    var out: VertexOutput;
    out.clip_pos = camera.view_proj * vec4<f32>(vertex.pos * scale + light.pos, 1.0);
    out.col = light.col;
    return out;
}


@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.col, 1.0);
}




