#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
struct PostProcessSettings {
    num_pixels: f32,
    left: f32,
    top: f32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    _webgl2_padding: vec3<f32>
#endif
}
@group(0) @binding(2) var<uniform> settings: PostProcessSettings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let num_pixels = settings.num_pixels;
    let left = settings.left;
    let top = settings.top;

    let pixelated_uv = floor(in.uv * vec2<f32>(num_pixels, num_pixels)) / vec2<f32>(num_pixels, num_pixels);
    let pixelated_color = textureSample(screen_texture, texture_sampler, pixelated_uv);

    return pixelated_color;
}
