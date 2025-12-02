
// vertex shader that creates full-screen triangle
struct VertexOut {
    @builtin(position) position : vec4<f32>,
    @location(0) uv : vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOut {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>( 3.0,  1.0),
        vec2<f32>(-1.0,  1.0)
    );
    var out : VertexOut;
    out.position = vec4<f32>(pos[idx], 0.0, 1.0);
    out.uv = (pos[idx] * 0.5) + vec2<f32>(0.5, 0.5);
    out.uv.y = 1 - out.uv.y;
    return out;
}

@group(0) @binding(0) var sceneTex: texture_2d<f32>;
@group(0) @binding(1) var sceneSampler: sampler;

const SOBEL_KERNEL_X : array<vec3<f32>, 3> = array<vec3<f32>, 3>(
    vec3<f32>(-1.0,  0.0,  1.0),
    vec3<f32>(-2.0,  0.0,  2.0),
    vec3<f32>(-1.0,  0.0,  1.0)
);

const SOBEL_KERNEL_Y : array<vec3<f32>, 3> = array<vec3<f32>, 3>(
    vec3<f32>(-1.0, -2.0, -1.0),
    vec3<f32>( 0.0,  0.0,  0.0),
    vec3<f32>( 1.0,  2.0,  1.0)
);

const UV_DELTA : f32 = 0.01;

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    var dx: vec4<f32> = vec4(0.0);
    var dy: vec4<f32> = vec4(0.0);
    for(var y = -1; y <= 1; y = y + 1){
        for(var x = -1; x <= 1; x = x + 1){
            let point: vec2<f32> = in.uv + UV_DELTA * vec2(f32(x), f32(y));
            let tex_color = textureSample(sceneTex, sceneSampler, point);
            dx += SOBEL_KERNEL_X[y][x] * tex_color;
            dy += SOBEL_KERNEL_Y[y][x] * tex_color;
        }
    }

    let edge = max(dx, dy);
    return vec4<f32>(edge.rgb, 1.0);
}