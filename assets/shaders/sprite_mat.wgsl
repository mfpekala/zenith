#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(1)
var texture: texture_2d<f32>;
@group(2) @binding(2)
var splr: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return  textureSample(texture, splr, in.uv);
}
