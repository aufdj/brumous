struct CameraUniform {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct ParticleInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) normal_matrix_0: vec3<f32>,
    @location(10) normal_matrix_1: vec3<f32>,
    @location(11) normal_matrix_2: vec3<f32>,
    @location(12) color: vec4<f32>,
};


struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
    @location(3) color: vec4<f32>,
};

@vertex
fn vs_main(vertex: VertexInput, particle: ParticleInput) -> VertexOutput {
    var out: VertexOutput;

    let model_matrix = mat4x4<f32>(
        particle.model_matrix_0,
        particle.model_matrix_1,
        particle.model_matrix_2,
        particle.model_matrix_3,
    );
    let normal_matrix = mat3x3<f32>(
        particle.normal_matrix_0,
        particle.normal_matrix_1,
        particle.normal_matrix_2,
    );

    out.world_pos = (model_matrix * vec4<f32>(vertex.pos, 1.0)).xyz;
    out.clip_pos = camera.view_proj * model_matrix * vec4<f32>(vertex.pos, 1.0);
    out.world_normal = vertex.normal;
    out.tex_coords = vertex.tex_coords;
    out.color = particle.color;

    return out;
}

@fragment
fn fs_color(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

// @fragment
// fn fs_texture(in: VertexOutput) -> @location(0) vec4<f32> {
//     let obj_col: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);

//     let ambient_strength = 0.4;
//     let ambient_color = light.color * ambient_strength;

//     let light_dir = normalize(light.pos - in.world_pos);
//     let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
//     let diffuse_color = light.color * diffuse_strength;

//     let result = (ambient_color + diffuse_color) * obj_col.xyz;

//     return vec4<f32>(result, obj_col.a);
// }