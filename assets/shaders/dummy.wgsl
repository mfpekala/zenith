#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(1)
var texture: texture_2d<f32>;
@group(2) @binding(2)
var splr: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // return  textureSample(texture, splr, vo.uv);
    let num_pixels = 160.0;
    let pixelated_uv = floor(in.uv * vec2<f32>(num_pixels, num_pixels)) / vec2<f32>(num_pixels, num_pixels);
    let original_color = textureSample(texture, splr, pixelated_uv);
    return original_color;
}
