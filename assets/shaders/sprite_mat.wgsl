#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(1)
var texture: texture_2d<f32>;
@group(2) @binding(2)
var splr: sampler;
@group(2) @binding(3)
var<uniform> x: f32;
@group(2) @binding(4)
var<uniform> y: f32;
@group(2) @binding(5)
var<uniform> w: f32;
@group(2) @binding(6)
var<uniform> h: f32;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let box_x = x + w * in.uv[0];
    let box_y = y + h * in.uv[1];
    let box_uv = vec2<f32>(box_x, box_y);
    return  textureSample(texture, splr, box_uv);
}
