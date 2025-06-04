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

@group(2) @binding(100) var texture: texture_2d<f32>;
@group(2) @binding(101) var texture_sampler: sampler;

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
    var color = textureSampleBias(texture, texture_sampler, in.uv, view.mip_bias);
    return color;
}

