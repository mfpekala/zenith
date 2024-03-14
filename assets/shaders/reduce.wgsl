#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(1)
var texture: texture_2d<f32>;
@group(2) @binding(2)
var splr: sampler;
@group(2) @binding(3)
var<uniform> num_pixels_w: f32;
@group(2) @binding(4)
var<uniform> num_pixels_h: f32;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Okay wow I don't even need to do this but I guess could be handy
    let pixelated_uv = floor(in.uv * vec2<f32>(num_pixels_w, num_pixels_h)) / vec2<f32>(num_pixels_w, num_pixels_h);
    let original_color = textureSample(texture, splr, pixelated_uv);
    return original_color;
}
