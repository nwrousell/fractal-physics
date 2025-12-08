// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct ObjectData {
    ctm : mat4x4<f32>,
    normal_matrix: mat3x3<f32>
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<storage, read> object_data : array<ObjectData>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
}

@vertex
fn vs_main(
    vertex: VertexInput,
    @builtin(instance_index) instance_idx : u32
) -> VertexOutput {
    var out: VertexOutput;
    let model = object_data[instance_idx].ctm;
    let normal_matrix = object_data[instance_idx].normal_matrix;
    out.tex_coords = vertex.tex_coords;
    out.clip_position = camera.view_proj * model * vec4<f32>(vertex.position, 1.0);
    out.normal = normalize(normal_matrix * vertex.normal);

    return out;
}

// Fragment shader

struct Light {
    position: vec3<f32>,
    used: u32,
    color: vec3<f32>,
};

@group(1) @binding(0)
var<uniform> lights: array<Light, 8>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color = vec4<f32>(1.0, 1.0, 1.0, 1.0);

    // ambient
    let ambient_strength = 0.2;
    var result = object_color * ambient_strength;

    // for each light
    let diffuse_strength = 0.8;
    for (var i = 0u; i < 8u; i++) {
        if (lights[i].used != 0u) {
            let light = lights[i];

            let distance_to_light = length(light.position - in.clip_position.xyz);
            // let att = min(1, 1/(0.8 + distance_to_light * 0.002 + distance_to_light*distance_to_light * 0.0));
            let att = 1.0;

            let direction_to_light = normalize(light.position - in.clip_position.xyz);

            // diffuse
            let lambert_factor = max(0.0, dot(direction_to_light, in.normal));
            result += vec4<f32>(light.color, 1.0) * lambert_factor * diffuse_strength * att;

            // specular
            // TODO
        }
    }

    return result;
}
