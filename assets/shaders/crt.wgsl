// Adapted from https://github.com/gavlig/bevy_crt_galore/blob/master/assets/shaders/endesga/pass0.wgsl
// by ENDESGA and gavlig

#import bevy_pbr::{
    mesh_view_bindings::globals,
    mesh_view_bindings::view,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

struct MyExtendedMaterial {
    noise_amount: f32,
    vignette_amount: f32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    // WebGL2 structs must be 16 byte aligned.
    _webgl2_padding: vec2<f32>
#endif
}


@group(2) @binding(100) var image: texture_2d<f32>;
@group(2) @binding(101) var image_sampler: sampler;
@group(2) @binding(102) var<uniform> noise_amount: f32;
@group(2) @binding(103) var<uniform> vignette_amount: f32;
@group(2) @binding(104) var<uniform> aberration_amount: f32;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(in, is_front);


    let aber_dis: vec2<f32> = (in.uv - vec2<f32>(0.5)) * aberration_amount * length(in.uv - 0.5);
    let aberration = vec3<f32>(
        textureSampleBias(image, image_sampler, in.uv, view.mip_bias).r,
        textureSampleBias(image, image_sampler, in.uv - aber_dis, view.mip_bias).g,
        textureSampleBias(image, image_sampler, in.uv - 2.0 * aber_dis, view.mip_bias).b
    );


    let resolution = vec2<f32>(1024.0, 768.0);
    let screen_ratio_y = resolution.y / resolution.x;

	let vignette_step = smoothstep(
		0.25,
		1.0,
		length((in.uv - vec2f(0.5)) * vec2f(1.0, screen_ratio_y * 2.0))
	);
	let vignette = mix(
		1.0,
		1.0 - clamp(vignette_step, 0.0, 1.0),
		vignette_amount
	);

    let frag_coord = in.uv * resolution;
    let frame: f32 = floor(f32(globals.frame_count));
	let rgb_grain = vec3f(
		grain(vec3<f32>(frag_coord, frame)),
		grain(vec3<f32>(frag_coord, frame + 9.0)),
		grain(vec3<f32>(frag_coord, frame - 9.0))
	);
    let aberration_wgrain = mix(aberration, mix(aberration * rgb_grain, aberration + (rgb_grain - 1.0), 0.5), noise_amount);
    pbr_input.material.base_color = vec4<f32>(aberration_wgrain * vignette, 1.0);

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
    // in deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

    return out;
}

fn hash(p: vec3f) -> f32 {
	var p_var = p;
	p_var = fract(p_var * 0.1031);
	p_var = p_var + (dot(p_var, p_var.yzx + 19.19));
	return fract((p_var.x + p_var.y) * p_var.z);
}

fn noise(x: vec3f) -> f32 {
	let p: vec3f = floor(x);
	let f: vec3f = fract(x);
	let m: vec3f = f * f * (3. - 2. * f);
	let i: vec3f = p + vec3f(1., 0., 0.);
	let hash: vec4f = vec4f(hash(p), hash(i), hash(p + vec3f(0., 1., 0.)), hash(i + vec3f(0., 1., 0.)));
	return mix(mix(hash.x, hash.y, m.x), mix(hash.z, hash.w, m.x), m.y);
}

fn grain(x: vec3f) -> f32 {
	return 0.5 + (4. * noise(x) - noise(x + 1.) + noise(x - 1.)) / 4.;
}
