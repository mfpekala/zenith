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

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let input_x = (-x_offset + 2.0 + x_repetitions * in.uv[0]) % 1.0;
    let input_y = (y_offset + y_repetitions * in.uv[1]) % 1.0;
    let index_lower = (1.0 / length) * (index + 0);
    let index_upper = (1.0 / length) * (index + 1);
    let out_uv = vec2<f32>(index_lower + (index_upper - index_lower) * input_x, input_y);
    return textureSample(texture, splr, out_uv);
}
