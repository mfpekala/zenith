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
    return  textureSample(texture, splr, in.uv);
}
