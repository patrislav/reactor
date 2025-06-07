#import bevy_sprite::{
    mesh2d_view_bindings::globals,
    mesh2d_view_bindings::view,
    mesh2d_functions::{get_world_from_local, mesh2d_position_local_to_clip},
}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(2) @binding(100) var mask_texture: texture_2d<f32>;
@group(2) @binding(101) var mask_sampler: sampler;
@group(2) @binding(102) var<uniform> scroll_speed: vec2<f32>;
@group(2) @binding(103) var<uniform> bg_color: vec4<f32>;
@group(2) @binding(104) var<uniform> fg_color: vec4<f32>;
@group(2) @binding(105) var<uniform> base_color: vec4<f32>;
@group(2) @binding(106) var<uniform> intensity: f32;

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let world_from_local = get_world_from_local(vertex.instance_index);
    out.clip_position = mesh2d_position_local_to_clip(world_from_local, vec4<f32>(vertex.position, 1.0));
    out.uv = vertex.uv;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let scrolled_uv = fract(in.uv + scroll_speed * globals.time);
    let mask = textureSample(mask_texture, mask_sampler, scrolled_uv);

    let final_bg = mix(base_color, bg_color, intensity);
    let final_fg = mix(base_color, fg_color, intensity);

    return mix(final_bg, final_fg, mask.r);
}

