
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

struct Resolution {
    width: f32,
    height: f32,
}
@group(0) @binding(2) var<uniform> resolution: Resolution;

const SOBEL_KERNEL_X : array<array<f32, 5>, 5> = array<array<f32, 5>, 5>(
    array<f32, 5>(-2.0, -1.0,  0.0,  1.0,  2.0),
    array<f32, 5>(-2.0, -1.0,  0.0,  1.0,  2.0),
    array<f32, 5>(-4.0, -2.0,  0.0,  2.0,  4.0),
    array<f32, 5>(-2.0, -1.0,  0.0,  1.0,  2.0),
    array<f32, 5>(-2.0, -1.0,  0.0,  1.0,  2.0)
);

const SOBEL_KERNEL_Y : array<array<f32, 5>, 5> = array<array<f32, 5>, 5>(
    array<f32, 5>(-2.0, -2.0, -4.0, -2.0, -2.0),
    array<f32, 5>(-1.0, -1.0, -2.0, -1.0, -1.0),
    array<f32, 5>( 0.0,  0.0,  0.0,  0.0,  0.0),
    array<f32, 5>( 1.0,  1.0,  2.0,  1.0,  1.0),
    array<f32, 5>( 2.0,  2.0,  4.0,  2.0,  2.0)
);

const bayerMatrix2x2 = array<array<f32, 2>, 2>(
    array<f32, 2>(0.0 / 4.0, 2.0 / 4.0),
    array<f32, 2>(3.0 / 4.0, 1.0 / 4.0)
);

const bayerMatrix8x8 = array<array<f32, 8>, 8>(
    array<f32, 8>( 0.0 / 64.0, 48.0 / 64.0, 12.0 / 64.0, 60.0 / 64.0,  3.0 / 64.0, 51.0 / 64.0, 15.0 / 64.0, 63.0 / 64.0),
    array<f32, 8>(32.0 / 64.0, 16.0 / 64.0, 44.0 / 64.0, 28.0 / 64.0, 35.0 / 64.0, 19.0 / 64.0, 47.0 / 64.0, 31.0 / 64.0),
    array<f32, 8>( 8.0 / 64.0, 56.0 / 64.0,  4.0 / 64.0, 52.0 / 64.0, 11.0 / 64.0, 59.0 / 64.0,  7.0 / 64.0, 55.0 / 64.0),
    array<f32, 8>(40.0 / 64.0, 24.0 / 64.0, 36.0 / 64.0, 20.0 / 64.0, 43.0 / 64.0, 27.0 / 64.0, 39.0 / 64.0, 23.0 / 64.0),
    array<f32, 8>( 2.0 / 64.0, 50.0 / 64.0, 14.0 / 64.0, 62.0 / 64.0,  1.0 / 64.0, 49.0 / 64.0, 13.0 / 64.0, 61.0 / 64.0),
    array<f32, 8>(34.0 / 64.0, 18.0 / 64.0, 46.0 / 64.0, 30.0 / 64.0, 33.0 / 64.0, 17.0 / 64.0, 45.0 / 64.0, 29.0 / 64.0),
    array<f32, 8>(10.0 / 64.0, 58.0 / 64.0,  6.0 / 64.0, 54.0 / 64.0,  9.0 / 64.0, 57.0 / 64.0,  5.0 / 64.0, 53.0 / 64.0),
    array<f32, 8>(42.0 / 64.0, 26.0 / 64.0, 38.0 / 64.0, 22.0 / 64.0, 41.0 / 64.0, 25.0 / 64.0, 37.0 / 64.0, 21.0 / 64.0)
);

fn hatch_effect(color: vec4<f32>, uv: vec2<f32>, normalizedPixelSize: vec2<f32>) -> vec4<f32> {
    let luma = dot(vec3<f32>(0.2126, 0.7152, 0.0722), color.rgb);
    let cellUV = fract(uv / normalizedPixelSize);

    var lineWidth = 0.0;
    if (luma > 0.0) {
        lineWidth = 1.0;
    }
    if (luma > 0.3) {
        lineWidth = 0.7;
    }
    if (luma > 0.5) {
        lineWidth = 0.5;
    }
    if (luma > 0.7) {
        lineWidth = 0.3;
    }
    if (luma > 0.9) {
        lineWidth = 0.1;
    }
    if (luma > 0.99) {
        lineWidth = 0.0;
    }

    let yStart = 0.05;
    let yEnd = 0.95;

    if (cellUV.y > yStart && cellUV.y < yEnd && cellUV.x > 0.0 && cellUV.x < lineWidth) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else {
        return vec4<f32>(0.70, 0.74, 0.73, 1.0);
    }
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    var dx: vec4<f32> = vec4(0.0);
    var dy: vec4<f32> = vec4(0.0);
    let kernel_width = 2;
    let uv_pixel = vec2(1.0, 1.0) / vec2(resolution.width, resolution.height);
    for(var y = -kernel_width; y <= kernel_width; y = y + 1){
        for(var x = -kernel_width; x <= kernel_width; x = x + 1){
            let point: vec2<f32> = in.uv + uv_pixel * vec2(f32(x), f32(y));
            let tex_color = textureSample(sceneTex, sceneSampler, point);
            dx += SOBEL_KERNEL_X[y + kernel_width][x + kernel_width] * tex_color;
            dy += SOBEL_KERNEL_Y[y + kernel_width][x + kernel_width] * tex_color;
        }
    }

    let middle_color = textureSample(sceneTex, sceneSampler, in.uv);

    let edge_magnitude = length(abs(dx) + abs(dy));
    let threshold = 0.15;
    if edge_magnitude > threshold {
        // render strong edge
        return middle_color;
    }else{
        // render subtle dithered pattern
        let x = i32(in.uv.x * resolution.width);
        let y = i32(in.uv.y * resolution.height);
        let band_width = 2;
        let band_height = 2;
        if (y / band_height) % 2 == 0 {
            return middle_color * 0.5;
        }else{
            return vec4(0.0);
        }
    }

    // return vec4<f32>(edge.rgb, 1.0);

    // let normalizedPixelSize = 4.0 / vec2<f32>(resolution.width, resolution.height);
    // let uvPixel = normalizedPixelSize * floor(in.uv / normalizedPixelSize);

    // let color = textureSample(sceneTex, sceneSampler, uvPixel); 

    // return hatch_effect(color, in.uv, normalizedPixelSize);

    // return textureSample(sceneTex, sceneSampler, uvPixel);

    // let color = textureSample(sceneTex, sceneSampler, in.uv);
    // let lum = dot(vec3(0.2126, 0.7152, 0.0722), color.xyz);

    // let x = i32(in.uv.x * 1024) % 8;
    // let y = i32(in.uv.y * 800) % 8;
    // let threshold = bayerMatrix2x2[y][x];

    // if (lum < threshold + 0.20) {
    //     return vec4(0.0);
    // } else {
    //     return vec4(1.0); 
    // }
}

