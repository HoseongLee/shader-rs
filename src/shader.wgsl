struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn palette(t: f32) -> vec3f {
    let a = vec3(0.5, 0.5, 0.5);
    let b = vec3(0.5, 0.5, 0.5);
    let c = vec3(1.0, 1.0, 1.0);
    let d = vec3(0.263,0.416,0.557);

    return a + b*cos( 6.28318*(c*t+d) );
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let uv0 = (pos.xy * 2.0 - uniforms.resolution) / uniforms.resolution.y;

    var uv = uv0;
    var finalColor = vec3(0.);

    for (var i = 0.0; i < 4.0; i=i+1.) {
        uv = fract(uv * 1.5) - 0.5;

        var d = length(uv) * exp(-length(uv0));

        let col = palette(length(uv0) + i*.4 + uniforms.time *.4);

        d = sin(d*8. + uniforms.time)/8.;
        d = abs(d);

        d = pow(0.01 / d, 1.2);

        finalColor += col * d;
    }

    return vec4(finalColor, 1.);
}
