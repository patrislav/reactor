// Adapted from https://github.com/gavlig/bevy_crt_galore/blob/master/assets/shaders/endesga/pass0.wgsl
// by ENDESGA and gavlig

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct Globals {
    // The time since startup in seconds
    // Wraps to 0 after 1 hour.
    time: f32,
    // The delta time since the previous frame in seconds
    delta_time: f32,
    // Frame count since the start of the app.
    // It wraps to zero when it reaches the maximum value of a u32.
    frame_count: u32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    // WebGL2 structs must be 16 byte aligned.
    _webgl2_padding: f32
#endif
};

struct Settings {
    noise_amount: f32,
    vignette_amount: f32,
    aberration_amount: f32,
    // WebGL2 structs must be 16 byte aligned.
    _webgl2_padding: f32
};

@group(0) @binding(0) var image: texture_2d<f32>;
@group(0) @binding(1) var image_sampler: sampler;
@group(0) @binding(2) var<uniform> globals: Globals;
@group(0) @binding(3) var<uniform> settings: Settings;

@fragment
fn fragment(in: FullscreenVertexOutput,) -> @location(0) vec4<f32> {
    let aber_dis: vec2<f32> = (in.uv - vec2<f32>(0.5)) * settings.aberration_amount * length(in.uv - 0.5);
    let aberration = vec3<f32>(
        textureSample(image, image_sampler, in.uv).r,
        textureSample(image, image_sampler, in.uv - aber_dis).g,
        textureSample(image, image_sampler, in.uv - 2.0 * aber_dis).b
    );

    let resolution_u32 = textureDimensions(image);
    let resolution = vec2<f32>(f32(resolution_u32.x), f32(resolution_u32.y));
    let screen_ratio_y = resolution.y / resolution.x;

	let vignette_step = smoothstep(
		0.25,
		1.0,
		length((in.uv - vec2f(0.5)) * vec2<f32>(1.0, screen_ratio_y * 2.0))
	);
	let vignette = mix(
		1.0,
		1.0 - clamp(vignette_step, 0.0, 1.0),
		settings.vignette_amount
	);

    let frag_coord = in.uv * resolution;
    let frame: f32 = floor(f32(globals.frame_count));
	let rgb_grain = vec3<f32>(
		grain(vec3<f32>(frag_coord, frame)),
		grain(vec3<f32>(frag_coord, frame + 9.0)),
		grain(vec3<f32>(frag_coord, frame - 9.0))
	);
    let aberration_wgrain = mix(aberration, mix(aberration * rgb_grain, aberration + (rgb_grain - 1.0), 0.5), settings.noise_amount);
    return vec4<f32>(aberration_wgrain * vignette, 1.0);
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
