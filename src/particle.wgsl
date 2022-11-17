struct Camera {
    view_proj: mat4x4<f32>,
    view_pos: vec4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: Camera;

struct ParticleInput {
    @location(5) model_mat_0: vec4<f32>,
    @location(6) model_mat_1: vec4<f32>,
    @location(7) model_mat_2: vec4<f32>,
    @location(8) model_mat_3: vec4<f32>,
    @location(9) norm_mat_0: vec3<f32>,
    @location(10) norm_mat_1: vec3<f32>,
    @location(11) norm_mat_2: vec3<f32>,
    @location(12) color: vec4<f32>,
};

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) norm: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_norm: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
    @location(3) color: vec4<f32>,
};


@group(1) @binding(0)
var tx: texture_2d<f32>;
@group(1) @binding(1)
var smpl: sampler;

struct Light {
    pos: vec4<f32>,
    color: vec4<f32>,
    pad1: vec4<f32>,
    pad2: vec4<f32>,
};
@group(2) @binding(0)
var<storage, read> lights: array<Light>;


@vertex
fn vs_main(vertex: VertexInput, particle: ParticleInput) -> VertexOutput {
    var out: VertexOutput;

    let model_mat = mat4x4<f32>(
        particle.model_mat_0,
        particle.model_mat_1,
        particle.model_mat_2,
        particle.model_mat_3,
    );
    let norm_mat = mat3x3<f32>(
        particle.norm_mat_0,
        particle.norm_mat_1,
        particle.norm_mat_2,
    );

    out.world_pos = (model_mat * vec4<f32>(vertex.pos, 1.0)).xyz;
    out.clip_pos = camera.view_proj * model_mat * vec4<f32>(vertex.pos, 1.0);
    out.world_norm = norm_mat * vertex.norm;
    out.uv = vertex.uv;
    out.color = particle.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var result = vec3<f32>(0.0, 0.0, 0.0);

    for (var i: i32 = 0; i < i32(arrayLength(&lights)); i++) {
        let light_pos = lights[i].pos.xyz;
        let light_color = lights[i].color.xyz;

        let ambient_strength = 0.1;
        let ambient_color = light_color * ambient_strength;

        let light_dir = normalize(light_pos - in.world_pos);
        let diffuse_strength = max(dot(in.world_norm, light_dir), 0.0);
        let diffuse_color = light_color * diffuse_strength;

        let view_dir = normalize(camera.view_pos.xyz - in.world_pos);
        let reflect_dir = reflect(-light_dir, in.world_norm);

        let specular_strength = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
        let specular_color = specular_strength * light_color;

        result += (ambient_color + diffuse_color + specular_color) * in.color.xyz;
    }
    return vec4<f32>(result, in.color.a);
}


// @fragment
// fn fs_texture(in: VertexOutput) -> @location(0) vec4<f32> {
//     let obj_col: vec4<f32> = textureSample(tx, smpl, in.uv);

//     var result = vec3<f32>(0.0, 0.0, 0.0);
    
//      for (var i: i32 = 0; i < i32(arrayLength(&lights)); i++) {
//         let light_pos = lights[i].pos.xyz;
//         let light_color = lights[i].color.xyz;

//         let ambient_strength = 0.1;
//         let ambient_color = light_color * ambient_strength;

//         let light_dir = normalize(light_pos - in.world_pos);
//         let diffuse_strength = max(dot(in.world_norm, light_dir), 0.0);
//         let diffuse_color = light_color * diffuse_strength;

//         let view_dir = normalize(camera.view_pos.xyz - in.world_pos);
//         let reflect_dir = reflect(-light_dir, in.world_norm);

//         let specular_strength = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
//         let specular_color = specular_strength * light_color;

//         result += (ambient_color + diffuse_color + specular_color) * obj_col.xyz;
//     }
//     return vec4<f32>(result, obj_col.a);
// }