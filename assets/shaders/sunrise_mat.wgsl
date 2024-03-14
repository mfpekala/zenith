#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(1)
var<uniform> time_frac: f32;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let height_blueness = (cos(in.uv[1] * 1.5)) * 0.9;
    let width_blueness = 0.7 + (0.1 + sin(in.uv[0] * 3.1415926)) * 0.3;

    return vec4<f32>(
        0.2 * height_blueness * height_blueness,
        0.05 * height_blueness * height_blueness * height_blueness,
        width_blueness * height_blueness, 
        time_frac * 0.6,
    );
}
