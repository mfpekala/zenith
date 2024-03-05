#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
struct PostProcessSettings {
    num_pixels: f32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    _webgl2_padding: vec3<f32>
#endif
}
@group(0) @binding(2) var<uniform> settings: PostProcessSettings;

const GAUSSIAN_KERNEL_SIZE: i32 = 9;
const GAUSSIAN_KERNEL: array<f32, GAUSSIAN_KERNEL_SIZE> = array<f32, GAUSSIAN_KERNEL_SIZE>(
    0.05, 0.09, 0.12, 0.15, 0.16, 0.15, 0.12, 0.09, 0.05
);

fn gaussian_blur(texture: texture_2d<f32>, splr: sampler, uv: vec2<f32>, horizontal: bool) -> vec3<f32> {
    var blur_color: vec3<f32> = vec3<f32>(0.0);

    // Unroll the loop manually
    let kernel_offset = GAUSSIAN_KERNEL_SIZE / 2;
    let kernel_weight = 1.0 / f32(GAUSSIAN_KERNEL_SIZE);
    let bloom_radius = 1.5;

    var offset_0 = vec2<f32>(-4.0, 0.0);
    var offset_1 = vec2<f32>(-3.0, 0.0);
    var offset_2 = vec2<f32>(-2.0, 0.0);
    var offset_3 = vec2<f32>(-1.0, 0.0);
    var offset_4 = vec2<f32>(0.0, 0.0);
    var offset_5 = vec2<f32>(1.0, 0.0);
    var offset_6 = vec2<f32>(2.0, 0.0);
    var offset_7 = vec2<f32>(3.0, 0.0);
    var offset_8 = vec2<f32>(4.0, 0.0);

    if (!horizontal) {
        offset_0 = vec2<f32>(0.0, -4.0);
        offset_1 = vec2<f32>(0.0, -3.0);
        offset_2 = vec2<f32>(0.0, -2.0);
        offset_3 = vec2<f32>(0.0, -1.0);
        offset_4 = vec2<f32>(0.0, 0.0);
        offset_5 = vec2<f32>(0.0, 1.0);
        offset_6 = vec2<f32>(0.0, 2.0);
        offset_7 = vec2<f32>(0.0, 3.0);
        offset_8 = vec2<f32>(0.0, 4.0);
    }

    let sample_color_0 = textureSample(texture, splr, uv + offset_0 * bloom_radius / settings.num_pixels);
    let sample_color_1 = textureSample(texture, splr, uv + offset_1 * bloom_radius / settings.num_pixels);
    let sample_color_2 = textureSample(texture, splr, uv + offset_2 * bloom_radius / settings.num_pixels);
    let sample_color_3 = textureSample(texture, splr, uv + offset_3 * bloom_radius / settings.num_pixels);
    let sample_color_4 = textureSample(texture, splr, uv + offset_4 * bloom_radius / settings.num_pixels);
    let sample_color_5 = textureSample(texture, splr, uv + offset_5 * bloom_radius / settings.num_pixels);
    let sample_color_6 = textureSample(texture, splr, uv + offset_6 * bloom_radius / settings.num_pixels);
    let sample_color_7 = textureSample(texture, splr, uv + offset_7 * bloom_radius / settings.num_pixels);
    let sample_color_8 = textureSample(texture, splr, uv + offset_8 * bloom_radius / settings.num_pixels);

    blur_color = (
        sample_color_0.rgb * GAUSSIAN_KERNEL[0] +
        sample_color_1.rgb * GAUSSIAN_KERNEL[1] +
        sample_color_2.rgb * GAUSSIAN_KERNEL[2] +
        sample_color_3.rgb * GAUSSIAN_KERNEL[3] +
        sample_color_4.rgb * GAUSSIAN_KERNEL[4] +
        sample_color_5.rgb * GAUSSIAN_KERNEL[5] +
        sample_color_6.rgb * GAUSSIAN_KERNEL[6] +
        sample_color_7.rgb * GAUSSIAN_KERNEL[7] +
        sample_color_8.rgb * GAUSSIAN_KERNEL[8]
    ) * kernel_weight;

    return blur_color;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // return textureSample(screen_texture, texture_sampler, in.uv);
    // let num_pixels = settings.num_pixels;

    // let pixelated_uv = floor(in.uv * vec2<f32>(num_pixels, num_pixels)) / vec2<f32>(num_pixels, num_pixels);
    // let original_color = textureSample(screen_texture, texture_sampler, pixelated_uv);

    // return original_color;
    let color = textureSample(screen_texture, texture_sampler, in.uv);

    let bloom_threshold = 0.0;
    let bloom_intensity = 7.5;

    let bright_areas = max(color.rgb - bloom_threshold, vec3<f32>(0.0));

    let blurred_horizontal = gaussian_blur(screen_texture, texture_sampler, in.uv, true);
    // let blurred_bright_areas = gaussian_blur(screen_texture, texture_sampler, in.uv, false);

    let bloom_color = color.rgb + blurred_horizontal * bloom_intensity;

    return vec4<f32>(bloom_color, 1.0);
}
