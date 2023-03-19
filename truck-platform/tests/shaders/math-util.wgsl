@vertex
fn vs_main(@location(0) idx: u32) -> @builtin(position) vec4<f32> {
    var vertex: array<vec2<f32>, 4>;
    vertex[0] = vec2<f32>(-1.0, -1.0);
    vertex[1] = vec2<f32>(1.0, -1.0);
    vertex[2] = vec2<f32>(-1.0, 1.0);
    vertex[3] = vec2<f32>(1.0, 1.0);
    return vec4<f32>(vertex[idx], 0.0, 1.0);
}

const EPS = 1.0e-5;

@fragment
fn fs_main() -> @location(0) vec4<f32> {
	let e = vec2<f32>(1.0, 0.0);

	var m2 = mat2x2<f32>(2.0, 3.0, -4.0, 1.0);
	m2 = m2 * inverse2(m2);
	if (distance(m2[0], e.xy) > EPS) {
		return vec4<f32>(1.0, 0.0, 0.0, 1.0);
	} else if (distance(m2[1], e.yx) > EPS) {
		return vec4<f32>(1.0, 0.0, 0.0, 1.0);
	}

	var m3 = mat3x3<f32>(2.0, 3.0, -4.0, -1.0, 2.0, 4.0, 5.0, -2.0, 3.0);
	m3 = m3 * inverse3(m3);
	if (distance(m3[0], e.xyy) > EPS) {
		return vec4<f32>(0.0, 1.0, 0.0, 1.0);
	} else if (distance(m3[1], e.yxy) > EPS) {
		return vec4<f32>(0.0, 1.0, 0.0, 1.0);
	} else if (distance(m3[2], e.yyx) > EPS) {
		return vec4<f32>(0.0, 1.0, 0.0, 1.0);
	}
	
	var m4 = mat4x4<f32>(2.0, 3.0, -4.0, 1.0, -1.0, 2.0, 4.0, -2.0, 5.0, -2.0, 3.0, 6.0, -1.0, 2.0, -3.0, 4.0);
	m4 = m4 * inverse4(m4);
	if (distance(m4[0], e.xyyy) > EPS) {
		return vec4<f32>(0.0, 0.0, 1.0, 1.0);
	} else if (distance(m4[1], e.yxyy) > EPS) {
		return vec4<f32>(0.0, 0.0, 1.0, 1.0);
	} else if (distance(m4[2], e.yyxy) > EPS) {
		return vec4<f32>(0.0, 0.0, 1.0, 1.0);
	} else if (distance(m4[3], e.yyyx) > EPS) {
		return vec4<f32>(0.0, 0.0, 1.0, 1.0);
	}

	let mr2 = rotate2D(PI / 3.0);
	if (distance(mr2[0], vec2<f32>(1.0, sqrt(3.0)) / 2.0) > EPS) {
		return vec4<f32>(1.0, 1.0, 0.0, 1.0);
	} else if (distance(mr2[1], vec2<f32>(-sqrt(3.0), 1.0) / 2.0) > EPS) {
		return vec4<f32>(1.0, 1.0, 0.0, 1.0);
	}

	let mr3 = rotate3D(TAU / 3.0, vec3<f32>(1.0, 1.0, 1.0));
	if (distance(mr3[0], e.yxy) > EPS) {
		return vec4<f32>(0.0, 1.0, 1.0, 1.0);
	} else if (distance(mr3[1], e.yyx) > EPS) {
		return vec4<f32>(0.0, 1.0, 1.0, 1.0);
	} else if (distance(mr3[2], e.xyy) > EPS) {
		return vec4<f32>(0.0, 1.0, 1.0, 1.0);
	}

    return vec4<f32>(0.2, 0.4, 0.6, 0.8);
}

@fragment
fn fs_main_anti() -> @location(0) vec4<f32> {
	let e = vec2<f32>(1.0, 0.0);

	var m2 = mat2x2<f32>(2.0, 3.0, -4.0, 1.0);
	m2 = m2 * inverse2(m2);
	if (distance(m2[0], e.xy) > EPS) {
		return vec4<f32>(1.0, 0.0, 0.0, 1.0);
	} else if (distance(m2[1], e.yx) > EPS) {
		return vec4<f32>(1.0, 0.0, 0.0, 1.0);
	}

	var m3 = mat3x3<f32>(2.0, 3.0, -4.0, -1.0, 2.0, 4.0, 5.0, -2.0, 3.0);
	m3 = m3 * inverse3(m3);
	if (distance(m3[0], e.xyy) > EPS) {
		return vec4<f32>(0.0, 1.0, 0.0, 1.0);
	} else if (distance(m3[1], e.yxy) > EPS) {
		return vec4<f32>(0.0, 1.0, 0.0, 1.0);
	} else if (distance(m3[2], e.yyx) > EPS) {
		return vec4<f32>(0.0, 1.0, 0.0, 1.0);
	}
	
	var m4 = mat4x4<f32>(2.0, 3.0, -4.0, 1.0, -1.0, 2.0, 4.0, -2.0, 5.0, -2.0, 3.0, 6.0, -1.0, 2.0, -3.0, 4.0);
	m4 = m4 * inverse4(m4);
	if (distance(m4[0], e.xyyy) > EPS) {
		return vec4<f32>(0.0, 0.0, 1.0, 1.0);
	} else if (distance(m4[1], e.yxyy) > EPS) {
		return vec4<f32>(0.0, 0.0, 1.0, 1.0);
	} else if (distance(m4[2], e.yyxy) > EPS) {
		return vec4<f32>(0.0, 0.0, 1.0, 1.0);
	} else if (distance(m4[3], e.yyyx) > EPS) {
		return vec4<f32>(0.0, 0.0, 1.0, 1.0);
	}

	let mr2 = rotate2D(PI / 3.0);
	if (distance(mr2[0], vec2<f32>(1.0, sqrt(3.0)) / 2.0) > EPS) {
		return vec4<f32>(1.0, 1.0, 0.0, 1.0);
	} else if (distance(mr2[1], vec2<f32>(-sqrt(3.0), 1.0) / 2.0) > EPS) {
		return vec4<f32>(1.0, 1.0, 0.0, 1.0);
	}

	let mr3 = rotate3D(TAU / 3.0, vec3<f32>(1.0, 1.0, 1.0));
	if (distance(mr3[0], e.yxy) < EPS) {
		return vec4<f32>(0.0, 1.0, 1.0, 1.0);
	} else if (distance(mr3[1], e.yyx) > EPS) {
		return vec4<f32>(0.0, 1.0, 1.0, 1.0);
	} else if (distance(mr3[2], e.xyy) > EPS) {
		return vec4<f32>(0.0, 1.0, 1.0, 1.0);
	}

    return vec4<f32>(0.2, 0.4, 0.6, 0.8);
}
