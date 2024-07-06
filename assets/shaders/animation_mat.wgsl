#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(1)
var texture: texture_2d<f32>;
@group(2) @binding(2)
var splr: sampler;
@group(2) @binding(3)
var<uniform> index: f32;
@group(2) @binding(4)
var<uniform> length: f32;
@group(2) @binding(5)
var<uniform> x_offset: f32;
@group(2) @binding(6)
var<uniform> y_offset: f32;
@group(2) @binding(7)
var<uniform> x_repetitions: f32;
@group(2) @binding(8)
var<uniform> y_repetitions: f32;
@group(2) @binding(9)
var<uniform> r: f32;
@group(2) @binding(10)
var<uniform> g: f32;
@group(2) @binding(11)
var<uniform> b: f32;
@group(2) @binding(12)
var<uniform> rot: f32;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let shifted = vec2<f32>((in.uv.x - 0.5) * 2.0, (in.uv.y - 0.5) * 2.0);
    let cs = cos(rot);
    let sn = sin(rot);
    let scaled = vec2<f32>(shifted.x * x_repetitions, shifted.y * y_repetitions);
    let rotated = vec2<f32>(cs * scaled.x - sn * scaled.y, sn * scaled.x + cs * scaled.y);
    let unshifted = vec2<f32>(rotated.x / 2.0 + 0.5, rotated.y / 2.0 + 0.5);
    let input_x = (-x_offset + 20.0 + unshifted.x) % 1.0;
    let input_y = (y_offset + unshifted.y) % 1.0;
    let index_lower = (1.0 / length) * (index + 0);
    let index_upper = (1.0 / length) * (index + 1);
    let out_uv = vec2<f32>(index_lower + (index_upper - index_lower) * input_x, input_y);
    let out_rgba = textureSample(texture, splr, out_uv);

    return vec4<f32>(out_rgba[0] * r, out_rgba[1] * g, out_rgba[2] * b, out_rgba[3]);
}
