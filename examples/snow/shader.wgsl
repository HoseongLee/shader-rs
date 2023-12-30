struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    var snow = 0.0;

    let t = uniforms.time;
    let uv = pos.xy / uniforms.resolution.x ;
    let gradient = pos.y / uniforms.resolution.y + 0.3;

    let c = cos(t * 2.5) * 0.3;
    let s = sin(t * 2.5) * 0.3;

    for (var k = 0.7; k < 3.0; k = k + 1.3) {

        for (var i = 3.0; i < 9.0; i = i + 0.97) {
            let cellSize = 2.0 + 3.0 * i;

            let uvShifted = uv - vec2(sin(k * 73.9 + t + i * 87.7) * 0.1, t - cos(k * 57.1 + t + i * 99.7) * 0.3) / sqrt(i) / 3.0;
            let uvStep = ceil(uvShifted * cellSize - 0.5) / cellSize;

            if fract(sin(dot(uvStep, vec2(17., 19.)) * k) * 129.) < 0.1 {
                let x = cos(dot(uvStep, vec2(i + t, k * 94.7)));
                let y = sin(dot(uvStep, vec2(k + t, i * 52.1)));

                let d = distance(uvStep + vec2(x * c, y * s) / cellSize, uvShifted);

                snow += 1.0 - smoothstep(0.0, 0.05 / sqrt(cellSize), d);
            }
        }
    }

    return vec4(snow, snow, snow, 1.0) + 0.4 * gradient * vec4(0.4, 0.8, 1.0, 0.0);
}
